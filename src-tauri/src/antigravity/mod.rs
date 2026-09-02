//! Antigravity CLI como terceiro provedor de turno, ao lado do Ollama e do
//! Claude Code.
//!
//! Este modulo substituiu o do Gemini CLI. O Google anunciou a transicao do
//! `gemini` para o `agy` (Antigravity CLI, escrito em Go), e o caminho antigo
//! deixou de funcionar para conta pessoal: o binario 0.50.0 recusa o tier
//! gratuito com `IneligibleTierError · UNSUPPORTED_CLIENT`, medido nesta
//! maquina tanto em modo headless quanto num terminal de verdade.
//!
//! A tese sobrevive de novo, e agora com uma folga que os outros provedores nao
//! tem: os proprios IDs de modelo do `agy` ja embutem o nivel de esforco, entao
//! o nivel do cargo vira o nome do modelo sem precisar de segundo parametro.
//!
//!   alto  -> Gemini 3.1 Pro (High)     decide, le mercado, julga a peca
//!   medio -> Gemini 3.7 Flash (High)   audita dentro de criterio recebido
//!   baixo -> Gemini 3.7 Flash (Low)    cumpre briefing pronto
//!
//! CONTRATO MEDIDO CONTRA O `agy` 1.1.23 NESTA MAQUINA. Quatro achados, e os
//! quatro mudaram o desenho:
//!
//! 1. NAO EXISTE SUBSTITUICAO DE SYSTEM PROMPT. O `gemini` tinha
//!    `GEMINI_SYSTEM_MD`; aqui nao ha equivalente. Testados e descartados:
//!    `AGENTS.md` e `GEMINI.md` no diretorio de trabalho — o binario documenta
//!    os dois como mecanismo de regras, mas em print mode nenhum e aplicado
//!    (o modelo respondeu "4" a `2+2` com um arquivo mandando responder
//!    sempre "PAPAGAIO", e a contagem de tokens de entrada nao mudou).
//!    Entao a doutrina do cargo vai NO PROPRIO PROMPT — medido, e ai o modelo
//!    obedece.
//!
//! 2. O CODIGO DE SAIDA NAO E A VERDADE. Medido: um turno em que uma ferramenta
//!    pede permissao volta com `exit 0`, resposta vazia e
//!    `"status":"CANCELED"`. Confiar no codigo de saida faria isso passar por
//!    sucesso e o orquestrador seguir com uma peca vazia. Quem manda e o campo
//!    `status`.
//!
//! 3. O PROMPT VAI COMO ARGUMENTO, e nao por stdin. Medido: `agy -p ""` com o
//!    texto no stdin devolve
//!    `{"status":"ERROR","error":"Error: empty prompt..."}`. E o inverso do
//!    Claude Code, onde o stdin existe justamente para fugir do limite de
//!    argumento — aqui nao ha escolha.
//!
//! 4. NAO HA GATE DE DIRETORIO CONFIAVEL. O `gemini` recusava rodar fora de uma
//!    pasta aprovada e precisava de `--skip-trust`; o `agy` roda direto.

use serde::Serialize;
use tokio::process::Command;

pub mod ambiente;
pub mod limite;
mod resposta;

pub use ambiente::{credencial_externa_no_ambiente, disponivel, localizar, versao};

use crate::orchestrator::roles::Tier;

/// Os modelos vieram de `agy models` nesta maquina, com conta Google AI Pro.
///
/// O ID JA CARREGA O ESFORCO (`-high`, `-low`), entao a flag `--effort` fica de
/// fora: dois lugares dizendo a mesma coisa e um convite a discordarem.
///
/// A LISTA E DA CONTA, e nao do binario. Uma conta com outro plano pode nao ter
/// o Pro, e ai o turno falha na hora de rodar com o modelo inexistente. E o
/// mesmo risco que o catalogo do Ollama corre quando uma tag some da
/// biblioteca — e, como la, a falha e visivel e diz o nome do modelo.
const PRO: &str = "gemini-3.1-pro-high";
const FLASH: &str = "gemini-3.7-flash-high";
const FLASH_BAIXO: &str = "gemini-3.7-flash-low";

pub fn modelo_do_nivel(tier: Tier) -> &'static str {
    modelo_do_nivel_com(tier, crate::prefs::load().modo)
}

/// O modo de desempenho vale aqui pelo mesmo motivo que vale nos outros: a
/// intencao de quem escolhe e a mesma, e um seletor que funcionasse num
/// provedor e nao no outro seria confuso. O eixo, como no Claude Code, e custo.
pub fn modelo_do_nivel_com(tier: Tier, modo: crate::prefs::ModoDesempenho) -> &'static str {
    use crate::prefs::ModoDesempenho as M;
    match (modo, tier) {
        (M::Economico, Tier::Alto) => FLASH,
        (M::Economico, _) => FLASH_BAIXO,

        (M::Normal, Tier::Alto) => PRO,
        (M::Normal, Tier::Medio) => FLASH,
        (M::Normal, Tier::Baixo) => FLASH_BAIXO,

        // No maximo o auditor sobe junto de quem decide: julgar a peca e a
        // segunda decisao mais cara da campanha.
        (M::Maximo, Tier::Baixo) => FLASH,
        (M::Maximo, _) => PRO,
    }
}

