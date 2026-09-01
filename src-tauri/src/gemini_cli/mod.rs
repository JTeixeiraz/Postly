//! Gemini CLI como provedor de turno, ao lado do Ollama e do Claude Code.
//!
//! Mesmo contrato de sempre: entra system + prompt, sai texto. O que muda e
//! quem executa. E a tese do produto sobrevive de novo — o nivel do cargo
//! continua proporcional ao que ele entrega, so que agora o eixo e a familia
//! do modelo:
//!
//!   alto  -> Pro         decide, le mercado, julga a peca
//!   medio -> Flash       audita dentro de criterio recebido
//!   baixo -> Flash-Lite  cumpre briefing pronto
//!
//! CONTRATO MEDIDO CONTRA O CLI 0.50.0 NESTA MAQUINA, nao presumido. Tres
//! achados mudaram o desenho, e nenhum deles estava na minha cabeca:
//!
//! 1. NAO EXISTE `--system-prompt`. O system entra por `GEMINI_SYSTEM_MD`,
//!    apontando um arquivo Markdown — e ele SUBSTITUI o prompt embutido em vez
//!    de somar. Isso e sorte nossa: o embutido e de um agente de codigo, cheio
//!    de instrucao sobre ler e editar arquivo, que nao tem nada a ver com um
//!    cargo de marketing.
//!
//! 2. O CLI RECUSA RODAR FORA DE UM DIRETORIO CONFIAVEL. Medido:
//!
//!    ```text
//!    Gemini CLI is not running in a trusted directory. To proceed, either use
//!    `--skip-trust`, set the `GEMINI_CLI_TRUST_WORKSPACE=true` environment
//!    variable, or trust this directory in interactive mode.
//!    ```
//!
//!    Sem `--skip-trust` a integracao falharia para todo mundo que nunca abriu
//!    o CLI a mao naquela pasta — a mesma armadilha do PATH do Claude Code:
//!    funciona na maquina de quem escreveu e em mais nenhuma.
//!
//! 3. O ENVELOPE DE ERRO SAI PELO STDERR, com o stdout vazio:
//!
//!    ```json
//!    {"session_id":"…","error":{"type":"Error","message":"…","code":41}}
//!    ```
//!
//!    Ler so o stdout faria todo erro de autenticacao chegar disfarcado de
//!    "resposta vazia" — o pior tipo de erro, o que aponta para o lugar errado.
//!
//! A autenticacao e o preparo do ambiente moram em `ambiente.rs`, com a nota
//! de por que aqui as variaveis de credencial sao avisadas e nao removidas.

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub mod ambiente;
pub mod limite;

pub use ambiente::{credencial_externa_no_ambiente, disponivel, localizar, versao};

use crate::orchestrator::roles::Tier;

/// Os tres modelos sao os proprios padroes do CLI, lidos do binario 0.50.0
/// (`DEFAULT_GEMINI_MODEL`, `DEFAULT_GEMINI_FLASH_MODEL` e o par flash-lite).
///
/// Escolher a familia estavel em vez dos `-preview` e deliberado: um preview
/// some sem aviso, e um cargo apontando para modelo inexistente falha no meio
/// da campanha, nao na tela de configuracao onde daria para consertar.
const PRO: &str = "gemini-2.5-pro";
const FLASH: &str = "gemini-2.5-flash";
const FLASH_LITE: &str = "gemini-2.5-flash-lite";

pub fn modelo_do_nivel(tier: Tier) -> &'static str {
    modelo_do_nivel_com(tier, crate::prefs::load().modo)
}

/// O modo de desempenho vale aqui pelo mesmo motivo que vale no Claude Code: a
/// intencao de quem escolhe e a mesma, e um seletor que funcionasse num
/// provedor e nao no outro seria confuso. O eixo, como la, e custo.
pub fn modelo_do_nivel_com(tier: Tier, modo: crate::prefs::ModoDesempenho) -> &'static str {
    use crate::prefs::ModoDesempenho as M;
    match (modo, tier) {
        (M::Economico, Tier::Alto) => FLASH,
        (M::Economico, _) => FLASH_LITE,

        (M::Normal, Tier::Alto) => PRO,
        (M::Normal, Tier::Medio) => FLASH,
        (M::Normal, Tier::Baixo) => FLASH_LITE,

        // No maximo o auditor sobe junto de quem decide: julgar a peca e a
        // segunda decisao mais cara da campanha.
        (M::Maximo, Tier::Baixo) => FLASH,
        (M::Maximo, _) => PRO,
    }
}

