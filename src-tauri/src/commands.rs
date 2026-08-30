//! Superficie de comandos exposta ao frontend.
//!
//! Nenhum comando aqui contem regra de negocio: cada um traduz um pedido da
//! interface para o modulo que sabe resolver.

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::brain::{Graph, NodeView, ReachedNode};
use crate::hardware::{self, ComputeProfile, OptimizePlan, OptimizeReport, RamSnapshot};
use crate::ollama::{catalog, client, installer};
use crate::orchestrator::{self, roles, CampaignReport, CampaignRequest};
use crate::platform::{self, Platform};
use crate::state::AppState;
use crate::vault;

// ------------------------------------------------------------- diagnostico

#[derive(Debug, Serialize)]
pub struct Diagnostico {
    pub plataforma: Platform,
    pub plataforma_label: &'static str,
    /// RAM, acelerador e os tres tetos de orcamento numa leitura so.
    pub computacao: ComputeProfile,
    pub ollama: installer::OllamaStatus,
    pub node_instalado: bool,
    pub diretorio_dados: String,
    pub redes_suportadas: Vec<RedeInfo>,
}

#[derive(Debug, Serialize)]
pub struct RedeInfo {
    pub slug: &'static str,
    pub label: &'static str,
    pub formato: &'static str,
}

/// Primeiro comando que a interface chama: descobre onde esta rodando e o que
/// a maquina aguenta.
#[tauri::command]
pub async fn diagnostico() -> Diagnostico {
    let strategy = platform::current();
    Diagnostico {
        plataforma: strategy.id(),
        plataforma_label: strategy.label(),
        computacao: hardware::compute_profile(),
        ollama: installer::status().await,
        node_instalado: strategy.node_installed(),
        diretorio_dados: strategy.data_dir().to_string_lossy().to_string(),
        redes_suportadas: [
            roles::Network::Instagram,
            roles::Network::Facebook,
            roles::Network::Tiktok,
            roles::Network::Linkedin,
            roles::Network::X,
        ]
        .iter()
        .map(|n| RedeInfo { slug: n.slug(), label: n.label(), formato: n.format_hint() })
        .collect(),
    }
}

#[tauri::command]
pub async fn memoria() -> RamSnapshot {
    hardware::snapshot()
}

// --- sondas individuais -------------------------------------------------
//
// A tela de preparo roda estas em sequencia em vez de um diagnostico unico.
// Nao e encenacao: cada uma faz trabalho diferente e leva tempo diferente. A
// do acelerador dispara processos externos, a do Ollama fala HTTP, e a do
// sistema so olha o PATH. Separadas, a pessoa ve cada resposta chegar quando
// ela realmente chega.

#[derive(Debug, Serialize)]
pub struct SondaSistema {
    pub plataforma: Platform,
    pub plataforma_label: &'static str,
    pub node_instalado: bool,
    pub diretorio_dados: String,
}

#[tauri::command]
pub async fn sonda_sistema() -> SondaSistema {
    let strategy = platform::current();
    SondaSistema {
        plataforma: strategy.id(),
        plataforma_label: strategy.label(),
        node_instalado: strategy.node_installed(),
        diretorio_dados: strategy.data_dir().to_string_lossy().to_string(),
    }
}

#[tauri::command]
pub async fn sonda_memoria() -> RamSnapshot {
    hardware::snapshot()
}

/// A sonda mais cara: dispara `nvidia-smi`, le sysfs, consulta o sistema.
#[tauri::command]
pub async fn sonda_acelerador() -> ComputeProfile {
    hardware::compute_profile()
}

#[tauri::command]
pub async fn sonda_ollama() -> installer::OllamaStatus {
    installer::status().await
}

/// Releitura completa do hardware, incluindo o acelerador. Mais cara que
/// `memoria`, entao a interface so chama quando o contexto muda de verdade.
#[tauri::command]
pub async fn computacao() -> ComputeProfile {
    hardware::compute_profile()
}

#[tauri::command]
pub async fn provisionar_ollama(app: AppHandle) -> installer::ProvisionReport {
    installer::provision(&app).await
}

// ------------------------------------------------------------- otimizacao

#[tauri::command]
pub async fn plano_otimizacao() -> OptimizePlan {
    hardware::plan()
}

