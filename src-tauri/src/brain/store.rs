//! Persistencia do cerebro: serializa em bytes, compacta, grava. Nunca texto puro.
//!
//! Ciclo, exatamente nesta ordem:
//!   grafo em memoria -> bincode (bytes) -> zstd (compactado) -> arquivo temporario
//!   -> rename atomico sobre o artefato.
//!
//! Em runtime o artefato e descompactado e desserializado para uma variavel em
//! memoria. O arquivo em disco permanece compactado o tempo todo, e a escrita
//! e serializada por um mutex: duas mutacoes concorrentes nunca se sobrescrevem.

use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use tokio::sync::{Mutex, RwLock};

use super::Graph;
use crate::platform;

/// Compressao alta: o artefato e pequeno e escrito com pouca frequencia, entao
/// vale trocar CPU por footprint.
const ZSTD_LEVEL: i32 = 19;
/// Cabecalho para detectar artefato de outra versao / arquivo corrompido.
const MAGIC: &[u8; 4] = b"AGB1";
/// Quantos snapshots rotativos manter, para encurtar a janela de perda.
const SNAPSHOT_KEEP: usize = 5;

pub fn brain_dir() -> PathBuf {
    platform::current().data_dir().join("brain")
}

pub fn artifact_path() -> PathBuf {
    brain_dir().join("brain.graph.zst")
}

fn snapshot_dir() -> PathBuf {
    brain_dir().join("snapshots")
}

/// Grafo vivo em memoria + trava de escrita em disco.
pub struct BrainHandle {
    graph: RwLock<Graph>,
    disk_lock: Mutex<()>,
}

impl BrainHandle {
    /// Carrega o artefato do disco, ou nasce com o cerebro semente.
    pub fn load() -> Self {
        let graph = read_artifact(&artifact_path()).unwrap_or_else(|_| seed_graph());
        Self {
            graph: RwLock::new(graph),
            disk_lock: Mutex::new(()),
        }
    }

    pub async fn read<T>(&self, f: impl FnOnce(&Graph) -> T) -> T {
        let guard = self.graph.read().await;
        f(&guard)
    }

    /// Muta o grafo em memoria e persiste o artefato compactado logo em seguida.
    pub async fn write<T>(&self, f: impl FnOnce(&mut Graph) -> T) -> Result<T, String> {
        let result = {
            let mut guard = self.graph.write().await;
            f(&mut guard)
        };
        self.flush().await?;
        Ok(result)
    }

    /// Serializa, compacta e substitui o artefato. Uma escrita por vez.
    pub async fn flush(&self) -> Result<(), String> {
        let _disk = self.disk_lock.lock().await;
        let bytes = {
            let guard = self.graph.read().await;
            encode(&guard)?
        };
        write_atomic(&artifact_path(), &bytes)?;
        rotate_snapshot(&bytes)?;
        Ok(())
    }

    pub async fn stats(&self) -> BrainStats {
        let guard = self.graph.read().await;
        let artifact = artifact_path();
        let compressed = std::fs::metadata(&artifact).map(|m| m.len()).unwrap_or(0);
        let raw = encode_raw(&guard).map(|b| b.len() as u64).unwrap_or(0);
        BrainStats {
            nodes: guard.node_count(),
            edges: guard.edge_count(),
            raw_bytes: raw,
            compressed_bytes: compressed,
            ratio: if raw > 0 {
                compressed as f32 / raw as f32
            } else {
                0.0
            },
            path: artifact.to_string_lossy().to_string(),
            updated_at: guard.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BrainStats {
    pub nodes: usize,
    pub edges: usize,
    /// Tamanho em bytes ANTES da compactacao.
    pub raw_bytes: u64,
    /// Tamanho do artefato em disco, ja compactado.
    pub compressed_bytes: u64,
    pub ratio: f32,
    pub path: String,
    pub updated_at: i64,
}

fn encode_raw(graph: &Graph) -> Result<Vec<u8>, String> {
    bincode::serialize(graph).map_err(|e| format!("falha ao serializar o cerebro: {e}"))
}

/// bincode -> zstd, com cabecalho de identificacao.
fn encode(graph: &Graph) -> Result<Vec<u8>, String> {
    let raw = encode_raw(graph)?;
    let compressed = zstd::encode_all(&raw[..], ZSTD_LEVEL)
        .map_err(|e| format!("falha ao compactar o cerebro: {e}"))?;
    let mut out = Vec::with_capacity(compressed.len() + 4);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&compressed);
    Ok(out)
}

/// zstd -> bincode. Unico caminho de leitura do artefato.
fn decode(bytes: &[u8]) -> Result<Graph, String> {
    if bytes.len() < 4 || &bytes[..4] != MAGIC {
        return Err("artefato do cerebro invalido ou de outra versao".into());
    }
    let raw = zstd::decode_all(&bytes[4..])
        .map_err(|e| format!("falha ao descompactar o cerebro: {e}"))?;
    bincode::deserialize(&raw).map_err(|e| format!("falha ao desserializar o cerebro: {e}"))
}

fn read_artifact(path: &Path) -> Result<Graph, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    decode(&bytes)
}

/// Grava em `.tmp` e renomeia. O rename e atomico no mesmo sistema de arquivos,
/// entao nunca existe um artefato pela metade.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("nao consegui criar {parent:?}: {e}"))?;
    }
    let tmp = path.with_extension("zst.tmp");
    {
        let mut file = std::fs::File::create(&tmp).map_err(|e| e.to_string())?;
        file.write_all(bytes).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
    }
    std::fs::rename(&tmp, path).map_err(|e| format!("falha ao substituir o artefato: {e}"))
}