pub fn rotulo_do_modelo(id: &str) -> &'static str {
    match id {
        PRO => "Gemini 2.5 Pro",
        FLASH => "Gemini 2.5 Flash",
        FLASH_LITE => "Gemini 2.5 Flash-Lite",
        _ => "Gemini",
    }
}

#[derive(Debug, Deserialize)]
struct RespostaCli {
    #[serde(default)]
    response: Option<String>,
    #[serde(default)]
    error: Option<ErroCli>,
}

#[derive(Debug, Deserialize)]
struct ErroCli {
    #[serde(default)]
    message: String,
    #[serde(default)]
    code: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnoGemini {
    pub texto: String,
}

/// Por que o turno nao saiu.
///
/// Mesma forma do `claude::ErroTurno`, e pelo mesmo motivo: o orquestrador
/// precisa distinguir "acabou a cota" (que pode virar espera) de todo o resto
/// (que encerra), e reconhecer isso pelo texto da mensagem quebraria na
/// primeira traducao.
#[derive(Debug, Clone)]
pub enum ErroTurno {
    Limite(crate::claude::limite::Limite),
    Outro(String),
}

impl std::fmt::Display for ErroTurno {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroTurno::Limite(l) => write!(
                f,
                "{} {}",
                crate::idioma::msg(
                    "A cota do Gemini CLI acabou.",
                    "The Gemini CLI quota ran out."
                ),
                l.evidencia.lines().next().unwrap_or("")
            ),
            ErroTurno::Outro(m) => write!(f, "{m}"),
        }
    }
}

impl From<ErroTurno> for String {
    fn from(e: ErroTurno) -> Self {
        e.to_string()
    }
}

