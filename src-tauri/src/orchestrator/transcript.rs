//! Gravacao das conversas em Markdown.
//!
//! A conversa INTEIRA de cada agente (system prompt, entrada e saida completas)
//! fica no disco. Para o proximo agente segue apenas a mensagem final. E essa
//! separacao que permite fechar a sessao do modelo sem perder rastreabilidade:
//! o historico existe no arquivo, nao na janela de contexto.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::roles::{Network, Role};
use crate::platform;

pub fn runs_dir() -> PathBuf {
    platform::current().data_dir().join("runs")
}

#[derive(Debug, Clone, Serialize)]
pub struct RunPaths {
    pub id: String,
    pub dir: String,
    pub index: String,
    pub media_dir: String,
}

/// Cria a pasta da campanha e o indice inicial.
pub fn start_run(objetivo: &str, redes: &[Network]) -> Result<RunPaths, String> {
    let id = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let dir = runs_dir().join(&id);
    let media = dir.join("midia");
    std::fs::create_dir_all(&media).map_err(|e| format!("falha ao criar {dir:?}: {e}"))?;

    // Um video avulso nao tem rede. Sem esta distincao o cabecalho diria
    // "Redes:" seguido de nada, e quem abrisse a pasta meses depois nao saberia
    // se foi uma campanha que falhou antes de escolher rede ou outra coisa.
    let (titulo, linha_redes) = if redes.is_empty() {
        (
            "Video",
            String::from(
                "- **Tipo:** video avulso, sem publicacao
",
            ),
        )
    } else {
        (
            "Campanha",
            format!(
                "- **Redes:** {}
",
                redes
                    .iter()
                    .map(|r| r.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )
    };

    let index = dir.join("campanha.md");
    let header = format!(
        "# {titulo} {id}\n\n\
         - **Iniciada em:** {}\n\
         {linha_redes}\n\
         ## Objetivo do usuario\n\n{}\n\n\
         ---\n\n## Linha do tempo\n\n",
        chrono::Local::now().format("%d/%m/%Y %H:%M:%S"),
        objetivo.trim()
    );
    std::fs::write(&index, header).map_err(|e| e.to_string())?;

    Ok(RunPaths {
        id,
        dir: dir.to_string_lossy().to_string(),
        index: index.to_string_lossy().to_string(),
        media_dir: media.to_string_lossy().to_string(),
    })
}

pub struct TurnRecord<'a> {
    pub step: usize,
    pub role: Role,
    pub network: Option<Network>,
    pub model: &'a str,
    pub system: &'a str,
    pub prompt: &'a str,
    pub output: &'a str,
    /// Raciocinio interno do modelo. Fica no arquivo e nao atravessa.
    pub thinking: &'a str,
    pub handoff: &'a str,
    pub elapsed_ms: u128,
    pub tokens_per_second: f32,
    pub ram_budget_bytes: u64,
    pub degraded: Option<String>,
}

/// Grava a conversa completa de um turno e devolve o caminho do arquivo.
pub fn write_turn(run: &RunPaths, record: &TurnRecord) -> Result<String, String> {
    let scope = record
        .network
        .map(|n| format!("-{}", n.slug()))
        .unwrap_or_default();
    let filename = format!("{:02}-{}{}.md", record.step, record.role.slug(), scope);
    let path = PathBuf::from(&run.dir).join(&filename);

    let mut body = String::new();
    body.push_str(&format!("# {} ", record.role.label()));
    if let Some(net) = record.network {
        body.push_str(&format!("— {} ", net.label()));
    }
    body.push_str(&format!("(passo {})\n\n", record.step));

    body.push_str("## Execucao\n\n");
    body.push_str(&format!("- **Modelo:** `{}`\n", record.model));
    body.push_str(&format!("- **Nivel do cargo:** {:?}\n", record.role.tier()));
    body.push_str(&format!(
        "- **Orcamento de RAM na hora de subir:** {}\n",
        crate::hardware::human(record.ram_budget_bytes)
    ));
    body.push_str(&format!(
        "- **Duracao:** {:.1}s\n",
        record.elapsed_ms as f64 / 1000.0
    ));
    body.push_str(&format!(
        "- **Velocidade:** {:.1} tokens/s\n",
        record.tokens_per_second
    ));
    body.push_str(&format!(
        "- **Quem pode receber esta mensagem:** {}\n",
        record
            .role
            .may_send_to()
            .iter()
            .map(|r| r.label())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    if let Some(warn) = &record.degraded {
        body.push_str(&format!("- **Aviso:** {warn}\n"));
    }

    body.push_str("\n## System prompt\n\n```\n");
    body.push_str(record.system.trim());
    body.push_str("\n```\n\n## Entrada recebida\n\n```\n");
    body.push_str(record.prompt.trim());
    body.push_str("\n```\n\n## Resposta completa do modelo\n\n");
    body.push_str(record.output.trim());
    body.push_str("\n\n## Mensagem repassada adiante\n\n> ");
    body.push_str(&record.handoff.trim().replace('\n', "\n> "));
    body.push('\n');

    std::fs::write(&path, body).map_err(|e| format!("falha ao gravar {path:?}: {e}"))?;

    append_index(
        run,
        &format!(
            "{}. **{}**{} — `{}` — {:.1}s — [{}]({})\n",
            record.step,
            record.role.label(),
            record
                .network
                .map(|n| format!(" ({})", n.label()))
                .unwrap_or_default(),
            record.model,
            record.elapsed_ms as f64 / 1000.0,
            filename,
            filename
        ),
    )?;

    Ok(path.to_string_lossy().to_string())
}

pub fn append_index(run: &RunPaths, line: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&run.index)
        .map_err(|e| e.to_string())?;
    file.write_all(line.as_bytes()).map_err(|e| e.to_string())
}

/// Lista as campanhas ja executadas, mais recente primeiro.
#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub id: String,
    pub dir: String,
    pub index: String,
    pub turns: usize,
    /// O que a pessoa pediu. Sem isto a lista e uma coluna de datas: ninguem
    /// procura uma campanha pelo horario em que ela rodou.
    #[serde(default)]
    pub objetivo: String,
    #[serde(default)]
    pub redes: Vec<String>,
    #[serde(default)]
    pub pecas: usize,
    #[serde(default)]
    pub publicadas: usize,
    #[serde(default)]
    pub simulado: bool,
    #[serde(default)]
    pub aprovado: bool,
}

/// O que a campanha produziu, gravado em disco ao lado da transcricao.
///
/// A transcricao guarda a CONVERSA; isto guarda o RESULTADO. Sem este arquivo
/// a legenda final, as hashtags e o caminho da arte so existiam na memoria da
/// janela: fechar o app apagava a peca, e o unico rastro sobrava enterrado no
/// meio da resposta do modelo no markdown do turno do Criador.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub id: String,
    pub objetivo: String,
    pub redes: Vec<String>,
    pub aprovado: bool,
    pub rodadas: u8,
    pub simulado: bool,
    pub parecer_auditor: String,
    #[serde(default)]
    pub avisos: Vec<String>,
    pub pecas: Vec<crate::orchestrator::PecaFinal>,
    pub encerrada_em: String,
}

