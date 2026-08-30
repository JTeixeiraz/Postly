//! Leitura de memoria e rotina de otimizacao.
//!
//! Duas nocoes de orcamento convivem aqui e nao devem ser confundidas:
//!
//! - `max_budget_bytes`  -> baseado na RAM que a maquina TEM. E o que define quais
//!   modelos aparecem no catalogo da primeira inicializacao.
//! - `live_budget_bytes` -> baseado na RAM DISPONIVEL agora. E o que decide, a cada
//!   troca de agente, qual modelo pode subir sem derrubar o PC.

pub mod accelerator;

use serde::{Deserialize, Serialize};
use sysinfo::System;

use accelerator::{Accelerator, ComputeMode};

use crate::platform::{self, ReclaimTarget, Step};

/// Fracao da RAM total que aceitamos dedicar a um modelo. O restante fica para
/// SO, navegador do Playwright e a propria janela do app.
const MAX_BUDGET_RATIO: f64 = 0.70;
/// Fracao da RAM livre que aceitamos consumir ao subir um agente agora.
const LIVE_BUDGET_RATIO: f64 = 0.85;
/// Fracao da VRAM que um modelo pode ocupar sem estourar a placa. Mais apertado
/// que o da RAM: quando a VRAM acaba, o driver nao troca para disco, ele falha.
const VRAM_BUDGET_RATIO: f64 = 0.92;
/// Abaixo disto o app mostra o aviso de pouca memoria.
const LOW_RAM_BYTES: u64 = 6 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pressure {
    Ok,
    Apertado,
    Critico,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamSnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    /// Teto para o catalogo (derivado da RAM total).
    pub max_budget_bytes: u64,
    /// Teto para subir um agente agora (derivado da RAM livre).
    pub live_budget_bytes: u64,
    pub pressure: Pressure,
    pub low_ram_warning: bool,
}

