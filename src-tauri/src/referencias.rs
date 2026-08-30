//! Referencias visuais e identidade da marca.
//!
//! Duas coisas diferentes vivem aqui, e a distincao importa para o prompt:
//!
//! - REFERENCIA PROPRIA: foto do produto, da loja, da equipe. E o material que
//!   a peca pode mostrar.
//! - REFERENCIA DE MARCA: o trabalho de outra marca que a pessoa quer imitar
//!   no resultado. Nao entra na peca; entra como direcao de estilo.
//!
//! Misturar as duas faz o criador colocar a marca do concorrente dentro da
//! arte, entao elas viajam separadas ate o prompt.

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TipoReferencia {
    /// Material da propria marca: pode aparecer na peca.
    Propria,
    /// Trabalho de terceiro usado como direcao de estilo. Nunca entra na arte.
    Marca,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Referencia {
    pub id: String,
    pub nome: String,
    pub caminho: String,
    pub bytes: u64,
    pub tipo: TipoReferencia,
    /// O que a pessoa quer que o time olhe nesta imagem.
    #[serde(default)]
    pub nota: String,
}

/// Identidade visual que o criador precisa respeitar.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignSystem {
    #[serde(default)]
    pub cores: String,
    #[serde(default)]
    pub tipografia: String,
    #[serde(default)]
    pub tom_visual: String,
    /// O que nunca pode aparecer. Vale mais que qualquer instrucao positiva.
    #[serde(default)]
    pub evitar: String,
}

impl DesignSystem {
    pub fn vazio(&self) -> bool {
        self.cores.trim().is_empty()
            && self.tipografia.trim().is_empty()
            && self.tom_visual.trim().is_empty()
            && self.evitar.trim().is_empty()
    }

    /// Bloco pronto para entrar no prompt do criador.
    pub fn bloco(&self) -> String {
        if self.vazio() {
            return String::new();
        }
        let mut linhas = vec!["IDENTIDADE VISUAL DA MARCA. Respeite sem excecao:".to_string()];
        if !self.cores.trim().is_empty() {
            linhas.push(format!("- Paleta: {}", self.cores.trim()));
        }
        if !self.tipografia.trim().is_empty() {
            linhas.push(format!("- Tipografia: {}", self.tipografia.trim()));
        }
        if !self.tom_visual.trim().is_empty() {
            linhas.push(format!("- Tom visual: {}", self.tom_visual.trim()));
        }
        if !self.evitar.trim().is_empty() {
            linhas.push(format!(
                "- NUNCA use, em nenhuma hipotese: {}",
                self.evitar.trim()
            ));
        }
        linhas.join("\n")
    }
}

fn pasta() -> PathBuf {
    platform::current().data_dir().join("referencias")
}

/// Extensoes que o Gemini e os modelos de visao aceitam.
fn extensao_valida(nome: &str) -> Option<&'static str> {
    let n = nome.to_lowercase();
    if n.ends_with(".png") {
        Some("png")
    } else if n.ends_with(".jpg") || n.ends_with(".jpeg") {
        Some("jpg")
    } else if n.ends_with(".webp") {
        Some("webp")
    } else {
        None
    }
}

/// Teto por arquivo. Uma referencia de 20 MB viraria um prompt que nenhum
/// modelo local aguenta, e o custo aparece so na hora do turno.
const MAX_BYTES: usize = 6 * 1024 * 1024;

pub fn salvar(
    nome: &str,
    base64_dados: &str,
    tipo: TipoReferencia,
    nota: &str,
) -> Result<Referencia, String> {
    let ext = extensao_valida(nome).ok_or("Formato nao aceito. Use PNG, JPG ou WEBP.")?;

    // O navegador manda `data:image/png;base64,AAAA...`; so a cauda interessa.
    let cru = base64_dados
        .split_once(",")
        .map(|(_, resto)| resto)
        .unwrap_or(base64_dados);

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(cru.trim())
        .map_err(|e| format!("imagem invalida: {e}"))?;

    if bytes.len() > MAX_BYTES {
        return Err(format!(
            "Imagem de {:.1} MB e grande demais. O limite e 6 MB por referencia.",
            bytes.len() as f64 / (1024.0 * 1024.0)
        ));
    }

    let dir = pasta();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let id = format!("{}", chrono::Utc::now().timestamp_millis());
    let arquivo = dir.join(format!("{id}.{ext}"));
    std::fs::write(&arquivo, &bytes).map_err(|e| format!("falha ao gravar referencia: {e}"))?;

    Ok(Referencia {
        id,
        nome: nome.to_string(),
        caminho: arquivo.to_string_lossy().to_string(),
        bytes: bytes.len() as u64,
        tipo,
        nota: nota.to_string(),
    })
}

pub fn remover(caminho: &str) -> Result<(), String> {
    // So apaga dentro da pasta de referencias: caminho vindo do frontend nao
    // pode virar remocao arbitraria de arquivo.
    let alvo = PathBuf::from(caminho);
    if !alvo.starts_with(pasta()) {
        return Err("caminho fora da pasta de referencias".into());
    }
    match std::fs::remove_file(&alvo) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("falha ao remover: {e}")),
    }
}

/// Le as referencias de volta em base64, para o turno do modelo com visao.
pub fn como_base64(refs: &[Referencia]) -> Vec<String> {
    refs.iter()
        .filter_map(|r| std::fs::read(&r.caminho).ok())
        .map(|b| base64::engine::general_purpose::STANDARD.encode(b))
        .collect()
}

/// Descricao textual das referencias, para os modelos SEM visao.
///
/// Sem isto, quem nao enxerga imagem simplesmente ignoraria que existe
/// material de apoio, e a nota que a pessoa escreveu se perderia.
pub fn bloco_descritivo(refs: &[Referencia]) -> String {
    if refs.is_empty() {
        return String::new();
    }
    let (proprias, marcas): (Vec<_>, Vec<_>) =
        refs.iter().partition(|r| r.tipo == TipoReferencia::Propria);

    let mut out = Vec::new();
    if !proprias.is_empty() {
        out.push("REFERENCIAS DA PROPRIA MARCA (material que a peca pode mostrar):".to_string());
        for r in &proprias {
            let nota = if r.nota.trim().is_empty() {
                "sem nota"
            } else {
                r.nota.trim()
            };
            out.push(format!("- {}: {}", r.nome, nota));
        }
    }
    if !marcas.is_empty() {
        if !out.is_empty() {
            out.push(String::new());
        }
        out.push(
            "REFERENCIAS DE ESTILO (trabalho de outras marcas). Use como direcao de \
             linguagem visual. NUNCA copie, cite ou reproduza a marca de terceiro na peca:"
                .to_string(),
        );
        for r in &marcas {
            let nota = if r.nota.trim().is_empty() {
                "sem nota"
            } else {
                r.nota.trim()
            };
            out.push(format!("- {}: {}", r.nome, nota));
        }
    }
    out.join("\n")
}
