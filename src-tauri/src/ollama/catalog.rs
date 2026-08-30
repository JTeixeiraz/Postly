//! Catalogo de modelos locais.
//!
//! Tamanhos conferidos em ollama.com/library (agosto/2026), quantizacao Q4_K_M.
//! `active_params_b` importa mais que `params_b` para velocidade: num MoE so os
//! especialistas ativos passam pela CPU a cada token. Numa maquina sem GPU,
//! qwen3:30b (3B ativos) gera mais rapido que qwen3:14b denso, apesar de ocupar
//! o dobro de RAM.

use serde::Serialize;

use crate::hardware::accelerator::{estimated_tokens_per_second, ComputeMode};
use crate::hardware::ComputeProfile;
use crate::orchestrator::roles::Tier;

/// Sobrecarga de runtime alem do arquivo do modelo: KV cache, contexto,
/// buffers do llama.cpp. Estimativa conservadora para contexto moderado.
pub const RUNTIME_OVERHEAD_BYTES: u64 = 1_400 * 1024 * 1024;

const GB: u64 = 1024 * 1024 * 1024;
const fn gb(x: u64) -> u64 {
    x * GB
}
const fn mb(x: u64) -> u64 {
    x * 1024 * 1024
}

// Sem Deserialize: os campos sao &'static str, o catalogo so viaja para o frontend.
#[derive(Debug, Clone, Serialize)]
pub struct ModelSpec {
    pub tag: &'static str,
    pub family: &'static str,
    pub label: &'static str,
    pub params_b: f32,
    /// Parametros ativos por token (igual a `params_b` em modelos densos).
    pub active_params_b: f32,
    pub moe: bool,
    /// Tamanho do arquivo baixado, sem contar runtime.
    pub weights_bytes: u64,
    pub context_k: u32,
    pub vision: bool,
    /// Pontuacao relativa de capacidade, usada para escolher o melhor que couber.
    pub strength: u16,
    pub tier: Tier,
    /// Familia priorizada pelo usuario (Qwen e Kimi).
    pub focus: bool,
    pub notes: &'static str,
}

impl ModelSpec {
    /// RAM total necessaria para manter este modelo carregado.
    pub fn footprint_bytes(&self) -> u64 {
        self.weights_bytes + RUNTIME_OVERHEAD_BYTES
    }

    pub fn fits(&self, budget_bytes: u64) -> bool {
        self.footprint_bytes() <= budget_bytes
    }

    /// Quao bom este modelo e para o cargo NESTA maquina.
    ///
    /// Capacidade bruta so vale se a resposta chegar. Sem acelerador, um denso
    /// de 27B a 0,6 tok/s leva quase uma hora para um briefing, e um MoE de 30B
    /// com 3B ativos entrega o mesmo trabalho em minutos. Por isso a nota cai
    /// junto com a vazao quando a inferencia e na CPU.
    pub fn rank_score(&self, mode: ComputeMode) -> f32 {
        let bruto = self.strength as f32;
        if mode != ComputeMode::Cpu {
            return bruto;
        }
        let tps = estimated_tokens_per_second(mode, self.active_params_b);
        // Abaixo de 2 tok/s o turno deixa de ser espera e vira abandono.
        bruto * (tps / 2.0).min(1.0)
    }
}

