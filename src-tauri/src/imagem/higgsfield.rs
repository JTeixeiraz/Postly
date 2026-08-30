//! Higgsfield Soul.
//!
//! Assíncrono, e o único que não autentica com uma chave só: o header é
//! `Authorization: Key <id>:<secret>`, com o par que o painel deles emite. Por
//! isso `ProvedorImagem::precisa_de_par` existe — a tela desenha dois campos.
//!
//! O pedido vai para `POST /higgsfield-ai/soul/v2/standard` e volta com
//! `{status, request_id, status_url}`; o resultado sai depois no `status_url`.
//!
//! RESSALVA MAIOR QUE A DOS OUTROS. A documentação oficial descreve o envelope
//! do pedido mas não publica o formato do resultado, e fontes de terceiros
//! divergem sobre o próprio endpoint (`/v1/generations` em um artigo,
//! `/higgsfield-ai/soul/v2/standard` na doc oficial). Implementei contra a doc
//! oficial e a busca pelo link varre a resposta inteira em vez de apostar num
//! campo. NÃO EXERCITADO contra a API real: espere ajuste no primeiro uso.

use std::path::PathBuf;

use super::{achar_url, baixar, gravar, http, GeneratedImage, ImageQuality};

const BASE: &str = "https://api.higgsfield.ai";

fn caminho(q: ImageQuality) -> &'static str {
    match q {
        ImageQuality::Rapida => "/higgsfield-ai/soul/v2/turbo",
        ImageQuality::Alta => "/higgsfield-ai/soul/v2/standard",
    }
}

/// O par `id:secret` vive numa string só no cofre, separado por dois-pontos.
/// Guardar dois campos separados espalharia a mesma credencial em dois lugares.
fn cabecalho(chave: &str) -> String {
    format!("Key {}", chave.trim())
}

pub async fn gerar(
    chave: &str,
    prompt: &str,
    aspect_ratio: &str,
    qualidade: ImageQuality,
    out_dir: &PathBuf,
) -> Result<GeneratedImage, String> {
    if !chave.contains(':') {
        return Err(crate::idioma::msg(
            "A credencial do Higgsfield e um par: cole o id e o segredo separados \
             por dois-pontos (id:segredo).",
            "The Higgsfield credential is a pair: paste the id and secret separated \
             by a colon (id:secret).",
        ));
    }

    let modelo = caminho(qualidade);
    let resp = http()
        .post(format!("{BASE}{modelo}"))
        .header("Authorization", cabecalho(chave))
        .json(&serde_json::json!({
            "prompt": prompt,
            "aspect_ratio": aspect_ratio
        }))
        .send()
        .await
        .map_err(|e| format!("falha ao falar com o Higgsfield: {e}"))?;

    let status = resp.status();
    let aberto: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("resposta ilegivel do Higgsfield ({status}): {e}"))?;

    if !status.is_success() {
        return Err(explicar(status, &aberto));
    }

    let status_url = aberto
        .get("status_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            aberto
                .get("request_id")
                .and_then(|v| v.as_str())
                .map(|id| format!("{BASE}/v1/generations/{id}"))
        })
        .ok_or_else(|| {
            crate::idioma::msg(
                "O Higgsfield aceitou o pedido mas nao disse onde buscar o resultado.",
                "Higgsfield accepted the request but did not say where to fetch the result.",
            )
        })?;

    let url = aguardar(&status_url, chave).await?;
    let bytes = baixar(&url).await?;
    gravar(&bytes, "jpg", "higgsfield-soul", aspect_ratio, out_dir)
}

async fn aguardar(status_url: &str, chave: &str) -> Result<String, String> {
    let limite = std::time::Instant::now() + std::time::Duration::from_secs(240);

    loop {
        if std::time::Instant::now() > limite {
            return Err(crate::idioma::msg(
                "O Higgsfield passou do tempo limite gerando esta imagem.",
                "Higgsfield took too long generating this image.",
            ));
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let resp = http()
            .get(status_url)
            .header("Authorization", cabecalho(chave))
            .send()
            .await
            .map_err(|e| format!("falha ao consultar o Higgsfield: {e}"))?;

        let corpo: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("resposta ilegivel do Higgsfield: {e}"))?;

        // Os estados nao estao publicados. Em vez de casar com uma lista que
        // pode nao existir, o adaptador para no primeiro link de imagem que
        // aparecer, e so trata como erro o que declara erro explicitamente.
        let estado = corpo
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        if let Some(url) = achar_url(&corpo) {
            return Ok(url);
        }
        if estado.contains("fail") || estado.contains("error") || estado.contains("cancel") {
            return Err(format!(
                "{}: {}",
                crate::idioma::msg("O Higgsfield falhou", "Higgsfield failed"),
                corpo
                    .get("error")
                    .or_else(|| corpo.get("message"))
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| estado.clone())
            ));
        }
        if estado.contains("moderat") || estado.contains("reject") {
            return Err(crate::idioma::msg(
                "O Higgsfield bloqueou o pedido por politica de conteudo.",
                "Higgsfield blocked the request on content policy.",
            ));
        }
    }
}

fn explicar(status: reqwest::StatusCode, corpo: &serde_json::Value) -> String {
    match status {
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => crate::idioma::msg(
            "O Higgsfield recusou a credencial. Confira o par id:segredo.",
            "Higgsfield refused the credential. Check the id:secret pair.",
        ),
        reqwest::StatusCode::PAYMENT_REQUIRED | reqwest::StatusCode::TOO_MANY_REQUESTS => {
            crate::idioma::msg(
                "Sem creditos ou cota esgotada no Higgsfield.",
                "No credits or quota exhausted at Higgsfield.",
            )
        }
        _ => format!(
            "Higgsfield recusou ({status}): {}",
            corpo.to_string().chars().take(200).collect::<String>()
        ),
    }
}

pub async fn validar(chave: &str) -> Result<String, String> {
    if !chave.contains(':') {
        return Err(crate::idioma::msg(
            "Cole o par id:segredo.",
            "Paste the id:secret pair.",
        ));
    }
    // Nao ha endpoint de conta publicado. O teste bate na raiz da API com a
    // credencial: 401 e 403 provam que ela esta errada, e qualquer outra
    // resposta prova pelo menos que o servico respondeu.
    let resp = http()
        .get(format!("{BASE}/v1/generations"))
        .header("Authorization", cabecalho(chave))
        .send()
        .await
        .map_err(|e| format!("falha ao falar com o Higgsfield: {e}"))?;

    match resp.status() {
        s if s == reqwest::StatusCode::UNAUTHORIZED || s == reqwest::StatusCode::FORBIDDEN => {
            Err(crate::idioma::msg(
                "O Higgsfield recusou a credencial.",
                "Higgsfield refused the credential.",
            ))
        }
        _ => Ok(crate::idioma::msg(
            "credencial aceita (o Higgsfield nao publica endpoint de conta, entao \
             isto confere o acesso, nao o saldo)",
            "credential accepted (Higgsfield publishes no account endpoint, so this \
             checks access, not balance)",
        )),
    }
}