/// Otimizacao em duas frentes: descarrega modelos residentes (o maior ganho
/// imediato) e limpa os caches que o usuario aprovou.
#[tauri::command]
pub async fn otimizar(caminhos: Vec<String>, permitir_elevacao: bool) -> OptimizeReport {
    let descarregados = client::unload_all().await;
    let mut report = hardware::optimize(&caminhos, permitir_elevacao);
    for modelo in descarregados {
        report.actions.insert(0, format!("Modelo {modelo} removido da memoria"));
    }
    report.after = hardware::snapshot();
    report
}

/// Uma vaga do organograma preenchida com o modelo que esta maquina daria a ela.
#[derive(Debug, Serialize)]
pub struct Vaga {
    pub cargo: &'static str,
    pub cargo_label: &'static str,
    pub nivel: roles::Tier,
    pub modelo: Option<&'static str>,
    pub modelo_label: Option<&'static str>,
    pub footprint_bytes: u64,
    pub estimated_tps: f32,
    pub instalado: bool,
    pub moe: bool,
    pub aviso: Option<String>,
}

/// O elenco que a maquina consegue escalar agora.
///
/// Roda a mesma escolha que o pipeline faria, com a memoria livre deste
/// instante. E a ponte entre "sua maquina tem tanto de RAM" e "entao o seu
/// gerente vai ser este modelo, a esta velocidade" — sem ela, a leitura de
/// hardware termina em numero solto.
#[tauri::command]
pub async fn elenco() -> Vec<Vaga> {
    let perfil = hardware::compute_profile();
    let instalados = client::installed_models().await;

    [
        roles::Role::DiretorGeral,
        roles::Role::GerenteSetor,
        roles::Role::Criador,
        roles::Role::Auditor,
    ]
    .iter()
    .map(|cargo| {
        let escolha = catalog::pick(
            cargo.tier(),
            perfil.live_budget_bytes,
            perfil.mode,
            false,
            &instalados,
        );
        match escolha {
            Some((spec, aviso)) => Vaga {
                cargo: cargo.slug(),
                cargo_label: cargo.label(),
                nivel: cargo.tier(),
                modelo: Some(spec.tag),
                modelo_label: Some(spec.label),
                footprint_bytes: spec.footprint_bytes(),
                estimated_tps: hardware::accelerator::estimated_tokens_per_second(
                    perfil.mode,
                    spec.active_params_b,
                ),
                instalado: instalados.iter().any(|t| t == spec.tag),
                moe: spec.moe,
                aviso,
            },
            None => Vaga {
                cargo: cargo.slug(),
                cargo_label: cargo.label(),
                nivel: cargo.tier(),
                modelo: None,
                modelo_label: None,
                footprint_bytes: 0,
                estimated_tps: 0.0,
                instalado: false,
                moe: false,
                aviso: Some(format!(
                    "Nenhum modelo cabe em {} de memoria livre.",
                    hardware::human(perfil.live_budget_bytes)
                )),
            },
        }
    })
    .collect()
}

// ----------------------------------------------------------------- modelos

/// Catalogo da primeira inicializacao. O que e marcado como suportado vem da
/// RAM TOTAL da maquina; a RAM livre so informa o que caberia agora.
#[tauri::command]
pub async fn catalogo_modelos() -> Vec<catalog::CatalogEntry> {
    let perfil = hardware::compute_profile();
    let instalados = client::installed_models().await;
    catalog::build(&perfil, &instalados)
}

#[tauri::command]
pub async fn baixar_modelo(app: AppHandle, tag: String) -> Result<(), String> {
    use tauri::Emitter;
    client::pull(&tag, |progresso| {
        let _ = app.emit("postly://download", progresso);
    })
    .await
}

#[tauri::command]
pub async fn modelos_carregados() -> Vec<client::LoadedModel> {
    client::loaded_models().await
}

#[tauri::command]
pub async fn descarregar_modelos() -> Vec<String> {
    client::unload_all().await
}

// ------------------------------------------------------------------- cofre

#[tauri::command]
pub async fn resumo_cofre() -> vault::VaultSummary {
    vault::summary()
}

#[tauri::command]
pub async fn salvar_chave_gemini(chave: String) -> Result<vault::VaultSummary, String> {
    let mut cofre = vault::load();
    cofre.gemini_api_key = chave.trim().to_string();
    vault::save(&cofre)?;
    Ok(vault::summary())
}

#[tauri::command]
pub async fn validar_chave_gemini() -> Result<String, String> {
    let cofre = vault::load();
    let p = crate::prefs::load().provedor_imagem;
    crate::imagem::validar(p, &cofre.chave_de(p)).await
}

#[tauri::command]
pub async fn esquecer_credenciais(rede: String) -> Result<vault::VaultSummary, String> {
    let mut cofre = vault::load();
    cofre.credentials.remove(&rede);
    vault::save(&cofre)?;
    Ok(vault::summary())
}

