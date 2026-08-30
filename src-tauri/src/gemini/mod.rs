//! Cliente da API do Gemini (Interactions API).
//!
//! O criador de conteudo usa este modulo para as duas entregas obrigatorias de
//! toda publicacao: a imagem e a legenda. Os modelos locais decidem o QUE
//! gerar; o Gemini executa a geracao final.
//!
//! Verificado contra a API real com uma chave valida:
//!
//! - `POST /v1beta/interactions` responde 200 e devolve `steps[]`, onde o
//!   passo de tipo `model_output` carrega o conteudo. Pode vir antes dele um
//!   passo `thought`, sem `content`, entao a busca varre os passos de tras
//!   para frente em vez de olhar so o ultimo.
//! - `response_format.mime_type` aceita **apenas** `image/jpeg`. Pedir
//!   `image/png` devolve 400 com essa mensagem exata.
//! - O erro vem com `code` em TEXTO (`too_many_requests`, `invalid_request`),
//!   nao em numero como na API classica.

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

const ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/interactions";

/// Modelos conferidos contra `GET /v1beta/models` com uma chave real.
pub const MODEL_TEXT: &str = "gemini-3.7-flash";
pub const MODEL_IMAGE_FAST: &str = "gemini-3.1-flash-image";
pub const MODEL_IMAGE_QUALITY: &str = "gemini-3-pro-image";

/// A API so aceita JPEG na geracao de imagem.
const MIME_IMAGEM: &str = "image/jpeg";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageQuality {
    /// gemini-3.1-flash-image: barato e rapido, suficiente para feed diario.
    Rapida,
    /// gemini-3-pro-image (Nano Banana Pro): texto dentro da imagem e coerencia
    /// de marca muito melhores, e mais caro.
    Alta,
}

impl ImageQuality {
    fn model(&self) -> &'static str {
        match self {
            ImageQuality::Rapida => MODEL_IMAGE_FAST,
            ImageQuality::Alta => MODEL_IMAGE_QUALITY,
        }
    }
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(240))
        .build()
        .expect("cliente http")
}

// ------------------------------------------------------------------ resposta

#[derive(Debug, Deserialize, Default)]
struct InteractionResponse {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    output_text: Option<String>,
    #[serde(default)]
    output_image: Option<OutputImage>,
    #[serde(default)]
    steps: Vec<Step>,
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize, Default)]
struct OutputImage {
    #[serde(default)]
    data: String,
}

#[derive(Debug, Deserialize, Default)]
struct Step {
    #[serde(default)]
    content: Vec<Content>,
}

#[derive(Debug, Deserialize, Default)]
struct Content {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    data: Option<String>,
}

/// `code` chega como texto nesta API (`too_many_requests`), nao como numero.
#[derive(Debug, Deserialize)]
struct ApiError {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
}

impl InteractionResponse {
    fn text(&self) -> Option<String> {
        if let Some(t) = &self.output_text {
            if !t.trim().is_empty() {
                return Some(t.clone());
            }
        }
        // De tras para frente: o passo final e o `model_output`, mas pode haver
        // um `thought` sem conteudo depois dele.
        self.steps.iter().rev().find_map(|s| {
            s.content
                .iter()
                .find(|c| c.kind == "text")
                .and_then(|c| c.text.clone())
                .filter(|t| !t.trim().is_empty())
        })
    }

    fn image_base64(&self) -> Option<String> {
        if let Some(img) = &self.output_image {
            if !img.data.is_empty() {
                return Some(img.data.clone());
            }
        }
        self.steps
            .iter()
            .flat_map(|s| s.content.iter())
            .find(|c| c.kind == "image" && c.data.as_deref().is_some_and(|d| !d.is_empty()))
            .and_then(|c| c.data.clone())
    }
}

/// Traduz o erro da API em algo que a pessoa consiga agir em cima.
///
/// O caso que mais aparece na pratica e o `limit: 0`: os modelos de imagem nao
/// tem cota nenhuma no nivel gratuito, entao a mensagem crua ("exceeded your
/// quota") faz a pessoa esperar por uma janela que nunca vai abrir.
fn explicar(err: &ApiError) -> String {
    let msg = err.message.as_str();
    match err.code.as_str() {
        "too_many_requests" if msg.contains("limit: 0") => crate::idioma::msg(
            "Este modelo de imagem nao esta disponivel no nivel gratuito da API \
             (cota zero). Ative faturamento no projeto Google Cloud ligado a esta \
             chave, em console.cloud.google.com/billing. Assinar o Google AI Studio \
             Pro nao habilita a API: sao produtos separados.",
            "This image model has no free-tier quota at all (limit: 0). Enable \
             billing on the Google Cloud project behind this key, at \
             console.cloud.google.com/billing. A Google AI Studio Pro subscription \
             does not enable the API: they are separate products.",
        ),
        "too_many_requests" => format!(
            "{} ({})",
            crate::idioma::msg(
                "Cota da API do Gemini esgotada. Espere a janela virar ou revise o \
                 plano do projeto da chave.",
                "Gemini API quota exhausted. Wait for the window to reset or review \
                 the plan on the key's project.",
            ),
            primeira_linha(msg)
        ),
        "invalid_request" => format!(
            "{}: {}",
            crate::idioma::msg("O Gemini recusou o pedido", "Gemini rejected the request"),
            primeira_linha(msg)
        ),
        "unauthenticated" | "permission_denied" => format!(
            "{}: {}",
            crate::idioma::msg("O Gemini recusou a chave", "Gemini rejected the key"),
            primeira_linha(msg)
        ),
        "unavailable" => crate::idioma::msg(
            "O modelo do Gemini esta sobrecarregado agora. Tente de novo em alguns \
             minutos ou use a qualidade Rapida.",
            "The Gemini model is overloaded right now. Try again in a few minutes or \
             switch to Fast quality.",
        ),
        "" => format!("Gemini recusou: {}", primeira_linha(msg)),
        outro => format!("Gemini recusou ({outro}): {}", primeira_linha(msg)),
    }
}