pub fn rotulo_do_modelo(id: &str) -> &'static str {
    match id {
        PRO => "Gemini 3.1 Pro",
        FLASH => "Gemini 3.7 Flash",
        FLASH_BAIXO => "Gemini 3.7 Flash (baixo)",
        _ => "Antigravity",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnoAntigravity {
    pub texto: String,
    /// Tokens do turno, como o CLI reporta.
    ///
    /// Nao vira dinheiro na trilha: o `agy` cobra por assinatura e nao informa
    /// preco. Mostrar um valor calculado por fora seria inventar.
    pub tokens_entrada: u64,
    pub tokens_saida: u64,
}

/// Por que o turno nao saiu.
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
                    "A cota do Antigravity acabou.",
                    "The Antigravity quota ran out."
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

/// A instrucao que evita a chamada de ferramenta antes dela acontecer.
///
/// O system prompt embutido do `agy` e de um agente de codigo, e nao ha como
/// substitui-lo. Sem esta linha, o modelo as vezes decide ler um arquivo no
/// meio de um turno que so precisa de texto — e cada tentativa custa tempo e
/// cota. As flags de permissao sao a rede embaixo; isto e o que impede a queda.
const SEM_FERRAMENTAS: &str = "\
NAO USE FERRAMENTA NENHUMA. Nao leia arquivo, nao rode comando, nao busque na \
web, nao liste diretorio. Tudo de que voce precisa esta escrito neste texto, e \
qualquer caminho de arquivo citado aqui e um NOME para voce referenciar na \
resposta, nao algo para abrir. Responda direto.";

/// Roda um turno.
pub async fn turno(
    tier: Tier,
    system: &str,
    prompt: &str,
    timeout_s: u64,
) -> Result<TurnoAntigravity, ErroTurno> {
    let Some(binario) = ambiente::localizar() else {
        return Err(ErroTurno::Outro(crate::idioma::msg(
            "Antigravity CLI nao encontrado nesta maquina. Instale em antigravity.google ou volte para o Ollama.",
            "Antigravity CLI was not found on this machine. Install it from antigravity.google or switch back to Ollama.",
        )));
    };

    let dir = ambiente::pasta_de_trabalho().map_err(ErroTurno::Outro)?;

    // A doutrina do cargo entra no PROPRIO prompt, porque o `agy` nao tem
    // substituicao de system — ver o achado 1 no topo do modulo. Os dois blocos
    // ficam separados por uma linha em branco e nesta ordem: o cargo primeiro,
    // a tarefa depois, que e como o modelo le "quem sou" antes de "o que faco".
    let texto = format!(
        "{}\n\n{}\n\n{}",
        system.trim(),
        SEM_FERRAMENTAS,
        prompt.trim()
    );

    let saida = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_s),
        Command::new(binario)
            .current_dir(&dir)
            .args([
                "--print",
                &texto,
                "--output-format",
                "json",
                "--model",
                modelo_do_nivel(tier),
                // Sem isto, um "/" no briefing viraria comando do CLI. Um texto
                // de marketing tem barra o tempo todo — "20/30 reais", datas —
                // e transformar conteudo de quem usa em comando e a mesma
                // classe de erro que o detector de cota evita ao olhar so o fim
                // da saida.
                "--disable-slash-commands",
                // FERRAMENTA E O PROBLEMA CENTRAL DESTE PROVEDOR, e a solucao
                // tem tres partes porque nenhuma sozinha basta.
                //
                // O `agy` nao tem como desligar ferramenta por flag, e o system
                // prompt embutido dele e de um agente de codigo — entao o
                // modelo as vezes decide ler um arquivo no meio de um turno de
                // marketing. Medido em producao: o Motion Designer pediu
                // `read_file` e o turno voltou `CANCELED` com resposta vazia,
                // depois de 95 segundos de dois turnos pagos.
                //
                // 1. `--mode plan` deixa o agente SOMENTE LEITURA. Ele nao
                //    escreve no disco de ninguem, que e a regra dos outros
                //    provedores.
                "--mode",
                "plan",
                // 2. `--dangerously-skip-permissions` impede que uma tentativa
                //    de ferramenta MATE o turno. O nome assusta e merece: sem
                //    ele, qualquer chamada vira `CANCELED`; com ele, dentro do
                //    modo plan, o pior que acontece e o agente ler um arquivo.
                //    Medido: `plan` sozinho cancela, `plan + skip` conclui.
                //
                //    A alternativa seria `toolPermission: "deny"` num settings
                //    proprio — testada, e ela apenas troca o cancelamento de
                //    lugar: o CLI nega e o turno morre igual.
                //
                // 3. A terceira parte esta no prompt: o cargo e instruido a nao
                //    usar ferramenta nenhuma. Ver `SEM_FERRAMENTAS`.
                "--dangerously-skip-permissions",
                // O teto do CLI e 5 minutos por padrao, e um briefing longo em
                // Pro passa disso. Fica um pouco abaixo do nosso proprio teto
                // para o CLI encerrar primeiro e devolver JSON, em vez de a
                // gente matar o processo e nao ter o que ler.
                "--print-timeout",
                &format!("{}s", timeout_s.saturating_sub(15).max(30)),
            ])
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| {
        ErroTurno::Outro(crate::idioma::msg(
            "O Antigravity passou do tempo limite deste turno.",
            "Antigravity exceeded this turn's time limit.",
        ))
    })?
    .map_err(|e| ErroTurno::Outro(format!("falha ao iniciar o Antigravity CLI: {e}")))?;

    let bruto = String::from_utf8_lossy(&saida.stdout);
    let erro_bruto = String::from_utf8_lossy(&saida.stderr);

    let junto = format!("{bruto}\n{erro_bruto}");
    if let Some(l) = limite::detectar(&junto) {
        return Err(ErroTurno::Limite(l));
    }
    if resposta::precisa_de_login(&junto) {
        return Err(ErroTurno::Outro(crate::idioma::msg(
            "O Antigravity CLI nao esta autenticado. Rode `agy` num terminal e faca login — \
             o Postly nao consegue autenticar por voce.",
            "Antigravity CLI is not signed in. Run `agy` in a terminal and sign in — \
             Postly cannot authenticate on your behalf.",
        )));
    }

    let Some(envelope) =
        resposta::achar_envelope(&bruto).or_else(|| resposta::achar_envelope(&erro_bruto))
    else {
        return Err(ErroTurno::Outro(format!(
            "{} {}",
            crate::idioma::msg(
                "O Antigravity nao devolveu resposta.",
                "Antigravity returned no response."
            ),
            resposta::motivo(&erro_bruto)
        )));
    };

    // O STATUS MANDA, E NAO O CODIGO DE SAIDA — ver o achado 2 no topo.
    if envelope.status != "SUCCESS" {
        return Err(ErroTurno::Outro(resposta::explicar(&envelope, &erro_bruto)));
    }

    let texto = envelope
        .response
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .ok_or_else(|| {
            ErroTurno::Outro(crate::idioma::msg(
                "O Antigravity devolveu uma resposta vazia.",
                "Antigravity returned an empty response.",
            ))
        })?;

    let uso = envelope.usage.unwrap_or_default();
    Ok(TurnoAntigravity {
        texto,
        tokens_entrada: uso.input_tokens,
        tokens_saida: uso.output_tokens,
    })
}

