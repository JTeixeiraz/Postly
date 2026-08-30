//! Claude Code como provedor de turno, no lugar do Ollama.
//!
//! Quem ja paga uma assinatura do Claude Code pode preferir usa-la a esperar
//! 1,2 tok/s de um modelo local. O contrato do cargo nao muda: entra system +
//! prompt, sai uma resposta de texto. O que muda e quem executa.
//!
//! A ideia central do produto sobrevive a troca. O nivel do cargo continua
//! proporcional ao que ele entrega, so que agora o eixo e outro:
//!
//!   alto  -> Opus     decide, le mercado, julga a peca
//!   medio -> Sonnet   audita dentro de criterio recebido
//!   baixo -> Haiku    cumpre briefing pronto
//!
//! Verificado contra o CLI real (2.1.251): `-p` com `--output-format json`
//! devolve `{"result": "...", "is_error": false, "total_cost_usd": 0.2}`.
//!
//! AUTENTICACAO. Isto executa o binario `claude` que ja esta na maquina, com a
//! sessao que a pessoa ja logou. Nao existe campo de chave de API em lugar
//! nenhum do Postly, e o cofre nunca guardou uma. Mas ha uma armadilha: o CLI
//! prefere `ANTHROPIC_API_KEY` quando ela existe no ambiente, e um processo
//! filho herda o ambiente do pai. Alguem com essa variavel exportada no shell
//! que abriu o app passaria a pagar por token sem perceber. Por isso ela e
//! removida do processo filho, junto das variaveis de credencial de terceiros
//! (Bedrock, Vertex). O turno roda pela assinatura, ou nao roda.

use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::orchestrator::roles::Tier;

/// Modelos por nivel. O nome do cargo nao muda; o executor, sim.
pub fn modelo_do_nivel(tier: Tier) -> &'static str {
    match tier {
        Tier::Alto => "claude-opus-5",
        Tier::Medio => "claude-sonnet-5",
        Tier::Baixo => "claude-haiku-4-5-20251001",
    }
}

/// Nome curto do modelo, para a tela.
pub fn rotulo_do_modelo(id: &str) -> &'static str {
    match id {
        "claude-opus-5" => "Opus 5",
        "claude-sonnet-5" => "Sonnet 5",
        "claude-haiku-4-5-20251001" => "Haiku 4.5",
        _ => "Claude",
    }
}

/// Ferramentas que um turno de marketing nao precisa.
///
/// Desligar reduz o system prompt que o CLI monta (medido: de 35k para 20k
/// tokens de cache por chamada) e, mais importante, impede que um agente saia
/// lendo ou escrevendo arquivos da maquina por conta propria. O cargo deve
/// escrever texto, nao mexer no disco.
const SEM_FERRAMENTAS: &str =
    "Bash,Read,Write,Edit,MultiEdit,NotebookEdit,Glob,Grep,WebFetch,WebSearch,Task,Agent,TodoWrite";

/// Variaveis que desviariam o turno da assinatura para uma cobranca por token.
const CREDENCIAIS_DE_FORA: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
];

/// Alguma credencial externa esta no ambiente deste processo?
///
/// Usado so para avisar na tela. O turno em si ja roda limpo: as variaveis sao
/// removidas do filho de qualquer jeito.
pub fn credencial_externa_no_ambiente() -> Option<String> {
    CREDENCIAIS_DE_FORA
        .iter()
        .find(|v| std::env::var_os(v).is_some_and(|x| !x.is_empty()))
        .map(|v| v.to_string())
}

#[derive(Debug, Deserialize)]
struct RespostaCli {
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    is_error: bool,
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default)]
    total_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnoClaude {
    pub texto: String,
    /// Custo em dolares do turno, como o proprio CLI reporta.
    pub custo_usd: f64,
}

/// Onde esta o `claude` desta maquina.
///
/// PROCURAR SO NO PATH NAO BASTA, e isto foi medido: o instalador oficial poe
/// o binario em `~/.local/bin`, que entra no PATH pelo perfil do shell
/// (`.zshrc`, `.profile`). Um aplicativo aberto pelo icone do menu recebe o
/// ambiente da sessao grafica, que nao le esses arquivos — e o PATH dele nao
/// tem `~/.local/bin`. O resultado seria a tela dizer "nao encontrado" para
/// quem tem o Claude Code instalado e funcionando, e so funcionar para quem
/// abrisse o app de um terminal.
///
/// Entao a busca tem tres degraus, do mais barato ao mais caro:
///
///   1. o PATH do processo             — resolve quando veio de um terminal
///   2. o shell de LOGIN da pessoa     — carrega o perfil, acha o resto
///   3. os caminhos conhecidos         — rede final, se o shell falhar
///
/// O sucesso e memorizado: um binario que apareceu nao some, e rodar um shell
/// de login a cada turno custaria mais que o turno. A falha nao e memorizada,
/// porque a pessoa pode instalar o Claude Code com o app aberto.
static ENCONTRADO: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

pub fn localizar() -> Option<std::path::PathBuf> {
    if let Some(p) = ENCONTRADO.get() {
        return Some(p.clone());
    }
    let achado = no_path()
        .or_else(perguntar_ao_shell)
        .or_else(nos_lugares_conhecidos)?;
    let _ = ENCONTRADO.set(achado.clone());
    Some(achado)
}

fn no_path() -> Option<std::path::PathBuf> {
    crate::platform::current().which("claude")
}