/// Snapshots rotativos: o artefato e um estado vivo, e um estado vivo sem
/// historico e um estado que voce perde uma vez so.
fn rotate_snapshot(bytes: &[u8]) -> Result<(), String> {
    let dir = snapshot_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S");
    std::fs::write(dir.join(format!("brain-{stamp}.zst")), bytes).map_err(|e| e.to_string())?;

    let mut snaps: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "zst"))
        .collect();
    snaps.sort();
    while snaps.len() > SNAPSHOT_KEEP {
        let oldest = snaps.remove(0);
        let _ = std::fs::remove_file(oldest);
    }
    Ok(())
}

/// Cerebro semente: o vocabulario minimo do dominio, para o primeiro agente
/// nao acordar diante de um grafo vazio.
fn seed_graph() -> Graph {
    let mut g = Graph::new();

    g.upsert_node(
        "postly",
        "sistema",
        "Middleware local que orquestra agentes de IA como funcionarios de marketing.",
    );
    g.upsert_node(
        "instagram",
        "rede_social",
        "Feed visual. Imagem 1:1 ou 4:5, legenda ate 2200 caracteres, hashtags relevantes.",
    );
    g.upsert_node(
        "facebook",
        "rede_social",
        "Alcance amplo e faixa etaria mais alta. Texto conversacional, imagem horizontal.",
    );
    g.upsert_node(
        "tiktok",
        "rede_social",
        "Video vertical 9:16. O gancho vive nos dois primeiros segundos.",
    );
    g.upsert_node(
        "linkedin",
        "rede_social",
        "Publico profissional. A primeira linha precisa segurar antes do corte de 'ver mais'.",
    );
    g.upsert_node(
        "x",
        "rede_social",
        "Texto curto ate 280 caracteres, imagem 16:9, ritmo de conversa.",
    );

    g.upsert_node("gerente_setor", "cargo", "Decide a linha criativa a partir de analise de mercado e concorrencia. So envia mensagem ao criador.");
    g.upsert_node(
        "criador_conteudo",
        "cargo",
        "Executa o briefing do gerente: gera prompt de imagem e legenda. Nao decide estrategia.",
    );
    g.upsert_node(
        "auditor",
        "cargo",
        "Verifica alucinacao e aderencia ao briefing, e decide junto com o gerente.",
    );
    g.upsert_node(
        "diretor_geral",
        "cargo",
        "Existe quando ha mais de uma rede. Distribui a estrategia macro para cada gerente.",
    );

    g.upsert_node(
        "prova_social",
        "tatica",
        "Depoimento, numero de clientes ou resultado concreto reduzem atrito de compra.",
    );
    g.upsert_node(
        "chamada_para_acao",
        "tatica",
        "Toda publicacao comercial termina com uma acao unica e explicita.",
    );
    g.upsert_node(
        "consistencia_visual",
        "tatica",
        "Paleta e tipografia estaveis fazem a marca ser reconhecida antes de ser lida.",
    );

    g.upsert_edge("diretor_geral", "gerente_setor", "delega_para", 0.95);
    g.upsert_edge("gerente_setor", "criador_conteudo", "delega_para", 0.95);
    g.upsert_edge("criador_conteudo", "auditor", "entrega_para", 0.90);
    g.upsert_edge("auditor", "gerente_setor", "valida_com", 0.90);
    g.upsert_edge("gerente_setor", "prova_social", "aplica", 0.72);
    g.upsert_edge("gerente_setor", "chamada_para_acao", "aplica", 0.80);
    g.upsert_edge("criador_conteudo", "consistencia_visual", "aplica", 0.75);
    g.upsert_edge("instagram", "consistencia_visual", "exige", 0.70);
    g.upsert_edge("tiktok", "chamada_para_acao", "exige", 0.62);
    g.upsert_edge("linkedin", "prova_social", "exige", 0.68);
    g.upsert_edge("postly", "gerente_setor", "opera", 0.88);

    g
}