fn caminho_do_resultado(dir: &str) -> PathBuf {
    PathBuf::from(dir).join("pecas.json")
}

pub fn write_result(run: &RunPaths, r: &RunResult) -> Result<(), String> {
    let texto = serde_json::to_string_pretty(r).map_err(|e| e.to_string())?;
    std::fs::write(caminho_do_resultado(&run.dir), texto)
        .map_err(|e| format!("falha ao gravar o resultado da campanha: {e}"))
}

/// Le o resultado de uma execucao. `None` quando a campanha e anterior a este
/// arquivo existir, ou foi interrompida antes do fim.
pub fn read_result(dir: &str) -> Option<RunResult> {
    let texto = std::fs::read_to_string(caminho_do_resultado(dir)).ok()?;
    serde_json::from_str(&texto).ok()
}

pub fn list_runs() -> Vec<RunSummary> {
    let Ok(entries) = std::fs::read_dir(runs_dir()) else {
        return Vec::new();
    };
    let mut runs: Vec<RunSummary> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| {
            let dir = e.path();
            let turns = std::fs::read_dir(&dir)
                .map(|d| {
                    d.flatten()
                        .filter(|f| {
                            f.path().extension().is_some_and(|x| x == "md")
                                && f.file_name() != "campanha.md"
                        })
                        .count()
                })
                .unwrap_or(0);
            let caminho = dir.to_string_lossy().to_string();
            // O resumo vem do resultado quando ele existe. Campanhas anteriores
            // a este arquivo, e as interrompidas no meio, continuam listadas
            // com o que da para saber pela pasta.
            let resultado = read_result(&caminho);
            RunSummary {
                id: e.file_name().to_string_lossy().to_string(),
                index: dir.join("campanha.md").to_string_lossy().to_string(),
                dir: caminho,
                turns,
                objetivo: resultado
                    .as_ref()
                    .map(|r| r.objetivo.clone())
                    .unwrap_or_default(),
                redes: resultado
                    .as_ref()
                    .map(|r| r.redes.clone())
                    .unwrap_or_default(),
                pecas: resultado.as_ref().map(|r| r.pecas.len()).unwrap_or(0),
                publicadas: resultado
                    .as_ref()
                    .map(|r| r.pecas.iter().filter(|p| p.publicado).count())
                    .unwrap_or(0),
                simulado: resultado.as_ref().is_some_and(|r| r.simulado),
                aprovado: resultado.as_ref().is_some_and(|r| r.aprovado),
            }
        })
        .collect();
    runs.sort_by(|a, b| b.id.cmp(&a.id));
    runs
}
