//! Deteccao de acelerador e perfil de computacao.
//!
//! Sem isto o sistema so enxerga RAM, e RAM sozinha da a recomendacao errada:
//! numa maquina com GPU dedicada um modelo denso de 27B roda liso, e numa
//! maquina so de CPU o mesmo modelo e inutilizavel. O que muda a resposta nao e
//! quanta memoria existe, e onde a multiplicacao acontece.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Vendor {
    Nvidia,
    Amd,
    Apple,
    Intel,
    Desconhecido,
}

#[derive(Debug, Clone, Serialize)]
pub struct Accelerator {
    pub vendor: Vendor,
    pub name: String,
    pub vram_total_bytes: u64,
    pub vram_free_bytes: u64,
    /// Memoria unificada: a "VRAM" sai da mesma RAM do sistema (Apple Silicon,
    /// graficos integrados). Nao se soma ao total, se divide com ele.
    pub unified: bool,
    /// O Ollama consegue de fato descarregar camadas nesta placa?
    pub usable: bool,
    /// Por que sim ou por que nao, em uma frase.
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputeMode {
    /// GPU dedicada com VRAM propria.
    Dedicada,
    /// Memoria unificada: Apple Silicon, ou integrada com driver utilizavel.
    Unificada,
    /// Tudo na CPU.
    Cpu,
}

impl ComputeMode {
    /// Constante empirica de vazao, em (tokens/s x bilhoes de parametros ativos).
    ///
    /// Dividida pelos parametros ATIVOS do modelo, estima tokens por segundo.
    /// Calibrada em CPU com uma medicao real de 1,2 tok/s num modelo denso de
    /// 14B; os valores de GPU sao ordem de grandeza, nao promessa.
    pub fn throughput_constant(&self) -> f32 {
        match self {
            ComputeMode::Dedicada => 320.0,
            ComputeMode::Unificada => 130.0,
            ComputeMode::Cpu => 17.0,
        }
    }

    /// Sem GPU, o que decide a velocidade sao os parametros ativos por token.
    /// Um MoE de 30B com 3B ativos supera um denso de 14B, apesar de ocupar
    /// mais que o dobro de memoria.
    pub fn prefers_moe(&self) -> bool {
        matches!(self, ComputeMode::Cpu)
    }

    pub fn label(&self) -> &'static str {
        match self {
            ComputeMode::Dedicada => "GPU dedicada",
            ComputeMode::Unificada => "memoria unificada",
            ComputeMode::Cpu => "somente CPU",
        }
    }
}

/// Estimativa de velocidade para um modelo, dado o modo de computacao.
pub fn estimated_tokens_per_second(mode: ComputeMode, active_params_b: f32) -> f32 {
    if active_params_b <= 0.0 {
        return 0.0;
    }
    mode.throughput_constant() / active_params_b
}

// ------------------------------------------------------------------ parsers

/// `nvidia-smi --query-gpu=name,memory.total,memory.free --format=csv,noheader,nounits`
/// devolve uma linha por placa: `NVIDIA GeForce RTX 4070, 12282, 11500`.
pub fn parse_nvidia_smi(saida: &str) -> Vec<Accelerator> {
    saida
        .lines()
        .filter_map(|linha| {
            let campos: Vec<&str> = linha.split(',').map(|c| c.trim()).collect();
            if campos.len() < 3 {
                return None;
            }
            let total_mib: u64 = campos[1].parse().ok()?;
            let livre_mib: u64 = campos[2].parse().ok()?;
            Some(Accelerator {
                vendor: Vendor::Nvidia,
                name: campos[0].to_string(),
                vram_total_bytes: total_mib * 1024 * 1024,
                vram_free_bytes: livre_mib * 1024 * 1024,
                unified: false,
                usable: true,
                detail: "CUDA disponivel; o Ollama descarrega as camadas na placa.".into(),
            })
        })
        .collect()
}

/// Bytes crus de `/sys/class/drm/card*/device/mem_info_vram_total`.
pub fn parse_amd_sysfs(total: &str, usado: &str) -> Option<(u64, u64)> {
    let total: u64 = total.trim().parse().ok()?;
    let usado: u64 = usado.trim().parse().unwrap_or(0);
    Some((total, total.saturating_sub(usado)))
}