// --------------------------------------------------------------- campanha

#[tauri::command]
pub async fn iniciar_campanha(
    app: AppHandle,
    state: State<'_, AppState>,
    pedido: CampaignRequest,
) -> Result<CampaignReport, String> {
    orchestrator::run_campaign(app, state.inner(), pedido).await
}

/// As pecas que uma execucao produziu.
///
/// Separado de `listar_campanhas` porque a lista precisa ser barata: carregar
/// legenda, hashtags e roteiro de todas as campanhas so para desenhar os
/// cabecalhos seria trabalho jogado fora.
#[tauri::command]
pub fn pecas_da_campanha(dir: String) -> Option<orchestrator::transcript::RunResult> {
    orchestrator::transcript::read_result(&dir)
}

#[tauri::command]
pub async fn listar_campanhas() -> Vec<orchestrator::transcript::RunSummary> {
    orchestrator::transcript::list_runs()
}

#[tauri::command]
pub async fn ler_markdown(caminho: String) -> Result<String, String> {
    std::fs::read_to_string(&caminho).map_err(|e| format!("nao consegui ler {caminho}: {e}"))
}

#[tauri::command]
pub async fn abrir_no_sistema(caminho: String) -> Result<(), String> {
    platform::current().open_step(&caminho).run().map(|_| ())
}

#[tauri::command]
pub async fn fechar_navegador(state: State<'_, AppState>) -> Result<(), String> {
    state.browser.shutdown().await;
    Ok(())
}

// ---------------------------------------------------------------- cerebro

#[tauri::command]
pub async fn cerebro_stats(state: State<'_, AppState>) -> Result<crate::brain::store::BrainStats, String> {
    Ok(state.brain.stats().await)
}

/// Grafo inteiro, para o visualizador.
#[tauri::command]
pub async fn cerebro_grafo(state: State<'_, AppState>) -> Result<Graph, String> {
    Ok(state.brain.read(|g| g.clone()).await)
}

/// A consulta que os agentes mais usam: node + vizinhanca ja ordenada por peso,
/// com limiar e top-k aplicados aqui, e nao no modelo.
#[tauri::command]
pub async fn cerebro_node(
    state: State<'_, AppState>,
    id: String,
    peso_minimo: f32,
    top_k: usize,
) -> Result<Option<NodeView>, String> {
    Ok(state.brain.read(|g| g.neighbors(&id, peso_minimo, top_k)).await)
}

#[tauri::command]
pub async fn cerebro_travessia(
    state: State<'_, AppState>,
    origem: String,
    profundidade: u8,
    peso_minimo: f32,
    top_k: usize,
) -> Result<Vec<ReachedNode>, String> {
    Ok(state
        .brain
        .read(|g| g.traverse(&origem, profundidade, peso_minimo, top_k))
        .await)
}

#[tauri::command]
pub async fn cerebro_buscar(
    state: State<'_, AppState>,
    termos: Vec<String>,
    top_k: usize,
) -> Result<Vec<NodeView>, String> {
    Ok(state.brain.read(|g| g.recall(&termos, top_k)).await)
}

#[tauri::command]
pub async fn cerebro_escrever_node(
    state: State<'_, AppState>,
    id: String,
    tipo: String,
    contexto: String,
) -> Result<crate::brain::store::BrainStats, String> {
    state.brain.write(|g| g.upsert_node(&id, &tipo, &contexto)).await?;
    Ok(state.brain.stats().await)
}

#[tauri::command]
pub async fn cerebro_escrever_aresta(
    state: State<'_, AppState>,
    origem: String,
    destino: String,
    tipo: String,
    peso: f32,
) -> Result<crate::brain::store::BrainStats, String> {
    state
        .brain
        .write(|g| g.upsert_edge(&origem, &destino, &tipo, peso))
        .await?;
    Ok(state.brain.stats().await)
}

#[tauri::command]
pub async fn cerebro_remover_node(
    state: State<'_, AppState>,
    id: String,
) -> Result<crate::brain::store::BrainStats, String> {
    state.brain.write(|g| g.remove_node(&id)).await?;
    Ok(state.brain.stats().await)
}

/// Decaimento manual. Sem isto, reforco continuo satura os pesos e a ordenacao
/// da vizinhanca perde sentido.
#[tauri::command]
pub async fn cerebro_decair(state: State<'_, AppState>) -> Result<usize, String> {
    state.brain.write(|g| g.decay()).await
}
