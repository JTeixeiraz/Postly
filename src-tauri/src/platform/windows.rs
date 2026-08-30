//! Estrategia Windows.

use super::{target, Platform, PlatformStrategy, ReclaimTarget, Step};
use crate::hardware::accelerator::{parse_nvidia_smi, parse_wmic_video, Accelerator};
use std::path::PathBuf;

pub struct WindowsStrategy;

impl PlatformStrategy for WindowsStrategy {
    fn id(&self) -> Platform {
        Platform::Windows
    }

    fn label(&self) -> &'static str {
        "Windows"
    }

    fn ollama_binary(&self) -> &'static str {
        "ollama.exe"
    }

    fn ollama_install_steps(&self) -> Vec<Step> {
        // winget ja vem no Windows 10 21H2+ e faz instalacao silenciosa.
        vec![Step::new(
            "Instalar Ollama via winget",
            "winget",
            &[
                "install",
                "--id",
                "Ollama.Ollama",
                "-e",
                "--silent",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ],
        )]
    }

    fn ollama_serve_step(&self) -> Step {
        Step::new("Subir servidor Ollama", "ollama.exe", &["serve"])
    }

    fn node_binary(&self) -> &'static str {
        "node.exe"
    }

    fn npm_binary(&self) -> &'static str {
        "npm.cmd"
    }

    fn npx_binary(&self) -> &'static str {
        "npx.cmd"
    }

    fn data_dir(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Postly")
    }

    fn reclaim_targets(&self) -> Vec<ReclaimTarget> {
        let local = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        let temp = std::env::temp_dir();
        [
            target("Arquivos temporarios do usuario", temp, true),
            target(
                "Cache do Edge",
                local.join("Microsoft/Edge/User Data/Default/Cache"),
                true,
            ),
            target(
                "Cache do Chrome",
                local.join("Google/Chrome/User Data/Default/Cache"),
                true,
            ),
            target("Cache do npm", local.join("npm-cache"), true),
            target(
                "Relatorios de erro do Windows",
                local.join("Microsoft/Windows/WER"),
                true,
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    fn drop_caches_step(&self) -> Option<Step> {
        // Windows nao expoe drop_caches. O equivalente pratico e reduzir o
        // working set dos processos do usuario, o que o proprio SO ja faz sob
        // pressao. Esvaziar a standby list exigiria driver de terceiro (RAMMap),
        // entao aqui nao ha passo seguro a oferecer.
        None
    }

    fn open_step(&self, path: &str) -> Step {
        // `start` e builtin do cmd, entao precisa do cmd para existir.
        Step::new("Abrir no Explorador", "cmd", &["/C", "start", "", path])
    }

    fn notify_step(&self, titulo: &str, corpo: &str) -> Option<Step> {
        // Balao da area de notificacao via WinForms: funciona em qualquer
        // Windows com PowerShell, sem instalar modulo nenhum.
        let limpo = |t: &str| t.replace(['\'', '`', '$'], " ");
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             $n = New-Object System.Windows.Forms.NotifyIcon; \
             $n.Icon = [System.Drawing.SystemIcons]::Information; \
             $n.BalloonTipTitle = '{}'; $n.BalloonTipText = '{}'; \
             $n.Visible = $true; $n.ShowBalloonTip(10000); Start-Sleep -Seconds 6",
            limpo(titulo),
            limpo(corpo)
        );
        Some(Step::new(
            "notificar",
            "powershell",
            &["-NoProfile", "-NonInteractive", "-Command", &script],
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

        // Fallback para AMD e Intel. O Windows relata AdapterRAM num inteiro de
        // 32 bits com sinal, entao qualquer placa acima de 4 GB volta saturada;
        // o parser marca essas como nao utilizaveis de proposito, para o
        // orcamento nunca ser calculado em cima de um numero que mente.
        if placas.is_empty() {
            if let Some(saida) = super::sonda(
                "wmic",
                &["path", "win32_VideoController", "get", "AdapterRAM,Name"],
            ) {
                placas.extend(parse_wmic_video(&saida));
            }
        }

        placas
    }
}