/// Roda um turno.
///
/// O prompt vai por stdin — o `-p ""` faz o CLI ler dali, e a documentacao do
/// proprio binario diz que o valor de `-p` e "appended to input on stdin". A
/// razao e a mesma do Claude Code: um briefing tem milhares de caracteres e
/// passa do limite de argumento em algumas plataformas.
pub async fn turno(
    tier: Tier,
    system: &str,
    prompt: &str,
    timeout_s: u64,
) -> Result<TurnoGemini, ErroTurno> {
    let Some(binario) = ambiente::localizar() else {
        return Err(ErroTurno::Outro(crate::idioma::msg(
            "Gemini CLI nao encontrado nesta maquina. Instale com `npm i -g @google/gemini-cli` ou volte para o Ollama.",
            "Gemini CLI was not found on this machine. Install it with `npm i -g @google/gemini-cli` or switch back to Ollama.",
        )));
    };

    let dir = ambiente::pasta_de_trabalho().map_err(ErroTurno::Outro)?;

    // O system vira arquivo porque o CLI nao tem flag para ele. Fica na pasta
    // de trabalho do proprio provedor, e nao em /tmp: e dado da sessao de quem
    // usa, e o diretorio de dados e onde esse dado mora.
    let system_md = dir.join("system.md");
    std::fs::write(&system_md, system)
        .map_err(|e| ErroTurno::Outro(format!("falha ao gravar o system prompt: {e}")))?;

    let mut filho = Command::new(binario)
        .current_dir(&dir)
        // Substitui o prompt de agente de codigo embutido pelo do cargo.
        .env("GEMINI_SYSTEM_MD", &system_md)
        .args([
            "-p",
            "",
            "--output-format",
            "json",
            "--model",
            modelo_do_nivel(tier),
            // Sem isto o CLI recusa rodar numa pasta que ninguem abriu a mao.
            "--skip-trust",
            // Nenhuma ferramenta e oferecida, entao nao ha o que aprovar; sem
            // a flag o CLI esperaria uma confirmacao que nunca chega num
            // processo sem terminal, e o turno morreria no tempo limite.
            "--approval-mode",
            "yolo",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| ErroTurno::Outro(format!("falha ao iniciar o Gemini CLI: {e}")))?;

    if let Some(mut entrada) = filho.stdin.take() {
        entrada
            .write_all(prompt.as_bytes())
            .await
            .map_err(|e| ErroTurno::Outro(format!("falha ao enviar o prompt: {e}")))?;
        entrada.shutdown().await.ok();
    }

    let saida = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_s),
        filho.wait_with_output(),
    )
    .await
    .map_err(|_| {
        ErroTurno::Outro(crate::idioma::msg(
            "O Gemini CLI passou do tempo limite deste turno.",
            "Gemini CLI exceeded this turn's time limit.",
        ))
    })?
    .map_err(|e| ErroTurno::Outro(format!("falha ao ler a resposta: {e}")))?;

    let bruto = String::from_utf8_lossy(&saida.stdout);
    let erro_bruto = String::from_utf8_lossy(&saida.stderr);

    // A cota e conferida nas DUAS saidas e antes de tentar ler a resposta:
    // medido, o envelope de erro sai pelo stderr com o stdout vazio.
    let junto = format!("{bruto}\n{erro_bruto}");
    if let Some(l) = limite::detectar(&junto) {
        return Err(ErroTurno::Limite(l));
    }

    // O envelope pode estar em qualquer uma das duas saidas: sucesso no
    // stdout, erro no stderr.
    let envelope = achar_envelope(&bruto).or_else(|| achar_envelope(&erro_bruto));

    if let Some(e) = envelope.as_ref().and_then(|r| r.error.as_ref()) {
        return Err(ErroTurno::Outro(explicar(e, &erro_bruto)));
    }

    let texto = envelope
        .and_then(|r| r.response)
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .ok_or_else(|| {
            ErroTurno::Outro(format!(
                "{} {}",
                crate::idioma::msg(
                    "O Gemini CLI nao devolveu resposta.",
                    "Gemini CLI returned no response."
                ),
                primeira_linha_util(&erro_bruto)
            ))
        })?;

    Ok(TurnoGemini { texto })
}

/// Le o envelope JSON de uma das saidas.
///
/// Nao basta `from_str` no texto inteiro: medido, o CLI as vezes escreve um
/// aviso em texto puro na mesma saida do JSON (foi assim que o aviso de
/// diretorio nao confiavel chegou). Entao a leitura tenta o texto inteiro e,
/// falhando, recomeca na primeira chave.
fn achar_envelope(texto: &str) -> Option<RespostaCli> {
    let t = texto.trim();
    if t.is_empty() {
        return None;
    }
    serde_json::from_str(t)
        .ok()
        .or_else(|| serde_json::from_str(&t[t.find('{')?..]).ok())
}

/// Transforma o erro do CLI em algo que diz o que fazer.
///
/// Os codigos vieram do proprio binario 0.50.0 (`FATAL_AUTHENTICATION_ERROR`,
/// `FATAL_INPUT_ERROR`, `FATAL_CONFIG_ERROR`) e da tabela de saida documentada
/// em `docs/cli/headless.md`. Um numero cru na tela nao ajuda ninguem.
fn explicar(e: &ErroCli, stderr: &str) -> String {
    let detalhe = if e.message.trim().is_empty() {
        primeira_linha_util(stderr)
    } else {
        e.message.trim().to_string()
    };
    let cabeca = match e.code {
        Some(41) => crate::idioma::msg(
            "O Gemini CLI nao esta autenticado. Rode `gemini` no terminal e faca login.",
            "Gemini CLI is not authenticated. Run `gemini` in a terminal and sign in.",
        ),
        Some(42) => crate::idioma::msg(
            "O Gemini CLI recusou a entrada deste turno.",
            "Gemini CLI rejected this turn's input.",
        ),
        Some(52) => crate::idioma::msg(
            "A configuracao do Gemini CLI esta invalida.",
            "The Gemini CLI configuration is invalid.",
        ),
        _ => return detalhe,
    };
    format!("{cabeca} {detalhe}")
}

