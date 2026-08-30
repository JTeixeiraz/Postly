//! Quem gera a arte da peça.
//!
//! O Gemini era o único, e estava soldado no orquestrador. Isso tinha dois
//! problemas: quem não tem faturamento ativo no Google não conseguia gerar
//! imagem nenhuma, e trocar de gerador exigia mexer no pipeline.
//!
//! Agora cada serviço é um adaptador pequeno com o mesmo contrato: entra um
//! prompt e uma proporção, sai um arquivo em disco. O pipeline não sabe qual
//! está ativo.
//!
//! HONESTIDADE SOBRE O QUE FOI TESTADO. Só o Gemini rodou contra a API real,
//! com uma chave de verdade. Os outros quatro foram escritos a partir da
//! documentação oficial de cada um e nunca fizeram uma chamada. Cada adaptador
//! declara isso em `verificado`, e a tela mostra o aviso: prometer que cinco
//! integrações funcionam quando quatro nunca foram exercitadas seria mentir
//! para quem for confiar nisto numa campanha.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

pub mod bfl;
pub mod higgsfield;
pub mod openai;
pub mod stability;

pub use crate::gemini::{GeneratedImage, ImageQuality};

/// Serviços que sabem gerar a arte.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvedorImagem {
    /// Padrão: já vinha configurado e é o único exercitado contra a API real.
    #[default]
    Gemini,
    /// gpt-image, pela API de imagens da OpenAI.
    OpenAi,
    /// FLUX, da Black Forest Labs. Assíncrono, com polling.
    Flux,
    /// Stable Image, da Stability AI. Devolve os bytes direto.
    Stability,
    /// Higgsfield Soul. Assíncrono, com polling.
    Higgsfield,
}

impl ProvedorImagem {
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Gemini => "gemini",
            Self::OpenAi => "openai",
            Self::Flux => "flux",
            Self::Stability => "stability",
            Self::Higgsfield => "higgsfield",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Gemini => "Gemini",
            Self::OpenAi => "OpenAI",
            Self::Flux => "FLUX",
            Self::Stability => "Stability AI",
            Self::Higgsfield => "Higgsfield",
        }
    }

    /// Rodou contra a API real alguma vez?
    pub fn verificado(&self) -> bool {
        matches!(self, Self::Gemini)
    }

    /// Onde a pessoa pega a chave. Sem isto, "configure sua chave" é um beco.
    pub fn url_da_chave(&self) -> &'static str {
        match self {
            Self::Gemini => "https://aistudio.google.com/apikey",
            Self::OpenAi => "https://platform.openai.com/api-keys",
            Self::Flux => "https://dashboard.bfl.ai/keys",
            Self::Stability => "https://platform.stability.ai/account/keys",
            Self::Higgsfield => "https://cloud.higgsfield.ai/api-keys",
        }
    }

    /// O Higgsfield autentica com um par `id:secret`, não com uma chave só.
    /// A tela precisa saber disso para desenhar dois campos em vez de um.
    pub fn precisa_de_par(&self) -> bool {
        matches!(self, Self::Higgsfield)
    }

    pub fn todos() -> [Self; 5] {
        [Self::Gemini, Self::OpenAi, Self::Flux, Self::Stability, Self::Higgsfield]
    }
}

pub fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(240))
        .build()
        .expect("cliente http")
}

/// Grava os bytes com a extensão certa e devolve o caminho.
///
/// Fica aqui e não em cada adaptador porque o nome do arquivo é contrato com o
/// resto do app: o Playwright vai anexá-lo e a galeria do histórico vai exibi-lo.
pub fn gravar(
    bytes: &[u8],
    ext: &str,
    modelo: &str,
    aspect_ratio: &str,
    out_dir: &PathBuf,
) -> Result<GeneratedImage, String> {
    if bytes.len() < 1024 {
        return Err(crate::idioma::msg(
            "O serviço devolveu um arquivo pequeno demais para ser uma imagem.",
            "The service returned a file too small to be an image.",
        ));
    }
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%3f");
    let path = out_dir.join(format!("post-{stamp}.{ext}"));
    std::fs::write(&path, bytes).map_err(|e| format!("falha ao gravar a imagem: {e}"))?;

    Ok(GeneratedImage {
        path: path.to_string_lossy().to_string(),
        bytes: bytes.len() as u64,
        model: modelo.to_string(),
        aspect_ratio: aspect_ratio.to_string(),
    })
}

/// Baixa a arte de uma URL assinada.
///
/// Os serviços assíncronos (FLUX, Higgsfield) não devolvem os bytes: devolvem
/// um link temporário. Baixar na hora é o que impede a peça de sumir quando o
/// link expira, tipicamente em minutos.
pub async fn baixar(url: &str) -> Result<Vec<u8>, String> {
    let resp = http()
        .get(url)
        .send()
        .await
        .map_err(|e| format!("falha ao baixar a arte: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("o link da arte devolveu {}", resp.status()));
    }
    Ok(resp
        .bytes()
        .await
        .map_err(|e| format!("falha ao ler a arte: {e}"))?
        .to_vec())
}

/// Procura a primeira coisa que pareça um link de imagem dentro de um JSON.
///
/// Existe porque a documentação do FLUX e a do Higgsfield descrevem o envelope
/// da resposta mas deixam o campo do resultado sem tipo. Em vez de apostar num
/// nome e falhar em silêncio quando ele mudar, o adaptador tenta o campo
/// documentado e cai nesta varredura. É defensivo de propósito: são as duas
/// integrações que eu não pude exercitar.
pub fn achar_url(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => {
            let baixo = s.to_lowercase();
            (baixo.starts_with("http")
                && [".png", ".jpg", ".jpeg", ".webp"]
                    .iter()
                    .any(|e| baixo.contains(e)))
            .then(|| s.clone())
        }
        serde_json::Value::Array(a) => a.iter().find_map(achar_url),
        serde_json::Value::Object(o) => {
            // Os nomes prováveis primeiro, para não pegar uma miniatura por acaso.
            for k in ["sample", "url", "image_url", "output_url", "result_url"] {
                if let Some(u) = o.get(k).and_then(achar_url) {
                    return Some(u);
                }
            }
            o.values().find_map(achar_url)
        }
        _ => None,
    }
}

