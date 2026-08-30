//! gpt-image, pela API de imagens da OpenAI.
//!
//! `POST https://api.openai.com/v1/images/generations`, `Authorization: Bearer`.
//! É o adaptador mais simples dos quatro: síncrono, e a resposta já traz os
//! bytes em base64 — não há link temporário para baixar antes de expirar.
//!
//! NÃO EXERCITADO contra a API real. Escrito a partir da referência oficial.

use std::path::PathBuf;

use super::{gravar, http, GeneratedImage, ImageQuality};

const ENDPOINT: &str = "https://api.openai.com/v1/images/generations";

fn modelo(q: ImageQuality) -> &'static str {
    match q {
        ImageQuality::Rapida => "gpt-image-1-mini",
        ImageQuality::Alta => "gpt-image-1",
    }
}

/// A API aceita um conjunto fechado de tamanhos, não uma proporção qualquer.
/// Pedir um valor fora da lista é 400, então a proporção da rede é mapeada
/// para o vizinho mais próximo que existe.
fn tamanho(aspect_ratio: &str) -> &'static str {
    match aspect_ratio {
        "9:16" | "4:5" => "1024x1536",
        "16:9" | "1.91:1" => "1536x1024",
        _ => "1024x1024",
    }
}

pub async fn gerar(
    chave: &str,
    prompt: &str,
    aspect_ratio: &str,
    qualidade: ImageQuality,
    out_dir: &PathBuf,
) -> Result<GeneratedImage, String> {
    let modelo = modelo(qualidade);
    let resp = http()
        .post(ENDPOINT)
        .bearer_auth(chave)
        .json(&serde_json::json!({
            "model": modelo,
            "prompt": prompt,
            "size": tamanho(aspect_ratio),
            "n": 1
        }))
        .send()
        .await
        .map_err(|e| format!("falha ao falar com a OpenAI: {e}"))?;

    let status = resp.status();
    let corpo: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("resposta ilegivel da OpenAI ({status}): {e}"))?;

    if let Some(erro) = corpo.get("error") {
        return Err(explicar(erro, status));
    }

    // gpt-image devolve base64 por padrao; `url` existe em modelos antigos.
    let dado = corpo.get("data").and_then(|d| d.get(0)).ok_or_else(|| {
        crate::idioma::msg("A OpenAI nao devolveu imagem.", "OpenAI returned no image.")
    })?;

    let bytes = if let Some(b64) = dado.get("b64_json").and_then(|v| v.as_str()) {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("base64 invalido da OpenAI: {e}"))?
    } else if let Some(url) = dado.get("url").and_then(|v| v.as_str()) {
        super::baixar(url).await?
    } else {
        return Err(crate::idioma::msg(
            "A OpenAI respondeu sem imagem nem link.",
            "OpenAI responded with neither image nor link.",
        ));
    };

    gravar(&bytes, "png", modelo, aspect_ratio, out_dir)
}

fn explicar(erro: &serde_json::Value, status: reqwest::StatusCode) -> String {
    let msg = erro.get("message").and_then(|v| v.as_str()).unwrap_or("");
    let tipo = erro.get("code").and_then(|v| v.as_str()).unwrap_or("");

    // A recusa mais comum nao e chave errada: e organizacao sem verificacao,
    // que devolve 403 com uma mensagem que nao diz o que fazer.
    if status == reqwest::StatusCode::FORBIDDEN || tipo.contains("verif") {
        return crate::idioma::msg(
            "A OpenAI recusou: a organizacao precisa estar verificada para usar \
             geracao de imagem. Verifique em platform.openai.com/settings/organization.",
            "OpenAI refused: the organization must be verified to use image \
             generation. Verify it at platform.openai.com/settings/organization.",
        );
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return crate::idioma::msg(
            "Cota da OpenAI esgotada, ou sem credito no projeto desta chave.",
            "OpenAI quota exhausted, or the key's project has no credit.",
        );
    }
    format!("OpenAI recusou ({status}): {msg}")
}

pub async fn validar(chave: &str) -> Result<String, String> {
    // Lista os modelos em vez de gerar uma imagem: confere a chave sem gastar
    // dinheiro, que e o ponto de um botao de teste.
    let resp = http()
        .get("https://api.openai.com/v1/models")
        .bearer_auth(chave)
        .send()
        .await
        .map_err(|e| format!("falha ao falar com a OpenAI: {e}"))?;

    if resp.status().is_success() {
        Ok("ok".into())
    } else {
        Err(format!(
            "{} ({})",
            crate::idioma::msg("A OpenAI recusou a chave", "OpenAI refused the key"),
            resp.status()
        ))
    }
}