pub static CATALOG: &[ModelSpec] = &[
    // ---------- Qwen 3.8 / 3.6 / 3.5 : geracao atual, cargos de comando ----------
    ModelSpec {
        tag: "qwen3.8:27b", family: "Qwen", label: "Qwen 3.8 27B",
        params_b: 27.0, active_params_b: 27.0, moe: false,
        weights_bytes: gb(18), context_k: 256, vision: true, strength: 97,
        tier: Tier::Alto, focus: true,
        notes: "Mais capaz do catalogo que roda em 32 GB. Aceita imagem na entrada, entao o auditor consegue olhar a arte gerada.",
    },
    ModelSpec {
        tag: "qwen3.6:35b", family: "Qwen", label: "Qwen 3.6 35B",
        params_b: 35.0, active_params_b: 35.0, moe: false,
        weights_bytes: gb(21), context_k: 256, vision: false, strength: 95,
        tier: Tier::Alto, focus: true,
        notes: "Denso e pesado. Em CPU fica lento; prefira se houver GPU dedicada.",
    },
    ModelSpec {
        tag: "qwen3.6:27b", family: "Qwen", label: "Qwen 3.6 27B",
        params_b: 27.0, active_params_b: 27.0, moe: false,
        weights_bytes: gb(17), context_k: 256, vision: false, strength: 92,
        tier: Tier::Alto, focus: true,
        notes: "Otimo raciocinio analitico com pegada menor que o 3.8.",
    },
    ModelSpec {
        tag: "qwen3.5:27b", family: "Qwen", label: "Qwen 3.5 27B",
        params_b: 27.0, active_params_b: 27.0, moe: false,
        weights_bytes: gb(17), context_k: 256, vision: false, strength: 86,
        tier: Tier::Alto, focus: true,
        notes: "Geracao anterior, estavel e bem testada.",
    },
    // ---------- Qwen 3 : MoE, a melhor relacao velocidade/RAM sem GPU ----------
    ModelSpec {
        tag: "qwen3:30b", family: "Qwen", label: "Qwen 3 30B-A3B (MoE)",
        params_b: 30.0, active_params_b: 3.0, moe: true,
        weights_bytes: gb(19), context_k: 256, vision: false, strength: 88,
        tier: Tier::Alto, focus: true,
        notes: "So 3B ativos por token: em CPU gera varias vezes mais rapido que um 14B denso. Melhor escolha para gerente sem GPU.",
    },
    ModelSpec {
        tag: "qwen3-coder:30b", family: "Qwen", label: "Qwen 3 Coder 30B-A3B",
        params_b: 30.0, active_params_b: 3.3, moe: true,
        weights_bytes: gb(19), context_k: 256, vision: false, strength: 84,
        tier: Tier::Alto, focus: true,
        notes: "Afiado em saida estruturada (JSON), util quando o criador precisa devolver schema rigido.",
    },
    ModelSpec {
        tag: "qwen3-vl:30b", family: "Qwen", label: "Qwen 3 VL 30B (visao)",
        params_b: 30.0, active_params_b: 3.0, moe: true,
        weights_bytes: gb(19), context_k: 256, vision: true, strength: 85,
        tier: Tier::Alto, focus: true,
        notes: "Enxerga imagem. Ideal para o auditor conferir a arte antes de aprovar.",
    },
    ModelSpec {
        tag: "qwen3:32b", family: "Qwen", label: "Qwen 3 32B",
        params_b: 32.0, active_params_b: 32.0, moe: false,
        weights_bytes: gb(20), context_k: 128, vision: false, strength: 83,
        tier: Tier::Alto, focus: true,
        notes: "Denso de 32B. Pesado em CPU.",
    },
    ModelSpec {
        tag: "qwen3:14b", family: "Qwen", label: "Qwen 3 14B",
        params_b: 14.0, active_params_b: 14.0, moe: false,
        weights_bytes: mb(9500), context_k: 128, vision: false, strength: 74,
        tier: Tier::Alto, focus: true,
        notes: "Cabe folgado em 32 GB e ja esta instalado nesta maquina.",
    },
    // ---------- Faixa intermediaria : auditor ----------
    ModelSpec {
        tag: "qwen3.5:9b", family: "Qwen", label: "Qwen 3.5 9B",
        params_b: 9.0, active_params_b: 9.0, moe: false,
        weights_bytes: gb(6), context_k: 128, vision: false, strength: 66,
        tier: Tier::Medio, focus: true,
        notes: "Equilibrio de auditoria: critica bem sem custar o que um 27B custa.",
    },
    ModelSpec {
        tag: "qwen3-vl:8b", family: "Qwen", label: "Qwen 3 VL 8B (visao)",
        params_b: 8.0, active_params_b: 8.0, moe: false,
        weights_bytes: mb(6300), context_k: 128, vision: true, strength: 63,
        tier: Tier::Medio, focus: true,
        notes: "Auditor barato que consegue de fato olhar a imagem gerada.",
    },
    ModelSpec {
        tag: "qwen3:8b", family: "Qwen", label: "Qwen 3 8B",
        params_b: 8.0, active_params_b: 8.0, moe: false,
        weights_bytes: mb(5200), context_k: 128, vision: false, strength: 60,
        tier: Tier::Medio, focus: true,
        notes: "Auditor padrao quando a memoria esta apertada.",
    },
    ModelSpec {
        tag: "gemma3:4b", family: "Google", label: "Gemma 3 4B",
        params_b: 4.0, active_params_b: 4.0, moe: false,
        weights_bytes: mb(3300), context_k: 128, vision: true, strength: 50,
        tier: Tier::Medio, focus: false,
        notes: "Alternativa fora da familia Qwen, com visao, para maquinas modestas.",
    },
    // ---------- Faixa leve : criador de conteudo ----------
    ModelSpec {
        tag: "qwen3.5:4b", family: "Qwen", label: "Qwen 3.5 4B",
        params_b: 4.0, active_params_b: 4.0, moe: false,
        weights_bytes: mb(2800), context_k: 128, vision: false, strength: 46,
        tier: Tier::Baixo, focus: true,
        notes: "Executor rapido: recebe briefing pronto e devolve prompt de imagem + legenda.",
    },
    ModelSpec {
        tag: "qwen3-vl:4b", family: "Qwen", label: "Qwen 3 VL 4B (visao)",
        params_b: 4.0, active_params_b: 4.0, moe: false,
        weights_bytes: mb(3500), context_k: 128, vision: true, strength: 44,
        tier: Tier::Baixo, focus: true,
        notes: "Criador leve que enxerga referencias visuais.",
    },
    ModelSpec {
        tag: "qwen3:4b", family: "Qwen", label: "Qwen 3 4B",
        params_b: 4.0, active_params_b: 4.0, moe: false,
        weights_bytes: mb(2600), context_k: 128, vision: false, strength: 40,
        tier: Tier::Baixo, focus: true,
        notes: "Piso confortavel para o cargo de execucao.",
    },
    ModelSpec {
        tag: "qwen3.5:2b", family: "Qwen", label: "Qwen 3.5 2B",
        params_b: 2.0, active_params_b: 2.0, moe: false,
        weights_bytes: mb(1600), context_k: 64, vision: false, strength: 30,
        tier: Tier::Baixo, focus: true,
        notes: "Para maquinas de 8 GB.",
    },
    ModelSpec {
        tag: "qwen3:1.7b", family: "Qwen", label: "Qwen 3 1.7B",
        params_b: 1.7, active_params_b: 1.7, moe: false,
        weights_bytes: mb(1400), context_k: 32, vision: false, strength: 22,
        tier: Tier::Baixo, focus: true,
        notes: "Ultimo recurso sob pressao severa de memoria.",
    },
    ModelSpec {
        tag: "qwen3:0.6b", family: "Qwen", label: "Qwen 3 0.6B",
        params_b: 0.6, active_params_b: 0.6, moe: false,
        weights_bytes: mb(520), context_k: 32, vision: false, strength: 12,
        tier: Tier::Baixo, focus: true,
        notes: "Cabe em qualquer lugar. Qualidade proporcional.",
    },
    // ---------- Outras familias : tags reais do ollama.com/library ----------
    //
    // O catalogo nao pode ser so Qwen. Quem baixa este app tem hardware e gosto
    // proprios, e um MoE da OpenAI ou um denso da Meta pode servir melhor ao
    // cargo dependendo da maquina. Todas as tags abaixo existem na biblioteca
    // publica do Ollama, entao o botao de baixar funciona de verdade.
    ModelSpec {
        tag: "gpt-oss:120b", family: "OpenAI", label: "GPT-OSS 120B (MoE)",
        params_b: 117.0, active_params_b: 5.1, moe: true,
        weights_bytes: gb(65), context_k: 128, vision: false, strength: 98,
        tier: Tier::Alto, focus: false,
        notes: "MoE aberto da OpenAI. So 5,1B ativos por token, entao e rapido para o tamanho, mas os 65 GB de pesos pedem uma estacao de trabalho.",
    },
    ModelSpec {
        tag: "llama3.3:70b", family: "Meta", label: "Llama 3.3 70B",
        params_b: 70.0, active_params_b: 70.0, moe: false,
        weights_bytes: gb(43), context_k: 128, vision: false, strength: 93,
        tier: Tier::Alto, focus: false,
        notes: "Denso de 70B. Precisa de GPU dedicada com bastante VRAM; em CPU e inviavel.",
    },
    ModelSpec {
        tag: "gpt-oss:20b", family: "OpenAI", label: "GPT-OSS 20B (MoE)",
        params_b: 21.0, active_params_b: 3.6, moe: true,
        weights_bytes: gb(14), context_k: 128, vision: false, strength: 88,
        tier: Tier::Alto, focus: false,
        notes: "O melhor custo-beneficio do catalogo para maquina sem GPU: 3,6B ativos por token com qualidade de um modelo muito maior.",
    },
    ModelSpec {
        tag: "deepseek-r1:32b", family: "DeepSeek", label: "DeepSeek R1 32B",
        params_b: 32.0, active_params_b: 32.0, moe: false,
        weights_bytes: gb(20), context_k: 128, vision: false, strength: 90,
        tier: Tier::Alto, focus: false,
        notes: "Raciocinio explicito e forte em analise. Denso: em CPU cada turno leva muito tempo.",
    },
    ModelSpec {
        tag: "gemma3:27b", family: "Google", label: "Gemma 3 27B",
        params_b: 27.0, active_params_b: 27.0, moe: false,
        weights_bytes: gb(17), context_k: 128, vision: true, strength: 87,
        tier: Tier::Alto, focus: false,
        notes: "Enxerga imagem, entao serve bem ao auditor quando ha memoria sobrando.",
    },
    ModelSpec {
        tag: "mistral-small3.2:24b", family: "Mistral", label: "Mistral Small 3.2 24B",
        params_b: 24.0, active_params_b: 24.0, moe: false,
        weights_bytes: gb(15), context_k: 128, vision: true, strength: 85,
        tier: Tier::Alto, focus: false,
        notes: "Bom em seguir instrucao ao pe da letra e escrever em portugues. Aceita imagem.",
    },
    ModelSpec {
        tag: "qwen2.5:32b", family: "Qwen", label: "Qwen 2.5 32B",
        params_b: 32.0, active_params_b: 32.0, moe: false,
        weights_bytes: gb(20), context_k: 128, vision: false, strength: 82,
        tier: Tier::Alto, focus: true,
        notes: "Geracao anterior, ainda solida e muito testada.",
    },
    ModelSpec {
        tag: "gemma3:12b", family: "Google", label: "Gemma 3 12B",
        params_b: 12.0, active_params_b: 12.0, moe: false,
        weights_bytes: mb(8100), context_k: 128, vision: true, strength: 68,
        tier: Tier::Medio, focus: false,
        notes: "Auditor com visao numa faixa de memoria acessivel.",
    },
    ModelSpec {
        tag: "deepseek-r1:14b", family: "DeepSeek", label: "DeepSeek R1 14B",
        params_b: 14.0, active_params_b: 14.0, moe: false,
        weights_bytes: mb(9000), context_k: 128, vision: false, strength: 66,
        tier: Tier::Medio, focus: false,
        notes: "Critica bem estruturada, com raciocinio antes da resposta.",
    },
    ModelSpec {
        tag: "phi4:14b", family: "Microsoft", label: "Phi-4 14B",
        params_b: 14.0, active_params_b: 14.0, moe: false,
        weights_bytes: mb(9100), context_k: 16, vision: false, strength: 64,
        tier: Tier::Medio, focus: false,
        notes: "Forte para o tamanho, mas janela de contexto curta: nao use com briefing longo.",
    },
    ModelSpec {
        tag: "mistral-nemo:12b", family: "Mistral", label: "Mistral Nemo 12B",
        params_b: 12.0, active_params_b: 12.0, moe: false,
        weights_bytes: mb(7100), context_k: 128, vision: false, strength: 61,
        tier: Tier::Medio, focus: false,
        notes: "Multilingue, bom com portugues.",
    },
    ModelSpec {
        tag: "llama3.1:8b", family: "Meta", label: "Llama 3.1 8B",
        params_b: 8.0, active_params_b: 8.0, moe: false,
        weights_bytes: mb(4900), context_k: 128, vision: false, strength: 55,
        tier: Tier::Medio, focus: false,
        notes: "O denso de 8B mais testado que existe. Escolha segura.",
    },
    ModelSpec {
        tag: "granite3.3:8b", family: "IBM", label: "Granite 3.3 8B",
        params_b: 8.0, active_params_b: 8.0, moe: false,
        weights_bytes: mb(4900), context_k: 128, vision: false, strength: 52,
        tier: Tier::Medio, focus: false,
        notes: "Licenca permissiva e treino voltado a uso corporativo.",
    },
    ModelSpec {
        tag: "llama3.2:3b", family: "Meta", label: "Llama 3.2 3B",
        params_b: 3.0, active_params_b: 3.0, moe: false,
        weights_bytes: mb(2000), context_k: 128, vision: false, strength: 38,
        tier: Tier::Baixo, focus: false,
        notes: "Executor leve e rapido, com boa aderencia a formato.",
    },
    ModelSpec {
        tag: "phi4-mini:3.8b", family: "Microsoft", label: "Phi-4 Mini 3.8B",
        params_b: 3.8, active_params_b: 3.8, moe: false,
        weights_bytes: mb(2500), context_k: 128, vision: false, strength: 36,
        tier: Tier::Baixo, focus: false,
        notes: "Cumpre briefing curto sem gastar memoria.",
    },
    ModelSpec {
        tag: "granite3.3:2b", family: "IBM", label: "Granite 3.3 2B",
        params_b: 2.0, active_params_b: 2.0, moe: false,
        weights_bytes: mb(1500), context_k: 128, vision: false, strength: 28,
        tier: Tier::Baixo, focus: false,
        notes: "Para maquinas modestas, com licenca Apache 2.0.",
    },
    // ---------- Kimi : pedido pelo usuario, mas fora do alcance de 32 GB ----------
    ModelSpec {
        tag: "kimi-k2.6:1t", family: "Moonshot", label: "Kimi K2.6 1T (MoE)",
        params_b: 1000.0, active_params_b: 32.0, moe: true,
        weights_bytes: gb(600), context_k: 256, vision: true, strength: 100,
        tier: Tier::Alto, focus: true,
        notes: "Lider em benchmark, mas 1T de parametros: ~600 GB de memoria. No Ollama e servido apenas na nuvem. Nao roda local em 32 GB.",
    },
    ModelSpec {
        tag: "kimi-k2.7-code", family: "Moonshot", label: "Kimi K2.7 Code",
        params_b: 1000.0, active_params_b: 32.0, moe: true,
        weights_bytes: gb(600), context_k: 256, vision: false, strength: 99,
        tier: Tier::Alto, focus: true,
        notes: "Mesma classe do K2.6, focado em codigo. Tambem inviavel localmente em 32 GB.",
    },
];