fn primeira_linha(s: &str) -> String {
    truncate(s.lines().next().unwrap_or(s).trim(), 220)
}

async fn call(api_key: &str, body: serde_json::Value) -> Result<InteractionResponse, String> {
    if api_key.trim().is_empty() {
        return Err(crate::idioma::msg(
            "Chave da API do Gemini nao configurada.",
            "Gemini API key is not set.",
        ));
    }
    let resp = http()
        .post(ENDPOINT)
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Gemini inacessivel: {e}"))?;

    let status = resp.status();
    let raw = resp.text().await.map_err(|e| e.to_string())?;
    let parsed: InteractionResponse = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "resposta ilegivel do Gemini ({status}): {e} :: {}",
            truncate(&raw, 400)
        )
    })?;

    if let Some(err) = &parsed.error {
        return Err(explicar(err));
    }
    if !status.is_success() {
        return Err(format!("Gemini devolveu {status}: {}", truncate(&raw, 400)));
    }
    Ok(parsed)
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(n).collect::<String>())
    }
}

// -------------------------------------------------------------------- texto

/// Gera ou refina a legenda da publicacao.
pub async fn generate_caption(api_key: &str, prompt: &str) -> Result<String, String> {
    let body = serde_json::json!({ "model": MODEL_TEXT, "input": prompt });
    call(api_key, body)
        .await?
        .text()
        .ok_or_else(|| "Gemini nao devolveu texto.".to_string())
}

// -------------------------------------------------------------------- imagem

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    pub path: String,
    pub bytes: u64,
    pub model: String,
    pub aspect_ratio: String,
}

/// Gera a arte e grava em disco. Devolve o caminho, que segue para o Playwright
/// no momento da publicacao.
pub async fn generate_image(
    api_key: &str,
    prompt: &str,
    aspect_ratio: &str,
    quality: ImageQuality,
    out_dir: &PathBuf,
) -> Result<GeneratedImage, String> {
    let model = quality.model();
    let body = serde_json::json!({
        "model": model,
        "input": [{ "type": "text", "text": prompt }],
        "response_format": {
            "type": "image",
            "mime_type": MIME_IMAGEM,
            "aspect_ratio": aspect_ratio,
            "image_size": "2K"
        }
    });

    let resposta = call(api_key, body).await?;
    let encoded = resposta
        .image_base64()
        .ok_or_else(|| match resposta.status.as_deref() {
            Some("blocked") | Some("filtered") => crate::idioma::msg(
                "O Gemini bloqueou o pedido por politica de conteudo. Reescreva o \
                 conceito da peca.",
                "Gemini blocked the request under its content policy. Rewrite the \
                 concept of the piece.",
            ),
            Some(outro) => format!("Gemini nao devolveu imagem (status: {outro})."),
            None => "Gemini nao devolveu imagem.".to_string(),
        })?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map_err(|e| format!("imagem do Gemini veio em base64 invalido: {e}"))?;

    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%3f");
    // A extensao acompanha o que a API realmente devolve. Gravar JPEG com nome
    // .png quebra qualquer consumidor que confie no sufixo.
    let path = out_dir.join(format!("post-{stamp}.jpg"));
    std::fs::write(&path, &bytes).map_err(|e| format!("falha ao gravar a imagem: {e}"))?;

    Ok(GeneratedImage {
        path: path.to_string_lossy().to_string(),
        bytes: bytes.len() as u64,
        model: model.to_string(),
        aspect_ratio: aspect_ratio.to_string(),
    })
}

/// Confere se a chave funciona antes de o usuario iniciar uma campanha inteira.
pub async fn validate_key(api_key: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "model": MODEL_TEXT,
        "input": "Responda apenas: ok"
    });
    call(api_key, body)
        .await
        .map(|r| r.text().unwrap_or_else(|| "ok".into()))
}
