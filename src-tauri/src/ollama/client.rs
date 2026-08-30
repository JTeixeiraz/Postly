//! Cliente HTTP do Ollama local.
//!
//! O ponto central do middleware esta em `generate`: toda chamada usa
//! `keep_alive: 0`, o que faz o Ollama descarregar o modelo da memoria assim
//! que a resposta termina. E isso que materializa a regra de "abre a sessao,
//! pega a resposta, fecha a sessao" sem deixar dois modelos residentes.

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const BASE_URL: &str = "http://127.0.0.1:11434";

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        // Inferencia em CPU e lenta; um gerente de 27B pode levar minutos.
        .timeout(Duration::from_secs(60 * 30))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .expect("cliente http")
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    version: String,
}

/// O servidor esta no ar? Devolve a versao quando sim.
pub async fn version() -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .ok()?;
    let resp = client.get(format!("{BASE_URL}/api/version")).send().await.ok()?;
    resp.json::<VersionResponse>().await.ok().map(|v| v.version)
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TagEntry>,
}

#[derive(Debug, Deserialize)]
struct TagEntry {
    name: String,
    #[serde(default)]
    size: u64,
}

/// Modelos ja baixados nesta maquina.
pub async fn installed_models() -> Vec<String> {
    let Ok(resp) = http().get(format!("{BASE_URL}/api/tags")).send().await else {
        return Vec::new();
    };
    resp.json::<TagsResponse>()
        .await
        .map(|t| t.models.into_iter().map(|m| m.name).collect())
        .unwrap_or_default()
}

/// Tamanho real em disco de cada modelo baixado, para conferir a estimativa
/// do catalogo contra a realidade.
pub async fn installed_sizes() -> Vec<(String, u64)> {
    let Ok(resp) = http().get(format!("{BASE_URL}/api/tags")).send().await else {
        return Vec::new();
    };
    resp.json::<TagsResponse>()
        .await
        .map(|t| t.models.into_iter().map(|m| (m.name, m.size)).collect())
        .unwrap_or_default()
}

#[derive(Debug, Deserialize)]
struct PsResponse {
    #[serde(default)]
    models: Vec<PsEntry>,
}