pub fn snapshot() -> RamSnapshot {
    let mut sys = System::new();
    sys.refresh_memory();

    let total = sys.total_memory();
    let available = sys.available_memory();
    let swap_total = sys.total_swap();
    let swap_used = sys.used_swap();

    let pressure = if available < LOW_RAM_BYTES / 2 {
        Pressure::Critico
    } else if available < LOW_RAM_BYTES {
        Pressure::Apertado
    } else {
        Pressure::Ok
    };

    RamSnapshot {
        total_bytes: total,
        available_bytes: available,
        used_bytes: total.saturating_sub(available),
        swap_total_bytes: swap_total,
        swap_used_bytes: swap_used,
        max_budget_bytes: (total as f64 * MAX_BUDGET_RATIO) as u64,
        live_budget_bytes: (available as f64 * LIVE_BUDGET_RATIO) as u64,
        pressure,
        low_ram_warning: available < LOW_RAM_BYTES,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHog {
    pub pid: u32,
    pub name: String,
    pub memory_bytes: u64,
}

/// Os processos que mais consomem memoria, para o usuario saber o que fechar.
pub fn top_consumers(limit: usize) -> Vec<ProcessHog> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let mut hogs: Vec<ProcessHog> = sys
        .processes()
        .iter()
        .map(|(pid, proc_)| ProcessHog {
            pid: pid.as_u32(),
            name: proc_.name().to_string_lossy().to_string(),
            memory_bytes: proc_.memory(),
        })
        .filter(|p| p.memory_bytes > 64 * 1024 * 1024)
        .collect();
    hogs.sort_by_key(|p| std::cmp::Reverse(p.memory_bytes));
    hogs.truncate(limit);
    hogs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizePlan {
    pub targets: Vec<ReclaimTarget>,
    pub reclaimable_bytes: u64,
    pub drop_caches: Option<Step>,
    pub hogs: Vec<ProcessHog>,
}

/// Monta o plano sem executar nada. O usuario ve e decide.
pub fn plan() -> OptimizePlan {
    let strategy = platform::current();
    let targets = strategy.reclaim_targets();
    let reclaimable_bytes = targets.iter().map(|t| t.bytes).sum();
    OptimizePlan {
        targets,
        reclaimable_bytes,
        drop_caches: strategy.drop_caches_step(),
        hogs: top_consumers(8),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeReport {
    pub before: RamSnapshot,
    pub after: RamSnapshot,
    pub freed_disk_bytes: u64,
    pub actions: Vec<String>,
    pub failures: Vec<String>,
}

/// Executa a limpeza. `paths` sao alvos que o usuario aprovou; `allow_elevation`
/// libera o passo que pede senha (drop_caches / purge).
pub fn optimize(paths: &[String], allow_elevation: bool) -> OptimizeReport {
    let before = snapshot();
    let mut actions = Vec::new();
    let mut failures = Vec::new();
    let mut freed = 0u64;

    for target in platform::current().reclaim_targets() {
        if !paths.iter().any(|p| p == &target.path) {
            continue;
        }
        if !target.safe {
            failures.push(format!(
                "{}: marcado como destrutivo, ignorado",
                target.label
            ));
            continue;
        }
        match clear_dir_contents(std::path::Path::new(&target.path)) {
            Ok(bytes) => {
                freed += bytes;
                actions.push(format!("{} limpo ({})", target.label, human(bytes)));
            }
            Err(e) => failures.push(format!("{}: {e}", target.label)),
        }
    }

    if allow_elevation {
        if let Some(step) = platform::current().drop_caches_step() {
            match step.run() {
                Ok(_) => actions.push(step.label.clone()),
                Err(e) => failures.push(e),
            }
        }
    }

    OptimizeReport {
        after: snapshot(),
        before,
        freed_disk_bytes: freed,
        actions,
        failures,
    }
}

/// Apaga o CONTEUDO do diretorio, preservando o diretorio em si. Devolve quantos
/// bytes sairam.
fn clear_dir_contents(dir: &std::path::Path) -> Result<u64, String> {
    if !dir.is_dir() {
        return Err("diretorio nao existe mais".into());
    }
    let mut freed = 0u64;
    let entries = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else { continue };
        let size = if meta.is_dir() {
            platform::dir_size(&path, 6)
        } else {
            meta.len()
        };
        let removed = if meta.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        if removed.is_ok() {
            freed += size;
        }
    }
    Ok(freed)
}

pub fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

/// Retrato completo do que esta maquina consegue rodar, e a que velocidade.
///
/// Tres tetos, porque "carrega" e "carrega rapido" sao perguntas diferentes:
///
/// - `max_budget_bytes`         o que a maquina carrega, mesmo que devagar.
/// - `accelerated_budget_bytes` o que cabe inteiro no acelerador.
/// - `live_budget_bytes`        o que cabe neste instante.
#[derive(Debug, Clone, Serialize)]
pub struct ComputeProfile {
    pub ram: RamSnapshot,
    pub accelerators: Vec<Accelerator>,
    pub mode: ComputeMode,
    pub mode_label: &'static str,
    pub primary_name: Option<String>,
    pub vram_total_bytes: u64,
    pub vram_free_bytes: u64,
    pub max_budget_bytes: u64,
    pub accelerated_budget_bytes: u64,
    pub live_budget_bytes: u64,
    /// Sem acelerador, parametros ativos importam mais que tamanho de arquivo.
    pub prefers_moe: bool,
    /// Constante de vazao usada nas estimativas de tokens por segundo.
    pub throughput_constant: f32,
}

pub fn compute_profile() -> ComputeProfile {
    let ram = snapshot();
    let accelerators = crate::platform::current().detect_accelerators();
    let mode = accelerator::modo(&accelerators);
    let primary = accelerator::primaria(&accelerators);

    // Em memoria unificada a "VRAM" e a propria RAM: contar as duas seria contar
    // a mesma memoria duas vezes.
    let (vram_total, vram_free) = match primary {
        Some(a) if !a.unified => (a.vram_total_bytes, a.vram_free_bytes),
        _ => (0, 0),
    };

    let accelerated_budget = match mode {
        ComputeMode::Dedicada => (vram_total as f64 * VRAM_BUDGET_RATIO) as u64,
        // Apple Silicon roda acelerado dentro do mesmo teto da RAM.
        ComputeMode::Unificada => ram.max_budget_bytes,
        ComputeMode::Cpu => 0,
    };

    ComputeProfile {
        mode_label: mode.label(),
        primary_name: primary.map(|a| a.name.clone()),
        vram_total_bytes: vram_total,
        vram_free_bytes: vram_free,
        // Uma placa grande num PC pequeno ainda amplia o que da para carregar.
        max_budget_bytes: ram.max_budget_bytes.max(accelerated_budget),
        accelerated_budget_bytes: accelerated_budget,
        live_budget_bytes: ram.live_budget_bytes.max(if mode == ComputeMode::Dedicada {
            (vram_free as f64 * VRAM_BUDGET_RATIO) as u64
        } else {
            0
        }),
        prefers_moe: mode.prefers_moe(),
        throughput_constant: mode.throughput_constant(),
        accelerators,
        mode,
        ram,
    }
}
