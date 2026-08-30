//! Pastas de produto: os assets de uma marca, organizados para reuso.
//!
//! Antes disto, cada campanha recomecava do zero — a pessoa subia foto por
//! foto, toda vez, para o mesmo produto. A galeria guarda esse material uma
//! vez e a campanha passa a escolher uma pasta.
//!
//! A ESTRUTURA ESPELHA A DISTINCAO QUE O PRODUTO JA FAZ. Cada pasta de
//! produto tem uma subpasta `referencias/` dentro dela:
//!
//!   galeria/cafe-especial/            <- material da marca (entra na peca)
//!   galeria/cafe-especial/referencias <- trabalho de terceiro (so direcao)
//!
//! Nao e organizacao por gosto: mandar a arte de outra marca para o modelo
//! copiar e o caminho mais curto para sair logotipo alheio na peca. A pasta
//! separada torna a confusao dificil de cometer — para errar, e preciso mover
//! o arquivo de proposito.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Nome da subpasta de referencias de terceiros, dentro de cada produto.
const SUB_REFS: &str = "referencias";

/// Extensoes aceitas. A mesma lista das referencias avulsas: o que o modelo de
/// visao consegue ler.
const EXTENSOES: &[&str] = &["png", "jpg", "jpeg", "webp"];

pub fn raiz() -> PathBuf {
    crate::platform::current().data_dir().join("galeria")
}

/// Uma pasta de produto.
#[derive(Debug, Clone, Serialize)]
pub struct Pasta {
    /// Nome de diretorio: e a identidade em disco.
    pub slug: String,
    /// O que a pessoa digitou. Pode ter acento, espaco e maiuscula.
    pub nome: String,
    pub caminho: String,
    /// Assets da propria marca — podem aparecer na peca.
    pub itens: Vec<Item>,
    /// Trabalho de terceiros — so direcao de estilo.
    pub referencias: Vec<Item>,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub nome: String,
    pub caminho: String,
    pub bytes: u64,
}

/// Transforma um nome digitado em nome de diretorio seguro.
///
/// Nao e enfeite: o nome vira caminho no disco, e sem isto um "../" digitado
/// no campo escreveria fora da galeria.
pub fn slugificar(nome: &str) -> String {
    let s: String = nome
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'ê' | 'ë' => 'e',
            'í' | 'ï' => 'i',
            'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ü' => 'u',
            'ç' => 'c',
            c if c.is_ascii_alphanumeric() => c,
            _ => '-',
        })
        .collect();
    // Hifens repetidos viram um; sobra nas pontas some.
    let mut saida = String::with_capacity(s.len());
    let mut ultimo_hifen = true;
    for c in s.chars() {
        if c == '-' {
            if !ultimo_hifen {
                saida.push(c);
            }
            ultimo_hifen = true;
        } else {
            saida.push(c);
            ultimo_hifen = false;
        }
    }
    saida.trim_matches('-').chars().take(64).collect()
}

/// Cria a pasta do produto, ja com a de referencias dentro.
///
/// A subpasta nasce junto e nao sob demanda: uma pasta de referencias que so
/// aparece depois de alguem procurar por ela nao ensina que a separacao
/// existe.
pub fn criar(nome: &str) -> Result<Pasta, String> {
    let slug = slugificar(nome);
    if slug.is_empty() {
        return Err(crate::idioma::msg(
            "De um nome a pasta.",
            "Give the folder a name.",
        ));
    }
    let dir = raiz().join(&slug);
    if dir.exists() {
        return Err(crate::idioma::msg(
            "Ja existe uma pasta com esse nome.",
            "A folder with that name already exists.",
        ));
    }
    std::fs::create_dir_all(dir.join(SUB_REFS)).map_err(|e| e.to_string())?;
    // O nome digitado fica ao lado, porque o slug perde acento e maiuscula.
    let _ = std::fs::write(dir.join(".nome"), nome.trim());
    ler(&slug).ok_or_else(|| "a pasta sumiu logo depois de criada".to_string())
}

/// Todas as pastas, em ordem alfabetica.
pub fn listar() -> Vec<Pasta> {
    let Ok(dir) = std::fs::read_dir(raiz()) else {
        return Vec::new();
    };
    let mut pastas: Vec<Pasta> = dir
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| ler(&e.file_name().to_string_lossy()))
        .collect();
    pastas.sort_by_key(|p| p.nome.to_lowercase());
    pastas
}

