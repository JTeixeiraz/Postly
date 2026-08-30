//! O navegador que publica: presença e instalação.
//!
//! O Playwright chega pelo `npm ci` do sidecar, mas o pacote npm não traz o
//! Chromium — o download é um passo à parte, e ele é preso à versão exata do
//! Playwright. Atualizar a biblioteca sem rodar o passo deixa um estado que
//! parece saudável até alguém clicar em publicar e receber "Executable doesn't
//! exist at .../chromium-1234".
//!
//! Por isso o navegador vira uma sonda da tela de preparação, ao lado do
//! Ollama: se falta, o app instala. Sem ele, metade do produto não existe —
//! não há como publicar nem coletar desempenho.

use crate::platform;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EstadoNavegador {
    /// O Chromium da versão certa está no disco.
    Pronto,
    /// O sidecar existe, mas o Chromium não foi baixado.
    Ausente,
    /// Nem o sidecar foi instalado — `npm ci --prefix sidecar` não rodou.
    SemSidecar,
    /// Node não existe, e sem ele nada disso roda.
    SemNode,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusNavegador {
    pub state: EstadoNavegador,
    /// Caminho do executável, quando encontrado.
    pub caminho: Option<String>,
    pub detalhe: String,
}

fn raiz_sidecar() -> PathBuf {
    // Em desenvolvimento o sidecar mora ao lado do src-tauri; empacotado, ele
    // vai junto do binário.
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../sidecar");
    if dev.join("package.json").exists() {
        return dev;
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("sidecar")))
        .unwrap_or(dev)
}

/// Pergunta ao próprio Playwright onde está o executável.
///
/// Vasculhar `~/.cache/ms-playwright` à mão erraria: o diretório guarda várias
/// versões do Chromium ao mesmo tempo, e só uma serve para a versão instalada
/// da biblioteca. Quem sabe qual é ela é ela mesma.
pub fn status() -> StatusNavegador {
    let strategy = platform::current();
    if !strategy.node_installed() {
        return StatusNavegador {
            state: EstadoNavegador::SemNode,
            caminho: None,
            detalhe: crate::idioma::msg(
                "Node nao encontrado. O navegador que publica roda sobre ele.",
                "Node not found. The browser that publishes runs on top of it.",
            ),
        };
    }

    let raiz = raiz_sidecar();
    if !raiz.join("node_modules/playwright").exists() {
        return StatusNavegador {
            state: EstadoNavegador::SemSidecar,
            caminho: None,
            detalhe: crate::idioma::msg(
                "As dependencias do sidecar nao foram instaladas.",
                "The sidecar dependencies were not installed.",
            ),
        };
    }

    let saida = Command::new(strategy.node_binary())
        .current_dir(&raiz)
        .args([
            "-e",
            "try{const{chromium}=require('playwright');const p=chromium.executablePath();\
             console.log(require('fs').existsSync(p)?p:'')}catch(e){console.log('')}",
        ])
        .output();

    let caminho = saida
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    match caminho {
        Some(p) => StatusNavegador {
            state: EstadoNavegador::Pronto,
            caminho: Some(p),
            detalhe: crate::idioma::msg("Chromium pronto.", "Chromium ready."),
        },
        None => StatusNavegador {
            state: EstadoNavegador::Ausente,
            caminho: None,
            detalhe: crate::idioma::msg(
                "O Chromium ainda nao foi baixado para esta versao do Playwright.",
                "Chromium has not been downloaded for this Playwright version yet.",
            ),
        },
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RelatorioNavegador {
    pub ok: bool,
    pub passos: Vec<String>,
    pub erros: Vec<String>,
    pub status_final: StatusNavegador,
}

/// Instala o que faltar: as dependências do sidecar e o Chromium.
///
/// `install chromium` e não `install`: os outros motores baixariam 400 MB que
/// este projeto nunca usa. `--with-deps` fica de fora de propósito — ele pede
/// sudo no Linux e travaria a instalação num prompt que ninguém vê.
pub async fn provisionar() -> RelatorioNavegador {
    let strategy = platform::current();
    let raiz = raiz_sidecar();
    let mut passos = Vec::new();
    let mut erros = Vec::new();

    if !strategy.node_installed() {
        erros.push(crate::idioma::msg(
            "Instale o Node 18 ou mais novo e tente de novo.",
            "Install Node 18 or newer and try again.",
        ));
        return RelatorioNavegador {
            ok: false,
            passos,
            erros,
            status_final: status(),
        };
    }

    if !raiz.join("node_modules/playwright").exists() {
        passos.push(crate::idioma::msg(
            "Instalando as dependencias do sidecar",
            "Installing sidecar dependencies",
        ));
        match Command::new(strategy.npm_binary())
            .current_dir(&raiz)
            .args(["ci", "--omit=dev"])
            .output()
        {
            Ok(o) if o.status.success() => {}
            Ok(o) => erros.push(format!(
                "npm ci: {}",
                String::from_utf8_lossy(&o.stderr)
                    .lines()
                    .last()
                    .unwrap_or("falhou")
            )),
            Err(e) => erros.push(format!("npm ci: {e}")),
        }
    }

    if erros.is_empty() {
        passos.push(crate::idioma::msg(
            "Baixando o Chromium",
            "Downloading Chromium",
        ));
        match Command::new(strategy.npx_binary())
            .current_dir(&raiz)
            .args(["playwright", "install", "chromium"])
            .output()
        {
            Ok(o) if o.status.success() => {}
            Ok(o) => erros.push(format!(
                "playwright install: {}",
                String::from_utf8_lossy(&o.stderr)
                    .lines()
                    .last()
                    .unwrap_or("falhou")
            )),
            Err(e) => erros.push(format!("playwright install: {e}")),
        }
    }

    let final_ = status();
    RelatorioNavegador {
        ok: final_.state == EstadoNavegador::Pronto,
        passos,
        erros,
        status_final: final_,
    }
}
