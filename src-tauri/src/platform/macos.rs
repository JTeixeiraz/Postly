//! Estrategia macOS.

use super::{target, Platform, PlatformStrategy, ReclaimTarget, Step};
use crate::hardware::accelerator::{Accelerator, Vendor};
use std::path::PathBuf;

pub struct MacOsStrategy;

impl PlatformStrategy for MacOsStrategy {
    fn id(&self) -> Platform {
        Platform::MacOS
    }

    fn label(&self) -> &'static str {
        "macOS"
    }

    fn ollama_binary(&self) -> &'static str {
        "ollama"
    }

    fn ollama_install_steps(&self) -> Vec<Step> {
        if std::path::Path::new("/opt/homebrew/bin/brew").exists()
            || std::path::Path::new("/usr/local/bin/brew").exists()
        {
            vec![Step::new(
                "Instalar Ollama via Homebrew",
                "brew",
                &["install", "ollama"],
            )]
        } else {
            vec![Step::new(
                "Instalar Ollama (script oficial)",
                "sh",
                &["-c", "curl -fsSL https://ollama.com/install.sh | sh"],
            )]
        }
    }

    fn ollama_serve_step(&self) -> Step {
        Step::new("Subir servidor Ollama", "ollama", &["serve"])
    }

    fn node_binary(&self) -> &'static str {
        "node"
    }

    fn data_dir(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Postly")
    }

    fn reclaim_targets(&self) -> Vec<ReclaimTarget> {
        let cache = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        [
            target("Caches do usuario", cache.clone(), true),
            target(
                "Cache do Safari",
                home.join("Library/Caches/com.apple.Safari"),
                true,
            ),
            target("Cache do npm", home.join(".npm/_cacache"), true),
            target(
                "Dados derivados do Xcode",
                home.join("Library/Developer/Xcode/DerivedData"),
                true,
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn drop_caches_step(&self) -> Option<Step> {
        // `purge` forca o flush do disk cache inativo de volta para memoria livre.
        Some(Step::new("Liberar cache inativo (purge)", "sudo", &["purge"]).elevated())
    }

    fn open_step(&self, path: &str) -> Step {
        Step::new("Abrir no Finder", "open", &[path])
    }

    fn notify_step(&self, titulo: &str, corpo: &str) -> Option<Step> {
        // As aspas sao removidas do texto antes de entrar no AppleScript: o
        // corpo vem de um modelo, e uma aspa solta viraria erro de sintaxe.
        let limpo = |t: &str| t.replace(['"', '\\'], "");
        let script = format!(
            "display notification \"{}\" with title \"Postly\" subtitle \"{}\"",
            limpo(corpo),
            limpo(titulo)
        );
        Some(Step::new("notificar", "osascript", &["-e", &script]))
    }

    fn detect_accelerators(&self) -> Vec<Accelerator> {
        // Apple Silicon usa memoria unificada: a GPU enxerga a mesma RAM da CPU,
        // sem copia entre elas. Nao existe VRAM separada para somar, e por isso
        // o teto do modelo continua saindo da RAM do sistema.
        let marca = super::sonda("sysctl", &["-n", "machdep.cpu.brand_string"]).unwrap_or_default();
        if !marca.to_lowercase().contains("apple") {
            return Vec::new();
        }

        let ram: u64 = super::sonda("sysctl", &["-n", "hw.memsize"])
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        vec![Accelerator {
            vendor: Vendor::Apple,
            name: marca.trim().to_string(),
            vram_total_bytes: ram,
            vram_free_bytes: 0,
            unified: true,
            usable: true,
            detail: "Memoria unificada: a GPU compartilha a RAM do sistema, sem copia.".into(),
        }]
    }
}
