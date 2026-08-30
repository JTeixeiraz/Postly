//! Modelos de imagem que rodam na maquina, e o motor que os executa.
//!
//! Um catalogo curto de proposito. Difusao local tem dezenas de modelos e
//! milhares de variantes; oferecer todos seria empurrar a escolha para quem
//! abriu o app para fazer marketing, nao para estudar difusao.
//!
//! O criterio de entrada e um so: cabe numa maquina comum e entrega em poucos
//! passos. Modelo que precisa de 20 passos numa CPU nao e opcao, e um turbo
//! que entrega em 4 passos resolve.

use serde::Serialize;

/// Um modelo de imagem para rodar localmente.
#[derive(Debug, Clone, Serialize)]
pub struct ModeloLocal {
    pub id: &'static str,
    pub nome: &'static str,
    /// Nome do arquivo em disco. E a chave: o que esta baixado e o que existe.
    pub arquivo: &'static str,
    pub url: &'static str,
    pub bytes: u64,
    /// Lado da imagem que o modelo foi treinado para gerar.
    pub base: u32,
    /// Passos recomendados. Turbo entrega em poucos; mandar mais so custa
    /// tempo e costuma piorar.
    pub passos: u32,
    /// Escala de aderencia ao prompt. Turbo pede valores baixos.
    pub cfg: f32,
    pub nota_pt: &'static str,
    pub nota_en: &'static str,
}

const GB: u64 = 1024 * 1024 * 1024;

pub static MODELOS: &[ModeloLocal] = &[
    ModeloLocal {
        id: "sd21-turbo-q4",
        nome: "SD 2.1 Turbo",
        arquivo: "stable-diffusion-v2-1-turbo-Q4_0.gguf",
        url: "https://huggingface.co/gpustack/stable-diffusion-v2-1-turbo-GGUF/resolve/main/stable-diffusion-v2-1-turbo-Q4_0.gguf",
        bytes: 2 * GB + GB / 25,
        base: 512,
        passos: 4,
        cfg: 1.0,
        nota_pt: "O menor do catalogo e o mais rapido numa CPU. Entrega em 4 passos, com qualidade de 2023.",
        nota_en: "The smallest here and the fastest on a CPU. Four steps, with 2023-era quality.",
    },
    ModeloLocal {
        id: "sdxl-turbo-q4",
        nome: "SDXL Turbo",
        arquivo: "stable-diffusion-xl-1.0-turbo-Q4_0.gguf",
        url: "https://huggingface.co/gpustack/stable-diffusion-xl-1.0-turbo-GGUF/resolve/main/stable-diffusion-xl-1.0-turbo-Q4_0.gguf",
        bytes: 3 * GB + GB * 94 / 100,
        base: 512,
        passos: 4,
        cfg: 1.0,
        nota_pt: "Melhor composicao e texto que o SD 2.1, e o dobro do peso. Ainda em 4 passos.",
        nota_en: "Better composition and text than SD 2.1, at twice the size. Still four steps.",
    },
    ModeloLocal {
        id: "sdxl-base-q4",
        nome: "SDXL 1.0",
        arquivo: "stable-diffusion-xl-base-1.0-Q4_0.gguf",
        url: "https://huggingface.co/gpustack/stable-diffusion-xl-base-1.0-GGUF/resolve/main/stable-diffusion-xl-base-1.0-Q4_0.gguf",
        bytes: 3 * GB + GB * 94 / 100,
        base: 1024,
        passos: 20,
        cfg: 7.0,
        nota_pt: "O melhor resultado do catalogo, e o unico que nao e turbo: 20 passos em 1024px. Sem GPU, conte com muitos minutos por imagem.",
        nota_en: "The best output here, and the only non-turbo: 20 steps at 1024px. Without a GPU, expect many minutes per image.",
    },
];

pub fn por_id(id: &str) -> Option<&'static ModeloLocal> {
    MODELOS.iter().find(|m| m.id == id)
}

pub fn por_arquivo(arquivo: &str) -> Option<&'static ModeloLocal> {
    MODELOS.iter().find(|m| m.arquivo == arquivo)
}

/// De onde baixar o motor, por sistema e arquitetura.
///
/// A release do projeto publica um zip por combinacao. Aqui so entram as
/// variantes de CPU: as de CUDA e ROCm passam de 250 MB e exigem driver
/// compativel, o que transformaria "baixar o motor" numa sessao de suporte.
pub fn url_do_motor() -> Option<&'static str> {
    const BASE: &str = "https://github.com/leejet/stable-diffusion.cpp/releases/latest/download";
    let _ = BASE;
    if cfg!(target_os = "windows") {
        Some("win-cpu-x64")
    } else if cfg!(target_os = "macos") {
        Some("Darwin-macOS-arm64")
    } else if cfg!(target_os = "linux") {
        Some("Linux-Ubuntu-24.04-x86_64")
    } else {
        None
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn todo_modelo_tem_arquivo_unico_e_url_coerente() {
        let mut vistos = std::collections::HashSet::new();
        for m in MODELOS {
            assert!(vistos.insert(m.arquivo), "arquivo repetido: {}", m.arquivo);
            assert!(
                m.url.ends_with(m.arquivo),
                "{}: a url nao termina no arquivo",
                m.id
            );
            assert!(m.url.starts_with("https://"), "{}: url insegura", m.id);
        }
    }

    #[test]
    fn turbo_pede_poucos_passos_e_cfg_baixo() {
        // Um turbo com cfg alto devolve imagem queimada, e com 20 passos gasta
        // minutos para piorar. Se alguem adicionar um turbo com os valores de
        // um modelo comum, este teste avisa.
        for m in MODELOS.iter().filter(|m| m.nome.contains("Turbo")) {
            assert!(m.passos <= 8, "{}: turbo com {} passos", m.id, m.passos);
            assert!(m.cfg <= 2.0, "{}: turbo com cfg {}", m.id, m.cfg);
        }
    }

    #[test]
    fn por_arquivo_encontra_o_que_por_id_encontra() {
        for m in MODELOS {
            assert_eq!(por_id(m.id).map(|x| x.arquivo), Some(m.arquivo));
            assert_eq!(por_arquivo(m.arquivo).map(|x| x.id), Some(m.id));
        }
    }
}