#[derive(Debug, Deserialize)]
struct PsEntry {
    name: String,
    #[serde(default)]
    size_vram: u64,
    #[serde(default)]
    size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadedModel {
    pub name: String,
    pub bytes: u64,
}

/// Modelos residentes em memoria neste instante.
pub async fn loaded_models() -> Vec<LoadedModel> {
    let Ok(resp) = http().get(format!("{BASE_URL}/api/ps")).send().await else {
        return Vec::new();
    };
    resp.json::<PsResponse>()
        .await
        .map(|p| {
            p.models
                .into_iter()
                .map(|m| LoadedModel {
                    name: m.name,
                    bytes: if m.size_vram > 0 { m.size_vram } else { m.size },
                })
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    stream: bool,
    /// 0 descarrega o modelo da memoria imediatamente apos responder.
    keep_alive: i32,
    options: GenerateOptions,
    /// Modelos hibridos da familia Qwen raciocinam antes de responder e mandam
    /// isso num campo separado. Com `think: false` o orcamento de tokens vai
    /// inteiro para a resposta.
    #[serde(skip_serializing_if = "Option::is_none")]
    think: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    images: Vec<String>,
}

#[derive(Serialize)]
pub struct GenerateOptions {
    pub temperature: f32,
    pub num_ctx: u32,
    pub num_predict: i32,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self { temperature: 0.7, num_ctx: 8192, num_predict: 2048 }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct GenerateResponse {
    #[serde(default)]
    pub response: String,
    /// Raciocinio interno, quando o modelo pensa. Vai para a transcricao, nunca
    /// para o proximo cargo.
    #[serde(default)]
    pub thinking: String,
    #[serde(default)]
    pub total_duration: u64,
    #[serde(default)]
    pub eval_count: u32,
    #[serde(default)]
    pub prompt_eval_count: u32,
}

impl GenerateResponse {
    pub fn tokens_per_second(&self) -> f32 {
        if self.total_duration == 0 {
            return 0.0;
        }
        self.eval_count as f32 / (self.total_duration as f32 / 1_000_000_000.0)
    }
}

/// Uma unica ida ao modelo. Sobe, responde, descarrega.
///
/// `images` sao PNG/JPEG em base64, aceitos apenas por modelos com visao.
/// `json_mode` forca saida em JSON valido, usado quando o cargo precisa devolver
/// estrutura e nao prosa.
/// `think` liga o raciocinio explicito. Deixar ligado num cargo que devolve JSON
/// e desperdicio: o modelo gasta o orcamento inteiro pensando e devolve resposta
/// vazia.
pub async fn generate(
    model: &str,
    system: Option<&str>,
    prompt: &str,
    options: GenerateOptions,
    json_mode: bool,
    think: bool,
    images: Vec<String>,
) -> Result<GenerateResponse, String> {
    let body = GenerateRequest {
        model,
        prompt,
        system,
        stream: false,
        keep_alive: 0,
        options,
        think: Some(think),
        format: if json_mode { Some("json") } else { None },
        images,
    };
    let resp = http()
        .post(format!("{BASE_URL}/api/generate"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama nao respondeu para {model}: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Ollama devolveu {status} para {model}: {text}"));
    }
    serde_json::from_str::<GenerateResponse>(&text)
        .map_err(|e| format!("resposta ilegivel do Ollama: {e} :: {text}"))
}

/// Forca a saida de um modelo da memoria, mesmo que algo o tenha deixado preso.
pub async fn unload(model: &str) -> Result<(), String> {
    let body = serde_json::json!({ "model": model, "keep_alive": 0, "prompt": "" });
    http()
        .post(format!("{BASE_URL}/api/generate"))
        .json(&body)
        .send()
        .await
        .map(|_| ())
        .map_err(|e| format!("falha ao descarregar {model}: {e}"))
}

/// Apaga um modelo do disco. Diferente de `unload`, que so tira da memoria.
///
/// O Ollama devolve 404 quando a tag nao existe, e isso nao e erro para quem
/// pediu para remover: o resultado desejado ja e o estado atual.
pub async fn delete_model(model: &str) -> Result<(), String> {
    let resp = http()
        .delete(format!("{BASE_URL}/api/delete"))
        .json(&serde_json::json!({ "model": model }))
        .send()
        .await
        .map_err(|e| format!("falha ao remover {model}: {e}"))?;

    if resp.status().is_success() || resp.status() == reqwest::StatusCode::NOT_FOUND {
        Ok(())
    } else {
        Err(format!(
            "Ollama recusou remover {model}: {}",
            resp.text().await.unwrap_or_default()
        ))
    }
}

/// Descarrega tudo que estiver residente. Usado pela rotina de otimizacao.
pub async fn unload_all() -> Vec<String> {
    let mut done = Vec::new();
    for m in loaded_models().await {
        if unload(&m.name).await.is_ok() {
            done.push(m.name);
        }
    }
    done
}

#[derive(Debug, Clone, Serialize)]
pub struct PullProgress {
    pub model: String,
    pub status: String,
    pub completed: u64,
    pub total: u64,
    pub percent: f32,
}

#[derive(Debug, Deserialize)]
struct PullChunk {
    #[serde(default)]
    status: String,
    #[serde(default)]
    completed: u64,
    #[serde(default)]
    total: u64,
    #[serde(default)]
    error: Option<String>,
}

/// Baixa um modelo, reportando progresso linha a linha (NDJSON).
pub async fn pull<F>(model: &str, mut on_progress: F) -> Result<(), String>
where
    F: FnMut(PullProgress),
{
    let body = serde_json::json!({ "model": model, "stream": true });
    let mut resp = http()
        .post(format!("{BASE_URL}/api/pull"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("falha ao iniciar download de {model}: {e}"))?;

    let mut buffer = String::new();
    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(newline) = buffer.find('\n') {
            let line: String = buffer.drain(..=newline).collect();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(parsed) = serde_json::from_str::<PullChunk>(line) else {
                continue;
            };
            if let Some(err) = parsed.error {
                return Err(format!("Ollama recusou o download de {model}: {err}"));
            }
            let percent = if parsed.total > 0 {
                (parsed.completed as f32 / parsed.total as f32) * 100.0
            } else {
                0.0
            };
            on_progress(PullProgress {
                model: model.to_string(),
                status: parsed.status,
                completed: parsed.completed,
                total: parsed.total,
                percent,
            });
        }
    }
    Ok(())
}
