//! Postly — gerenciador comercial de publicacoes sociais operado por agentes
//! de IA que rodam inteiramente na maquina do usuario.
//!
//! Mapa dos modulos:
//!
//! - `platform`     padrao Strategy: tudo que muda entre Windows, Linux e macOS.
//! - `hardware`     leitura de memoria e rotina de otimizacao.
//! - `ollama`       catalogo, cliente HTTP e instalacao automatica.
//! - `brain`        grafo de contexto ponderado, serializado e compactado.
//! - `orchestrator` os cargos, os prompts e o pipeline da campanha.
//! - `gemini`       geracao de imagem e legenda.
//! - `browser`      ponte com o sidecar do Playwright.
//! - `vault`        cofre cifrado de chave e credenciais.

pub mod atualizacao;
pub mod auditoria_cmds;
pub mod brain;
pub mod browser;
pub mod claude;
pub mod commands;
pub mod config_cmds;
pub mod falha;
pub mod gemini;
pub mod hardware;
pub mod idioma;
pub mod imagem;
pub mod metricas;
pub mod navegador;
pub mod ollama;
pub mod orchestrator;
pub mod platform;
pub mod prefs;
pub mod referencias;
pub mod state;
pub mod vault;

use state::AppState;

/// Raiz do projeto, onde vive a pasta `sidecar/`. Em desenvolvimento o binario
/// roda em `src-tauri/target/debug`, entao subimos ate encontrar o sidecar.
fn app_root() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf());
        while let Some(candidate) = dir {
            if candidate.join("sidecar/playwright-agent.mjs").exists() {
                return candidate;
            }
            dir = candidate.parent().map(|p| p.to_path_buf());
        }
    }
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let root = app_root();
    // Garante o diretorio de dados antes de qualquer leitura de cofre ou cerebro.
    let _ = std::fs::create_dir_all(platform::current().data_dir());

    tauri::Builder::default()
        .manage(AppState::new(root))
        .invoke_handler(tauri::generate_handler![
            auditoria_cmds::listar_metricas,
            auditoria_cmds::leitura_desempenho,
            auditoria_cmds::registrar_metrica,
            auditoria_cmds::remover_metrica,
            auditoria_cmds::coletar_metricas,
            auditoria_cmds::analisar_desempenho,
            auditoria_cmds::responder_motion,
            auditoria_cmds::responder_limite,
            config_cmds::elenco_claude,
            config_cmds::provedores_de_imagem,
            config_cmds::definir_provedor_imagem,
            config_cmds::salvar_chave_de_imagem,
            config_cmds::testar_provedor_imagem,
            commands::diagnostico,
            commands::memoria,
            commands::computacao,
            commands::sonda_sistema,
            commands::sonda_memoria,
            commands::sonda_acelerador,
            commands::sonda_ollama,
            commands::sonda_navegador,
            commands::verificar_atualizacao,
            commands::instalar_atualizacao,
            commands::provisionar_navegador,
            commands::provisionar_ollama,
            commands::plano_otimizacao,
            commands::otimizar,
            commands::elenco,
            commands::catalogo_modelos,
            commands::baixar_modelo,
            commands::modelos_carregados,
            commands::descarregar_modelos,
            commands::resumo_cofre,
            commands::salvar_chave_gemini,
            commands::validar_chave_gemini,
            commands::esquecer_credenciais,
            commands::iniciar_campanha,
            commands::listar_campanhas,
            commands::pecas_da_campanha,
            commands::ler_markdown,
            commands::abrir_no_sistema,
            commands::fechar_navegador,
            commands::cerebro_stats,
            commands::cerebro_grafo,
            commands::cerebro_node,
            commands::cerebro_travessia,
            commands::cerebro_buscar,
            commands::cerebro_escrever_node,
            commands::cerebro_escrever_aresta,
            commands::cerebro_remover_node,
            commands::cerebro_decair,
            config_cmds::preferencias,
            config_cmds::definir_modo_avancado,
            config_cmds::definir_modelo_do_cargo,
            config_cmds::remover_modelo,
            config_cmds::salvar_referencia,
            config_cmds::remover_referencia,
            config_cmds::anotar_referencia,
            config_cmds::definir_idioma,
            config_cmds::status_provedor,
            config_cmds::definir_provedor,
            config_cmds::salvar_skill,
            config_cmds::remover_skill,
            config_cmds::previa_de_skills,
            config_cmds::salvar_design_system,
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar a janela da Postly");
}