#[derive(Debug, Clone, Serialize)]
pub struct CatalogEntry {
    #[serde(flatten)]
    pub spec: ModelSpec,
    pub footprint_bytes: u64,
    /// Tokens por segundo estimados nesta maquina. Estimativa, nao promessa.
    pub estimated_tps: f32,
    /// Cabe inteiro no acelerador, sem cair para a CPU no meio.
    pub accelerated: bool,
    /// O hardware desta maquina suporta o modelo na sua capacidade MAXIMA?
    pub supported: bool,
    /// Ja esta baixado no Ollama local?
    pub installed: bool,
    /// Cabe na memoria livre AGORA?
    pub fits_now: bool,
    pub reason: String,
}

/// Monta a lista que a tela de modelos exibe.
///
/// `max_budget` vem da RAM TOTAL (capacidade maxima da maquina) e decide o que
/// e marcado como suportado. `live_budget` vem da RAM livre e so informa o que
/// caberia neste instante.
pub fn build(profile: &ComputeProfile, installed: &[String]) -> Vec<CatalogEntry> {
    let max_budget = profile.max_budget_bytes;
    let live_budget = profile.live_budget_bytes;

    let mut entries: Vec<CatalogEntry> = CATALOG
        .iter()
        .map(|spec| {
            let footprint = spec.footprint_bytes();
            let supported = footprint <= max_budget;
            let accelerated = profile.accelerated_budget_bytes > 0
                && footprint <= profile.accelerated_budget_bytes;
            let tps = estimated_tokens_per_second(
                if accelerated {
                    profile.mode
                } else {
                    ComputeMode::Cpu
                },
                spec.active_params_b,
            );

            let reason = if !supported {
                format!(
                    "Precisa de {} de memoria; o teto desta maquina e {}.",
                    crate::hardware::human(footprint),
                    crate::hardware::human(max_budget)
                )
            } else if footprint > live_budget {
                format!(
                    "Cabe na capacidade da maquina, mas nao no que esta livre agora ({}).",
                    crate::hardware::human(live_budget)
                )
            } else if !accelerated && profile.mode != ComputeMode::Cpu {
                format!(
                    "Nao cabe inteiro na {}; parte roda na CPU e a velocidade cai.",
                    profile.mode_label
                )
            } else if tps < 1.0 {
                "Roda, mas devagar demais para uso pratico nesta maquina.".to_string()
            } else {
                "Pronto para uso.".to_string()
            };

            CatalogEntry {
                footprint_bytes: footprint,
                estimated_tps: tps,
                accelerated,
                supported,
                installed: installed.iter().any(|t| tag_matches(t, spec.tag)),
                fits_now: footprint <= live_budget,
                reason,
                spec: spec.clone(),
            }
        })
        .collect();

    // Suportados primeiro; entre eles, o que serve melhor ESTA maquina.
    entries.sort_by(|a, b| {
        b.supported.cmp(&a.supported).then(
            b.spec
                .rank_score(profile.mode)
                .partial_cmp(&a.spec.rank_score(profile.mode))
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });
    entries
}

/// `ollama list` devolve `qwen3:14b`; o catalogo guarda o mesmo formato, mas
/// toleramos a ausencia do sufixo `:latest`.
fn tag_matches(installed: &str, catalog_tag: &str) -> bool {
    installed == catalog_tag
        || installed.trim_end_matches(":latest") == catalog_tag
        || installed == format!("{catalog_tag}:latest")
}

/// Escolhe o modelo mais capaz de um cargo que caiba no orcamento informado.
///
/// Rebaixa de nivel quando nada do nivel pedido cabe, e devolve junto o aviso
/// para o sistema registrar que houve degradacao.
pub fn pick(
    tier: Tier,
    budget_bytes: u64,
    mode: ComputeMode,
    installed_only: bool,
    installed: &[String],
) -> Option<(&'static ModelSpec, Option<String>)> {
    for (tentativa, nivel) in tier.degradation_path().iter().enumerate() {
        let mut opcoes: Vec<&'static ModelSpec> = CATALOG
            .iter()
            .filter(|m| m.tier == *nivel)
            .filter(|m| m.fits(budget_bytes))
            .filter(|m| !installed_only || installed.iter().any(|t| tag_matches(t, m.tag)))
            .collect();

        opcoes.sort_by(|a, b| {
            b.rank_score(mode)
                .partial_cmp(&a.rank_score(mode))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(melhor) = opcoes.first() {
            let aviso = if tentativa > 0 {
                Some(format!(
                    "Sem memoria para um modelo de nivel {:?}; rebaixado para {:?} ({}).",
                    tier, nivel, melhor.label
                ))
            } else {
                None
            };
            return Some((melhor, aviso));
        }
    }
    None
}