pub fn ler(slug: &str) -> Option<Pasta> {
    let slug = slugificar(slug);
    let dir = raiz().join(&slug);
    if !dir.is_dir() {
        return None;
    }
    let nome = std::fs::read_to_string(dir.join(".nome"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| slug.clone());

    let itens = imagens_de(&dir);
    let referencias = imagens_de(&dir.join(SUB_REFS));
    let bytes = itens.iter().chain(&referencias).map(|i| i.bytes).sum();

    Some(Pasta {
        slug,
        nome,
        caminho: dir.to_string_lossy().to_string(),
        itens,
        referencias,
        bytes,
    })
}

fn imagens_de(dir: &Path) -> Vec<Item> {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut itens: Vec<Item> = entradas
        .flatten()
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| EXTENSOES.contains(&x.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| Item {
            nome: e.file_name().to_string_lossy().to_string(),
            caminho: e.path().to_string_lossy().to_string(),
            bytes: e.metadata().map(|m| m.len()).unwrap_or(0),
        })
        .collect();
    itens.sort_by(|a, b| a.nome.cmp(&b.nome));
    itens
}

/// Grava um arquivo enviado pela tela dentro da pasta.
///
/// Recebe base64, como as referencias avulsas: sem o plugin de dialogo do
/// Tauri o navegador nao entrega o caminho real do arquivo, e trazer o plugin
/// so para isto seria peso permanente por conveniencia de uma tela.
pub fn adicionar(
    slug: &str,
    nome: &str,
    base64_dados: &str,
    para_referencias: bool,
) -> Result<Item, String> {
    use base64::Engine;

    let slug = slugificar(slug);
    let dir = raiz().join(&slug);
    if !dir.is_dir() {
        return Err(crate::idioma::msg(
            "Pasta nao encontrada.",
            "Folder not found.",
        ));
    }
    let ext = Path::new(nome)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .filter(|e| EXTENSOES.contains(&e.as_str()))
        .ok_or_else(|| {
            crate::idioma::msg(
                "Formato nao aceito. Use PNG, JPG ou WEBP.",
                "Unsupported format. Use PNG, JPG or WEBP.",
            )
        })?;

    // O navegador manda `data:image/png;base64,AAAA...`; so a cauda interessa.
    let cru = base64_dados
        .split_once(',')
        .map(|(_, resto)| resto)
        .unwrap_or(base64_dados);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(cru.trim())
        .map_err(|e| format!("imagem invalida: {e}"))?;

    const MAX: usize = 6 * 1024 * 1024;
    if bytes.len() > MAX {
        return Err(format!(
            "{} {:.1} MB.",
            crate::idioma::msg(
                "O limite e 6 MB por imagem, e esta tem",
                "The limit is 6 MB per image, and this one is"
            ),
            bytes.len() as f64 / (1024.0 * 1024.0)
        ));
    }

    let destino_dir = if para_referencias {
        dir.join(SUB_REFS)
    } else {
        dir
    };
    std::fs::create_dir_all(&destino_dir).map_err(|e| e.to_string())?;

    let base = Path::new(nome)
        .file_stem()
        .map(|s| slugificar(&s.to_string_lossy()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "imagem".into());
    // Nome livre: dois arquivos chamados `foto.png` nao podem se sobrescrever
    // em silencio.
    let mut destino = destino_dir.join(format!("{base}.{ext}"));
    let mut n = 2;
    while destino.exists() {
        destino = destino_dir.join(format!("{base}-{n}.{ext}"));
        n += 1;
    }
    std::fs::write(&destino, &bytes).map_err(|e| format!("nao consegui gravar: {e}"))?;

    Ok(Item {
        nome: destino
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        caminho: destino.to_string_lossy().to_string(),
        bytes: bytes.len() as u64,
    })
}

/// Apaga um arquivo da pasta.
///
/// Confere que o caminho esta DENTRO da galeria antes de apagar: um caminho
/// vindo da tela nao e confiavel, e um `..` no meio apagaria arquivo de fora.
pub fn remover_item(caminho: &str) -> Result<(), String> {
    let alvo = std::fs::canonicalize(caminho).map_err(|e| e.to_string())?;
    let dentro = std::fs::canonicalize(raiz()).map_err(|e| e.to_string())?;
    if !alvo.starts_with(&dentro) {
        return Err(crate::idioma::msg(
            "Esse arquivo nao esta na galeria.",
            "That file is not in the gallery.",
        ));
    }
    std::fs::remove_file(alvo).map_err(|e| e.to_string())
}

/// Apaga uma pasta inteira, com o que houver dentro.
pub fn remover_pasta(slug: &str) -> Result<(), String> {
    let slug = slugificar(slug);
    if slug.is_empty() {
        return Err("nome invalido".into());
    }
    let dir = raiz().join(&slug);
    if dir.is_dir() {
        std::fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Converte uma pasta nas referencias que a campanha entende.
///
/// E aqui que a estrutura de diretorios vira a distincao do prompt: o que esta
/// na raiz da pasta pode aparecer na peca; o que esta em `referencias/` nunca.
pub fn como_referencias(slug: &str) -> Vec<crate::referencias::Referencia> {
    use crate::referencias::{Referencia, TipoReferencia};
    let Some(p) = ler(slug) else {
        return Vec::new();
    };
    let conv = |i: &Item, tipo: TipoReferencia| Referencia {
        id: i.caminho.clone(),
        nome: i.nome.clone(),
        caminho: i.caminho.clone(),
        bytes: i.bytes,
        tipo,
        nota: String::new(),
    };
    p.itens
        .iter()
        .map(|i| conv(i, TipoReferencia::Propria))
        .chain(p.referencias.iter().map(|i| conv(i, TipoReferencia::Marca)))
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_slug_nao_deixa_escapar_da_galeria() {
        // O nome vira caminho no disco. Sem isto, um "../" digitado no campo
        // escreveria fora da pasta da galeria.
        for perigoso in ["../../etc", "..", "/etc/passwd", "a/../../b", "....//"] {
            let s = slugificar(perigoso);
            assert!(!s.contains('/'), "{perigoso} -> {s}");
            assert!(!s.contains(".."), "{perigoso} -> {s}");
        }
    }

    #[test]
    fn o_slug_preserva_o_que_da_para_ler() {
        assert_eq!(slugificar("Café Especial"), "cafe-especial");
        assert_eq!(slugificar("  Curso de Barista  "), "curso-de-barista");
        assert_eq!(slugificar("Linha 2026 — Verão"), "linha-2026-verao");
        assert_eq!(slugificar("A///B"), "a-b");
    }

    #[test]
    fn nome_que_vira_slug_vazio_e_recusado() {
        // "---" e "..." viram nada depois da limpeza, e uma pasta sem nome
        // seria a propria raiz da galeria.
        for vazio in ["", "   ", "---", "..."] {
            assert!(slugificar(vazio).is_empty(), "{vazio:?} devia virar vazio");
        }
    }
}