/// Uma linha de `wmic path win32_VideoController get name,AdapterRAM`.
///
/// AdapterRAM e um inteiro de 32 bits com sinal, entao ele satura em 4 GB e
/// mente em placas maiores. Serve para identificar a placa, nao para dimensionar
/// o orcamento; por isso o resultado sai marcado como nao utilizavel ate que o
/// caminho do fornecedor confirme a memoria.
pub fn parse_wmic_video(saida: &str) -> Vec<Accelerator> {
    saida
        .lines()
        .skip(1)
        .filter_map(|linha| {
            let linha = linha.trim();
            if linha.is_empty() {
                return None;
            }
            let (bytes_txt, nome) = linha.split_once(char::is_whitespace)?;
            let bytes: u64 = bytes_txt.trim().parse().ok()?;
            let nome = nome.trim();
            if nome.is_empty() {
                return None;
            }
            let vendor = vendor_por_nome(nome);
            Some(Accelerator {
                vendor,
                name: nome.to_string(),
                vram_total_bytes: bytes,
                vram_free_bytes: 0,
                unified: vendor == Vendor::Intel,
                usable: false,
                detail: "Memoria relatada pelo Windows satura em 4 GB; tratada como estimativa."
                    .into(),
            })
        })
        .collect()
}

pub fn vendor_por_nome(nome: &str) -> Vendor {
    let n = nome.to_lowercase();
    if n.contains("nvidia") || n.contains("geforce") || n.contains("quadro") || n.contains("rtx") {
        Vendor::Nvidia
    } else if n.contains("amd") || n.contains("radeon") {
        Vendor::Amd
    } else if n.contains("apple") {
        Vendor::Apple
    } else if n.contains("intel") || n.contains("arc") || n.contains("iris") {
        Vendor::Intel
    } else {
        Vendor::Desconhecido
    }
}

/// Escolhe a placa que vale usar: dedicada e utilizavel na frente, depois
/// unificada, e entre iguais a de maior memoria.
pub fn primaria(lista: &[Accelerator]) -> Option<&Accelerator> {
    lista
        .iter()
        .filter(|a| a.usable)
        .max_by_key(|a| (!a.unified as u8, a.vram_total_bytes))
}

pub fn modo(lista: &[Accelerator]) -> ComputeMode {
    match primaria(lista) {
        Some(a) if !a.unified => ComputeMode::Dedicada,
        Some(_) => ComputeMode::Unificada,
        None => ComputeMode::Cpu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_a_saida_do_nvidia_smi() {
        let placas = parse_nvidia_smi("NVIDIA GeForce RTX 4070, 12282, 11500\n");
        assert_eq!(placas.len(), 1);
        assert_eq!(placas[0].vendor, Vendor::Nvidia);
        assert_eq!(placas[0].vram_total_bytes, 12282 * 1024 * 1024);
        assert!(placas[0].usable && !placas[0].unified);
    }

    #[test]
    fn ignora_linha_truncada_do_nvidia_smi() {
        assert!(parse_nvidia_smi("placa sem memoria\n\n").is_empty());
    }

    #[test]
    fn dedicada_ganha_de_unificada_na_escolha() {
        let lista = vec![
            Accelerator {
                vendor: Vendor::Intel,
                name: "Iris".into(),
                vram_total_bytes: 32 << 30,
                vram_free_bytes: 0,
                unified: true,
                usable: true,
                detail: String::new(),
            },
            Accelerator {
                vendor: Vendor::Nvidia,
                name: "RTX".into(),
                vram_total_bytes: 8 << 30,
                vram_free_bytes: 0,
                unified: false,
                usable: true,
                detail: String::new(),
            },
        ];
        assert_eq!(primaria(&lista).unwrap().name, "RTX");
        assert_eq!(modo(&lista), ComputeMode::Dedicada);
    }

    #[test]
    fn sem_placa_utilizavel_o_modo_e_cpu() {
        let lista = vec![Accelerator {
            vendor: Vendor::Amd,
            name: "iGPU".into(),
            vram_total_bytes: 2 << 30,
            vram_free_bytes: 0,
            unified: true,
            usable: false,
            detail: String::new(),
        }];
        assert_eq!(modo(&lista), ComputeMode::Cpu);
        assert!(primaria(&lista).is_none());
    }

    #[test]
    fn a_estimativa_de_velocidade_premia_parametros_ativos() {
        // 30B com 3B ativos precisa superar um denso de 14B na mesma CPU.
        let moe = estimated_tokens_per_second(ComputeMode::Cpu, 3.0);
        let denso = estimated_tokens_per_second(ComputeMode::Cpu, 14.0);
        assert!(moe > denso * 4.0, "moe {moe} vs denso {denso}");
    }

    #[test]
    fn gpu_dedicada_e_muito_mais_rapida_que_cpu() {
        assert!(
            estimated_tokens_per_second(ComputeMode::Dedicada, 14.0)
                > estimated_tokens_per_second(ComputeMode::Cpu, 14.0) * 10.0
        );
    }
}
