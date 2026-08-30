//! Estrategia Linux.

use super::{target, Platform, PlatformStrategy, ReclaimTarget, Step};
use crate::hardware::accelerator::{parse_amd_sysfs, parse_nvidia_smi, Accelerator, Vendor};
use std::path::PathBuf;

pub struct LinuxStrategy;

impl PlatformStrategy for LinuxStrategy {
    fn id(&self) -> Platform {
        Platform::Linux
    }

    fn label(&self) -> &'static str {
        "Linux"
    }

    fn ollama_binary(&self) -> &'static str {
        "ollama"
    }

    fn ollama_install_steps(&self) -> Vec<Step> {
        // O instalador oficial cobre qualquer distro; em Arch damos preferencia
        // ao pacote nativo quando o pacman existe.
        if which_exists("pacman") {
            vec![Step::new(
                "Instalar Ollama via pacman",
                "pkexec",
                &["pacman", "-S", "--noconfirm", "ollama"],
            )
            .elevated()]
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
            .join("postly")
    }

    fn reclaim_targets(&self) -> Vec<ReclaimTarget> {
        let cache = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        [
            target(
                "Miniaturas do gerenciador de arquivos",
                cache.join("thumbnails"),
                true,
            ),
            target("Cache do Chromium", cache.join("chromium"), true),
            target("Cache do Google Chrome", cache.join("google-chrome"), true),
            target("Cache do Mozilla", cache.join("mozilla"), true),
            target("Cache do pip", cache.join("pip"), true),
            target("Cache do npm", home.join(".npm/_cacache"), true),
            target(
                "Builds antigos do Cargo",
                home.join(".cargo/registry/cache"),
                true,
            ),
            target(
                "Journal e crash dumps do usuario",
                cache.join("crash"),
                true,
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn drop_caches_step(&self) -> Option<Step> {
        // Devolve page cache / dentries / inodes ao pool livre. Nao perde dado
        // (tudo e regeneravel a partir do disco), mas exige root.
        Some(
            Step::new(
                "Liberar page cache do kernel",
                "pkexec",
                &["sh", "-c", "sync && echo 3 > /proc/sys/vm/drop_caches"],
            )
            .elevated(),
        )
    }

    fn open_step(&self, path: &str) -> Step {
        Step::new("Abrir no gerenciador de arquivos", "xdg-open", &[path])
    }

    fn notify_step(&self, titulo: &str, corpo: &str) -> Option<Step> {
        // notify-send esta em qualquer desktop com libnotify; sem ele, silencio.
        self.which("notify-send")?;
        Some(Step::new(
            "notificar",
            "notify-send",
            &["--app-name=Postly", "--urgency=normal", titulo, corpo],
        ))
    }

    fn detect_accelerators(&self) -> Vec<Accelerator> {
        let mut placas = Vec::new();

        if let Some(saida) = super::sonda(
            "nvidia-smi",
            &[
                "--query-gpu=name,memory.total,memory.free",
                "--format=csv,noheader,nounits",
            ],
        ) {
            placas.extend(parse_nvidia_smi(&saida));
        }

        // AMD: o kernel expoe a VRAM em sysfs sem depender de ferramenta externa.
        // Mas ter a placa nao basta: sem ROCm instalado o Ollama nao descarrega
        // nada nela e a inferencia cai para a CPU de qualquer jeito.
        let rocm = std::path::Path::new("/opt/rocm").exists();
        for card in glob_cards() {
            let total = std::fs::read_to_string(card.join("device/mem_info_vram_total")).ok();
            let usado = std::fs::read_to_string(card.join("device/mem_info_vram_used"))
                .unwrap_or_else(|_| "0".into());
            let Some((total, livre)) = total.and_then(|t| parse_amd_sysfs(&t, &usado)) else {
                continue;
            };
            let nome = std::fs::read_to_string(card.join("device/product_name"))
                .map(|n| n.trim().to_string())
                .unwrap_or_else(|_| "GPU AMD".to_string());
            // VRAM minuscula denuncia grafico integrado compartilhando a RAM.
            let integrada = total < 2 * 1024 * 1024 * 1024;
            placas.push(Accelerator {
                vendor: Vendor::Amd,
                name: nome,
                vram_total_bytes: total,
                vram_free_bytes: livre,
                unified: integrada,
                usable: rocm,
                detail: if rocm {
                    "ROCm encontrado em /opt/rocm.".into()
                } else {
                    "Sem ROCm instalado: o Ollama nao consegue usar esta placa.".into()
                },
            });
        }

        placas
    }
}

fn which_exists(binary: &str) -> bool {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|dir| dir.join(binary).is_file()))
        .unwrap_or(false)
}

/// Lista os cartoes de video expostos pelo kernel em /sys/class/drm.
fn glob_cards() -> Vec<std::path::PathBuf> {
    let Ok(entradas) = std::fs::read_dir("/sys/class/drm") else {
        return Vec::new();
    };
    entradas
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                // "card0" sim, "card0-DP-1" nao: conectores nao tem VRAM.
                .is_some_and(|n| n.starts_with("card") && !n.contains('-'))
        })
        .collect()
}