/// Converte a proporção em largura e altura.
///
/// Metade dos serviços aceita `aspect_ratio` como texto e a outra metade só
/// aceita pixels. Os valores são múltiplos de 32 porque difusão trabalha em
/// blocos e um lado quebrado é recusado ou silenciosamente arredondado.
pub fn dimensoes(aspect_ratio: &str) -> (u32, u32) {
    match aspect_ratio {
        "9:16" => (768, 1344),
        "4:5" => (896, 1120),
        "16:9" => (1344, 768),
        "1.91:1" => (1344, 704),
        _ => (1024, 1024),
    }
}

// ------------------------------------------------------------------ despacho

/// Gera a arte pelo serviço escolhido.
pub async fn gerar(
    provedor: ProvedorImagem,
    cofre: &crate::vault::Vault,
    prompt: &str,
    aspect_ratio: &str,
    qualidade: ImageQuality,
    out_dir: &PathBuf,
) -> Result<GeneratedImage, String> {
    let chave = cofre.chave_de(provedor);
    if chave.trim().is_empty() {
        return Err(format!(
            "{} {}.",
            crate::idioma::msg(
                "Nenhuma chave configurada para",
                "No key configured for"
            ),
            provedor.label()
        ));
    }

    match provedor {
        ProvedorImagem::Gemini => {
            crate::gemini::generate_image(&chave, prompt, aspect_ratio, qualidade, out_dir).await
        }
        ProvedorImagem::OpenAi => openai::gerar(&chave, prompt, aspect_ratio, qualidade, out_dir).await,
        ProvedorImagem::Flux => bfl::gerar(&chave, prompt, aspect_ratio, qualidade, out_dir).await,
        ProvedorImagem::Stability => {
            stability::gerar(&chave, prompt, aspect_ratio, qualidade, out_dir).await
        }
        ProvedorImagem::Higgsfield => {
            higgsfield::gerar(&chave, prompt, aspect_ratio, qualidade, out_dir).await
        }
    }
}

/// Confere a chave antes de a pessoa gastar uma campanha inteira descobrindo
/// que ela está errada.
pub async fn validar(provedor: ProvedorImagem, chave: &str) -> Result<String, String> {
    match provedor {
        ProvedorImagem::Gemini => crate::gemini::validate_key(chave).await,
        ProvedorImagem::OpenAi => openai::validar(chave).await,
        ProvedorImagem::Flux => bfl::validar(chave).await,
        ProvedorImagem::Stability => stability::validar(chave).await,
        ProvedorImagem::Higgsfield => higgsfield::validar(chave).await,
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn a_varredura_acha_o_link_onde_quer_que_ele_esteja() {
        // O campo documentado.
        let v = serde_json::json!({ "result": { "sample": "https://x.com/a.jpg" } });
        assert_eq!(achar_url(&v).as_deref(), Some("https://x.com/a.jpg"));

        // Um nome que a documentacao nao previu, dentro de um array.
        let v = serde_json::json!({ "results": [{ "media": "https://x.com/b.png" }] });
        assert_eq!(achar_url(&v).as_deref(), Some("https://x.com/b.png"));

        // `url` vence os irmaos, mesmo que outro campo tambem seja um link.
        let v = serde_json::json!({ "thumb": "https://x.com/t.jpg", "url": "https://x.com/full.jpg" });
        assert_eq!(achar_url(&v).as_deref(), Some("https://x.com/full.jpg"));

        // Texto que nao e link nao pode ser confundido com um.
        assert!(achar_url(&serde_json::json!({ "status": "Ready" })).is_none());
        assert!(achar_url(&serde_json::json!({ "doc": "https://x.com/guia" })).is_none());
    }

    #[test]
    fn a_proporcao_vira_lado_multiplo_de_32() {
        for ar in ["9:16", "4:5", "16:9", "1.91:1", "1:1", "coisa invalida"] {
            let (w, h) = dimensoes(ar);
            assert_eq!(w % 32, 0, "largura de {ar} nao e multiplo de 32");
            assert_eq!(h % 32, 0, "altura de {ar} nao e multiplo de 32");
        }
        // A orientacao precisa bater com a proporcao pedida.
        assert!(dimensoes("9:16").1 > dimensoes("9:16").0, "9:16 tem que ser vertical");
        assert!(dimensoes("16:9").0 > dimensoes("16:9").1, "16:9 tem que ser horizontal");
        assert_eq!(dimensoes("1:1").0, dimensoes("1:1").1);
    }

    #[test]
    fn arquivo_pequeno_demais_nao_passa_por_imagem() {
        let dir = std::env::temp_dir().join("postly-teste-imagem");
        // Uma resposta de erro em texto tem essa cara: gravar isso como .png
        // deixaria a campanha seguir com uma peca quebrada.
        assert!(gravar(b"{\"error\":\"nope\"}", "png", "m", "1:1", &dir).is_err());
    }
}