#[cfg(test)]
mod testes_modo {
    use super::*;
    use crate::prefs::ModoDesempenho::*;

    fn preco(m: &str) -> u8 {
        match m {
            FLASH_BAIXO => 1,
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
        // A tese do produto, e ela nao pode depender do provedor.
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
                assert_ne!(rotulo_do_modelo(m), "Antigravity", "sem rotulo para {m}");
            }
        }
    }

    #[test]
    fn o_turno_manda_o_cargo_nao_usar_ferramenta() {
        // Medido em producao: sem esta instrucao, o Motion Designer pediu
        // `read_file` no meio de um turno que so precisava de texto, e o `agy`
        // devolveu CANCELED com resposta vazia. As flags de permissao sao a
        // rede embaixo; a instrucao e o que impede a queda.
        assert!(SEM_FERRAMENTAS.contains("NAO USE FERRAMENTA"));
        assert!(
            SEM_FERRAMENTAS.contains("nao leia arquivo")
                || SEM_FERRAMENTAS.contains("Nao leia arquivo")
        );
    }

    #[test]
    fn todo_id_existe_na_lista_da_conta_medida() {
        // Capturada de `agy models` nesta maquina. Se alguem trocar um ID por
        // um que nao existe, o turno so falharia no meio da campanha — este
        // teste falha antes, na maquina de quem escreveu.
        const DA_CONTA: &[&str] = &[
            "gemini-3.7-flash-high",
            "gemini-3.7-flash-medium",
            "gemini-3.7-flash-low",
            "gemini-3.6-flash-high",
            "gemini-3.6-flash-medium",
            "gemini-3.6-flash-low",
            "gemini-3.1-pro-high",
            "gemini-3.1-pro-low",
        ];
        for m in [PRO, FLASH, FLASH_BAIXO] {
            assert!(DA_CONTA.contains(&m), "{m} nao apareceu em `agy models`");
        }
    }
}
