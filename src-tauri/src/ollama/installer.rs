//! Deteccao e instalacao automatica do Ollama.
//!
//! Toda a parte que muda entre SOs vem da estrategia de plataforma; aqui fica
//! so a maquina de estados: existe binario? existe servidor no ar? o que falta?

use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::platform;

/// Andamento da instalacao, um evento por linha do instalador.
#[derive(Debug, Clone, Serialize)]
pub struct ProvisionProgress {
    /// Passo atual, a partir de 1.
    pub passo: usize,
    pub total: usize,
    pub label: String,
    /// A linha que o instalador acabou de escrever.
    pub linha: String,
    /// Percentual, quando o instalador informa um. Nem todo passo informa.
    pub percent: Option<f32>,
    pub fase: &'static str,
}

/// Extrai o percentual de uma linha de instalador.
///
/// O script do Ollama usa curl, que escreve "##### 45,2%"; o winget escreve
/// barras com o numero no fim. Nao ha formato comum, entao pegamos o ultimo
/// numero seguido de % que aparecer na linha.
/// Reexportado so para o teste de integracao: a regra de parsing e frageis o
/// bastante para merecer cobertura, e nao ha outro jeito de alcanca-la.
pub fn percentual_publico(linha: &str) -> Option<f32> {
    percentual(linha)
}

fn percentual(linha: &str) -> Option<f32> {
    let bytes = linha.as_bytes();
    let pos = linha.rfind('%')?;
    let mut ini = pos;
    let mut viu_digito = false;
    while ini > 0 {
        let c = bytes[ini - 1] as char;
        if c.is_ascii_digit() {
            viu_digito = true;
            ini -= 1;
        } else if (c == '.' || c == ',') && viu_digito {
            ini -= 1;
        } else {
            break;
        }
    }
    if !viu_digito {
        return None;
    }
    linha[ini..pos]
        .replace(',', ".")
        .parse::<f32>()
        .ok()
        .filter(|p| (0.0..=100.0).contains(p))
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OllamaState {
    /// Binario presente e servidor respondendo.
    Pronto,
    /// Binario presente, mas `ollama serve` nao esta no ar.
    InstaladoParado,
    /// Nao ha binario nesta maquina.
    Ausente,
}

#[derive(Debug, Clone, Serialize)]
pub struct OllamaStatus {
    pub state: OllamaState,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub install_plan: Vec<platform::Step>,
}

pub async fn status() -> OllamaStatus {
    let strategy = platform::current();
    let binary = strategy.which(strategy.ollama_binary());
    let version = super::client::version().await;

    let state = match (&binary, &version) {
        (_, Some(_)) => OllamaState::Pronto,
        (Some(_), None) => OllamaState::InstaladoParado,
        (None, None) => OllamaState::Ausente,
    };

    OllamaStatus {
        state,
        version,
        binary_path: binary.map(|p| p.to_string_lossy().to_string()),
        install_plan: strategy.ollama_install_steps(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvisionReport {
    pub ok: bool,
    pub steps: Vec<String>,
    pub errors: Vec<String>,
    pub final_status: OllamaStatus,
}

/// Garante Ollama instalado E servindo. Instala se faltar, sobe se estiver
/// parado, e so entao devolve o controle.
pub async fn provision(app: &AppHandle) -> ProvisionReport {
    let strategy = platform::current();
    let mut steps = Vec::new();
    let mut errors = Vec::new();

    let avisar = |p: ProvisionProgress| {
        let _ = app.emit("postly://provisao", p);
    };

    if !strategy.ollama_installed() {
        let plano = strategy.ollama_install_steps();
        let total = plano.len() + 1; // +1 para subir o servidor
        for (i, step) in plano.iter().enumerate() {
            let passo = i + 1;
            let label = step.label.clone();
            avisar(ProvisionProgress {
                passo,
                total,
                label: label.clone(),
                linha: crate::idioma::msg("iniciando", "starting"),
                percent: Some(0.0),
                fase: "instalando",
            });
            let resultado = step
                .run_streaming(|linha| {
                    let limpa = linha.trim();
                    if limpa.is_empty() {
                        return;
                    }
                    avisar(ProvisionProgress {
                        passo,
                        total,
                        label: label.clone(),
                        linha: limpa.chars().take(160).collect(),
                        percent: percentual(limpa),
                        fase: "instalando",
                    });
                })
                .await;
            match resultado {
                Ok(_) => steps.push(format!("{} concluido", step.label)),
                Err(e) => errors.push(e),
            }
        }
    } else {
        steps.push(crate::idioma::msg(
            "Ollama ja estava instalado",
            "Ollama was already installed",
        ));
    }

    if super::client::version().await.is_none() {
        avisar(ProvisionProgress {
            passo: 0,
            total: 0,
            label: crate::idioma::msg("Subindo o servidor", "Starting the server"),
            linha: String::new(),
            percent: None,
            fase: "subindo",
        });
        match spawn_server() {
            Ok(()) => steps.push(crate::idioma::msg(
                "Servidor Ollama iniciado",
                "Ollama server started",
            )),
            Err(e) => errors.push(e),
        }
        // O servidor leva alguns instantes para abrir a porta.
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if super::client::version().await.is_some() {
                break;
            }
        }
    }

    let final_status = status().await;
    avisar(ProvisionProgress {
        passo: 0,
        total: 0,
        label: String::new(),
        linha: String::new(),
        percent: Some(100.0),
        fase: "fim",
    });
    ProvisionReport {
        ok: final_status.state == OllamaState::Pronto,
        steps,
        errors,
        final_status,
    }
}

/// Sobe `ollama serve` desacoplado do processo do app: se a janela fechar, o
/// servidor continua, e se ja houver um, este simplesmente falha e some.
fn spawn_server() -> Result<(), String> {
    let step = platform::current().ollama_serve_step();
    std::process::Command::new(&step.program)
        .args(&step.args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("nao consegui subir `{} serve`: {e}", step.program))
}