/// Pergunta ao shell de login onde esta o `claude`.
///
/// `-l` e o que importa: faz o shell ler o perfil do usuario, que e onde o
/// PATH completo e montado. Sem ele, o shell herda o mesmo ambiente pobre do
/// processo e a pergunta nao acrescenta nada.
///
/// Nao existe no Windows, onde nao ha shell de login com perfil equivalente —
/// la o degrau 3 faz o trabalho.
fn perguntar_ao_shell() -> Option<std::path::PathBuf> {
    if crate::platform::current().id() == crate::platform::Platform::Windows {
        return None;
    }
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let saida = std::process::Command::new(&shell)
        .args(["-lc", "command -v claude"])
        .output()
        .ok()?;
    if !saida.status.success() {
        return None;
    }
    let caminho = String::from_utf8_lossy(&saida.stdout).trim().to_string();
    let p = std::path::PathBuf::from(caminho);
    p.is_file().then_some(p)
}

/// Onde os instaladores costumam deixar o binario.
///
/// Vale como rede final: um shell configurado de forma incomum pode nao
/// responder, e o caminho continua existindo.
fn nos_lugares_conhecidos() -> Option<std::path::PathBuf> {
    let casa = dirs::home_dir()?;
    let candidatos: Vec<std::path::PathBuf> =
        if crate::platform::current().id() == crate::platform::Platform::Windows {
            vec![
                casa.join("AppData/Roaming/npm/claude.cmd"),
                casa.join("AppData/Local/Programs/claude/claude.exe"),
                casa.join(".local/bin/claude.exe"),
            ]
        } else {
            vec![
                casa.join(".local/bin/claude"),
                casa.join(".claude/local/claude"),
                casa.join(".bun/bin/claude"),
                casa.join(".npm-global/bin/claude"),
                std::path::PathBuf::from("/usr/local/bin/claude"),
                std::path::PathBuf::from("/opt/homebrew/bin/claude"),
            ]
        };
    candidatos.into_iter().find(|p| p.is_file())
}

/// Esta maquina tem o Claude Code instalado e autenticado?
pub fn disponivel() -> bool {
    localizar().is_some()
}

/// Versao do CLI, quando ele responde.
pub async fn versao() -> Option<String> {
    let saida = Command::new(localizar()?)
        .arg("--version")
        .output()
        .await
        .ok()?;
    if !saida.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

/// Roda um turno. O prompt vai pelo stdin: um briefing tem milhares de
/// caracteres e passa do limite de argumento em algumas plataformas.
pub async fn turno(
    tier: Tier,
    system: &str,
    prompt: &str,
    timeout_s: u64,
) -> Result<TurnoClaude, String> {
    let Some(binario) = localizar() else {
        return Err(crate::idioma::msg(
            "Claude Code nao encontrado nesta maquina. Instale em claude.com/code ou volte para o Ollama.",
            "Claude Code was not found on this machine. Install it from claude.com/code or switch back to Ollama.",
        ));
    };

    let mut filho = Command::new(binario);
    for v in CREDENCIAIS_DE_FORA {
        filho.env_remove(v);
    }
    let mut filho = filho
        .args([
            "-p",
            "--output-format",
            "json",
            "--model",
            modelo_do_nivel(tier),
            "--system-prompt",
            system,
            "--disallowed-tools",
            SEM_FERRAMENTAS,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("falha ao iniciar o Claude Code: {e}"))?;

    if let Some(mut entrada) = filho.stdin.take() {
        entrada
            .write_all(prompt.as_bytes())
            .await
            .map_err(|e| format!("falha ao enviar o prompt: {e}"))?;
        entrada.shutdown().await.ok();
    }

    let saida = tokio::time::timeout(
        std::time::Duration::from_secs(timeout_s),
        filho.wait_with_output(),
    )
    .await
    .map_err(|_| {
        crate::idioma::msg(
            "O Claude Code passou do tempo limite deste turno.",
            "Claude Code exceeded this turn's time limit.",
        )
    })?
    .map_err(|e| format!("falha ao ler a resposta: {e}"))?;

    let bruto = String::from_utf8_lossy(&saida.stdout);
    if bruto.trim().is_empty() {
        let erro = String::from_utf8_lossy(&saida.stderr);
        return Err(format!(
            "{} {}",
            crate::idioma::msg(
                "O Claude Code nao devolveu nada.",
                "Claude Code returned nothing."
            ),
            erro.lines().next().unwrap_or("").trim()
        ));
    }

    let resposta: RespostaCli = serde_json::from_str(bruto.trim()).map_err(|e| {
        format!(
            "resposta ilegivel do Claude Code: {e} :: {}",
            bruto.chars().take(200).collect::<String>()
        )
    })?;

    if resposta.is_error {
        return Err(format!(
            "{} ({})",
            crate::idioma::msg(
                "O Claude Code recusou o turno",
                "Claude Code refused the turn"
            ),
            resposta.subtype.unwrap_or_else(|| "sem detalhe".into())
        ));
    }

    let texto = resposta.result.unwrap_or_default();
    if texto.trim().is_empty() {
        return Err(crate::idioma::msg(
            "O Claude Code devolveu uma resposta vazia.",
            "Claude Code returned an empty response.",
        ));
    }

    Ok(TurnoClaude {
        texto,
        custo_usd: resposta.total_cost_usd.unwrap_or(0.0),
    })
}