/// A primeira linha do stderr que nao seja rastro de pilha.
///
/// O CLI despeja o stack do Node no stderr, e mostrar
/// `at process.processTicksAndRejections` para quem so quer saber por que a
/// campanha parou nao explica nada.
fn primeira_linha_util(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("at ") && !l.starts_with('{'))
        .unwrap_or("")
        .chars()
        .take(240)
        .collect()
}

#[cfg(test)]
mod testes_modo {
    use super::*;
    use crate::prefs::ModoDesempenho::*;

    fn preco(m: &str) -> u8 {
        match m {
            FLASH_LITE => 1,
            FLASH => 2,
            PRO => 3,
            _ => 0,
        }
    }

    #[test]
    fn o_modo_muda_quem_assume_cada_cargo() {
        for tier in [Tier::Alto, Tier::Medio, Tier::Baixo] {
            let (e, m) = (
                modelo_do_nivel_com(tier, Economico),
                modelo_do_nivel_com(tier, Maximo),
            );
            assert!(
                preco(e) < preco(m),
                "{tier:?}: economico ({e}) devia custar menos que o maximo ({m})"
            );
        }
    }

    #[test]
    fn o_nivel_do_cargo_continua_mandando_dentro_de_cada_modo() {
        // A tese do produto e essa, e ela nao pode depender do provedor.
        for modo in [Economico, Normal, Maximo] {
            let alto = preco(modelo_do_nivel_com(Tier::Alto, modo));
            let baixo = preco(modelo_do_nivel_com(Tier::Baixo, modo));
            assert!(
                alto >= baixo,
                "{modo:?}: o cargo que decide nao pode receber menos que o que executa"
            );
        }
    }

    #[test]
    fn todo_modelo_devolvido_tem_rotulo() {
        for modo in [Economico, Normal, Maximo] {
            for tier in [Tier::Alto, Tier::Medio, Tier::Baixo] {
                let m = modelo_do_nivel_com(tier, modo);
                assert_ne!(rotulo_do_modelo(m), "Gemini", "sem rotulo para {m}");
            }
        }
    }
}

#[cfg(test)]
mod testes_envelope {
    use super::*;

    #[test]
    fn le_o_envelope_de_erro_real_do_cli() {
        // Capturado desta maquina, CLI 0.50.0, sem autenticacao configurada.
        let stderr = r#"{
  "session_id": "0c282124-2846-4c9c-99d2-93a0233759c7",
  "error": {
    "type": "Error",
    "message": "Please set an Auth method in your settings.json",
    "code": 41
  }
}"#;
        let env = achar_envelope(stderr).expect("o envelope de erro devia ser lido");
        let e = env.error.expect("devia ter erro");
        assert_eq!(e.code, Some(41));
        // O codigo 41 vira instrucao, nao numero cru.
        assert!(explicar(&e, stderr).contains("gemini"));
    }

    #[test]
    fn le_o_envelope_mesmo_com_aviso_em_texto_antes() {
        // Medido: o CLI escreve avisos em texto puro na mesma saida do JSON.
        // Um `from_str` no texto inteiro falharia e o turno viraria "resposta
        // vazia" — o pior tipo de erro, o que aponta para o lugar errado.
        let saida = "Gemini CLI is not running in a trusted directory.\n{\"response\":\"ok\"}";
        let env = achar_envelope(saida).expect("devia achar o JSON depois do aviso");
        assert_eq!(env.response.as_deref(), Some("ok"));
    }

    #[test]
    fn o_stack_do_node_nao_vira_mensagem_de_erro() {
        // Quem quer saber por que a campanha parou nao ganha nada com
        // "at process.processTicksAndRejections".
        let stderr = "\nError authenticating: IneligibleTierError: cliente sem suporte\n    at throwIneligible (file:///…)\n    at process.processTicksAndRejections (node:internal)";
        assert!(primeira_linha_util(stderr).starts_with("Error authenticating"));
    }
}
