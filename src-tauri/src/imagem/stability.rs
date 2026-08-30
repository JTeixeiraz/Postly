//! Stable Image, da Stability AI.
//!
//! `POST https://api.stability.ai/v2beta/stable-image/generate/<modelo>`, com
//! `Authorization: Bearer` e corpo `multipart/form-data`. É o único que devolve
//! os bytes da imagem direto no corpo, sem envelope JSON e sem polling — desde
//! que o header `accept` peça `image/*`. Com `accept: application/json` ele
//! troca para base64, e o adaptador usa isso para ler o erro quando falha.
//!
//! NÃO EXERCITADO contra a API real.

use std::path::PathBuf;

use super::{gravar, http, GeneratedImage, ImageQuality};

const BASE: &str = "https://api.stability.ai/v2beta/stable-image/generate";

fn modelo(q: ImageQuality) -> &'static str {
    match q {
        ImageQuality::Rapida => "core",
        ImageQuality::Alta => "ultra",
    }
}

/// A API aceita um conjunto fechado de proporções. O 1.91:1 do Facebook e do
/// LinkedIn não está nele, então cai no 16:9, que é o vizinho mais próximo.
fn proporcao(aspect_ratio: &str) -> &'static str {
    match aspect_ratio {
        "9:16" => "9:16",
        "4:5" => "4:5",
        "16:9" | "1.91:1" => "16:9",
        _ => "1:1",
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

    let form = reqwest::multipart::Form::new()
        .text("prompt", prompt.to_string())
        .text("aspect_ratio", proporcao(aspect_ratio).to_string())
        .text("output_format", "png".to_string());

    let resp = http()
        .post(format!("{BASE}/{modelo}"))
        .bearer_auth(chave)
        .header("accept", "image/*")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("falha ao falar com a Stability: {e}"))?;

    let status = resp.status();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("falha ao ler a resposta da Stability: {e}"))?
        .to_vec();

    if !status.is_success() {
        // No erro o corpo volta em JSON mesmo tendo pedido imagem.
        return Err(explicar(status, &bytes));
    }

    gravar(&bytes, "png", modelo, aspect_ratio, out_dir)
}

fn explicar(status: reqwest::StatusCode, corpo: &[u8]) -> String {
    let texto = String::from_utf8_lossy(corpo);
    let detalhe = serde_json::from_str::<serde_json::Value>(&texto)
        .ok()
        .and_then(|v| {
            v.get("errors")
                .and_then(|e| e.get(0))
                .and_then(|e| e.as_str())
                .map(|s| s.to_string())
                .or_else(|| v.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()))
        })
        .unwrap_or_else(|| texto.chars().take(200).collect());

    match status {
        reqwest::StatusCode::UNAUTHORIZED => crate::idioma::msg(
            "A Stability recusou a chave.",
            "Stability refused the key.",
        ),
        reqwest::StatusCode::PAYMENT_REQUIRED => crate::idioma::msg(
            "Sem creditos na Stability. Recarregue em platform.stability.ai.",
            "No credits at Stability. Top up at platform.stability.ai.",
        ),
        // 403 aqui e quase sempre o filtro de conteudo, nao permissao.
        reqwest::StatusCode::FORBIDDEN => crate::idioma::msg(
            "A Stability bloqueou o pedido por politica de conteudo. Reescreva o \
             conceito da peca.",
            "Stability blocked the request on content policy. Rewrite the piece's \
             concept.",
        ),
        _ => format!("Stability recusou ({status}): {detalhe}"),
    }
}

pub async fn validar(chave: &str) -> Result<String, String> {
    // Consulta a conta: confere a chave sem gastar credito gerando arte.
    let resp = http()
        .get("https://api.stability.ai/v1/user/account")
        .bearer_auth(chave)
        .send()
        .await
        .map_err(|e| format!("falha ao falar com a Stability: {e}"))?;

    if resp.status().is_success() {
        Ok("ok".into())
    } else {
        Err(format!(
            "{} ({})",
            crate::idioma::msg("A Stability recusou a chave", "Stability refused the key"),
            resp.status()
        ))
    }
}
