//! FLUX, da Black Forest Labs.
//!
//! Assíncrono: `POST https://api.bfl.ai/v1/<modelo>` devolve `{id, polling_url}`,
//! e o resultado sai depois num `GET` ao `polling_url`. Autentica com o header
//! `x-key`, não com Bearer nem com `X-API-Key`.
//!
//! O nome do header foi verificado contra a API real, porque errá-lo não dá
//! erro distinguível: com `X-API-Key` a resposta é `{"detail":"Not
//! authenticated"}`, exatamente a mesma de não mandar header nenhum. Só com
//! `x-key` a API chega a olhar o valor — e aí responde `{"detail":"Invalid API
//! key format"}`, que é o que prova que ela leu.
//!
//! Os estados vêm do OpenAPI oficial (`api.bfl.ai/openapi.json`): `Pending`,
//! `Reasoning`, `Generating`, `Ready`, `Error`, `Request Moderated`,
//! `Content Moderated`, `Task not found`. O envelope do resultado é tipado, mas
//! o campo `result` não — por isso a busca pelo link é defensiva.
//!
//! NÃO EXERCITADO contra a API real.

use std::path::PathBuf;

use super::{achar_url, baixar, dimensoes, gravar, http, GeneratedImage, ImageQuality};

const BASE: &str = "https://api.bfl.ai/v1";

fn modelo(q: ImageQuality) -> &'static str {
    match q {
        ImageQuality::Rapida => "flux-2-klein-9b",
        ImageQuality::Alta => "flux-2-pro",
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
    let (largura, altura) = dimensoes(aspect_ratio);

    let resp = http()
        .post(format!("{BASE}/{modelo}"))
        .header("x-key", chave)
        .json(&serde_json::json!({
            "prompt": prompt,
            "width": largura,
            "height": altura,
            "output_format": "jpeg"
        }))
        .send()
        .await
        .map_err(|e| format!("falha ao falar com a Black Forest Labs: {e}"))?;

    let status = resp.status();
    let aberto: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("resposta ilegivel da BFL ({status}): {e}"))?;

    if !status.is_success() {
        return Err(explicar(status, &aberto));
    }

    let polling = aberto
        .get("polling_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            aberto
                .get("id")
                .and_then(|v| v.as_str())
                .map(|id| format!("{BASE}/get-result?id={id}"))
        })
        .ok_or_else(|| {
            crate::idioma::msg(
                "A BFL aceitou o pedido mas nao disse onde buscar o resultado.",
                "BFL accepted the request but did not say where to fetch the result.",
            )
        })?;

    let url = aguardar(&polling, chave).await?;
    let bytes = baixar(&url).await?;
    gravar(&bytes, "jpg", modelo, aspect_ratio, out_dir)
}

/// Espera o resultado ficar pronto.
///
/// Intervalo fixo de 2s e teto de 4 minutos. Sem o teto, um trabalho que trava
/// do lado deles deixaria a campanha parada segurando o navegador e a pasta da
/// execução até alguém fechar o app.
async fn aguardar(polling_url: &str, chave: &str) -> Result<String, String> {
    let limite = std::time::Instant::now() + std::time::Duration::from_secs(240);

    loop {
        if std::time::Instant::now() > limite {
            return Err(crate::idioma::msg(
                "A Black Forest Labs passou do tempo limite gerando esta imagem.",
                "Black Forest Labs took too long generating this image.",
            ));
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let resp = http()
            .get(polling_url)
            .header("x-key", chave)
            .send()
            .await
            .map_err(|e| format!("falha ao consultar a BFL: {e}"))?;

        let corpo: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("resposta ilegivel da BFL: {e}"))?;

        match corpo.get("status").and_then(|v| v.as_str()).unwrap_or("") {
            "Ready" => {
                let resultado = corpo.get("result").unwrap_or(&corpo);
                return achar_url(resultado).ok_or_else(|| {
                    crate::idioma::msg(
                        "A BFL disse que ficou pronto mas nao devolveu o link da arte.",
                        "BFL reported ready but returned no art link.",
                    )
                });
            }
            "Content Moderated" | "Request Moderated" => {
                return Err(crate::idioma::msg(
                    "A Black Forest Labs bloqueou o pedido por politica de conteudo. \
                     Reescreva o conceito da peca.",
                    "Black Forest Labs blocked the request on content policy. \
                     Rewrite the piece's concept.",
                ))
            }
            "Error" | "Task not found" => {
                return Err(format!(
                    "{}: {}",
                    crate::idioma::msg("A BFL falhou", "BFL failed"),
                    corpo
                        .get("details")
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "sem detalhe".into())
                ))
            }
            // Pending, Reasoning, Generating: continua esperando.
            _ => continue,
        }
    }
}

fn explicar(status: reqwest::StatusCode, corpo: &serde_json::Value) -> String {
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return crate::idioma::msg(
            "A Black Forest Labs recusou a chave.",
            "Black Forest Labs refused the key.",
        );
    }
    if status == reqwest::StatusCode::PAYMENT_REQUIRED {
        return crate::idioma::msg(
            "Sem creditos na Black Forest Labs. Recarregue em dashboard.bfl.ai.",
            "No credits at Black Forest Labs. Top up at dashboard.bfl.ai.",
        );
    }
    format!("BFL recusou ({status}): {}", primeira_linha(corpo))
}

fn primeira_linha(v: &serde_json::Value) -> String {
    let t = v
        .get("detail")
        .or_else(|| v.get("message"))
        .map(|d| d.to_string())
        .unwrap_or_else(|| v.to_string());
    t.chars().take(200).collect()
}

pub async fn validar(chave: &str) -> Result<String, String> {
    // Consulta o saldo: confere a chave sem enfileirar uma geracao paga.
    let resp = http()
        .get("https://api.bfl.ai/v1/get_balance")
        .header("x-key", chave)
        .send()
        .await
        .map_err(|e| format!("falha ao falar com a BFL: {e}"))?;

    if resp.status().is_success() {
        Ok("ok".into())
    } else {
        Err(format!(
            "{} ({})",
            crate::idioma::msg("A BFL recusou a chave", "BFL refused the key"),
            resp.status()
        ))
    }
}
