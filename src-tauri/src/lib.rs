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
//! - `antigravity`  o Antigravity CLI local (`agy`) como provedor de turno.
//! - `browser`      ponte com o sidecar do Playwright.
//! - `recursos`     onde `sidecar/` e `motion/` vivem na maquina de quem usa.
//! - `vault`        cofre cifrado de chave e credenciais.
//! - `video`        o video avulso: assets, roteiro de cenas e render.

pub mod antigravity;
pub mod atualizacao;
pub mod auditoria_cmds;
pub mod brain;
pub mod browser;
pub mod claude;
pub mod commands;
pub mod config_cmds;
pub mod falha;
pub mod galeria;
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
pub mod recursos;
pub mod referencias;
pub mod state;
pub mod vault;
pub mod video;
pub mod video_cmds;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Garante o diretorio de dados antes de qualquer leitura de cofre ou cerebro.
    let _ = std::fs::create_dir_all(platform::current().data_dir());

    tauri::Builder::default()
        .setup(|app| {
            // A SEMEADURA ACONTECE ANTES DE QUALQUER SONDA, e essa ordem e a
            // correcao de um defeito relatado: sem o `sidecar/` em disco, a
            // tela de Preparacao mostrava o erro de navegador e o
            // provisionamento automatico nao tinha o que instalar.
            use tauri::Manager;
            if let Ok(dir) = app.path().resource_dir() {
                recursos::definir_diretorio_de_recursos(dir);
            }
            if let Err(e) = recursos::semear() {
                // Nao derruba a abertura: o app ainda serve para configurar e
                // ler historico, e a tela de Preparacao vai dizer o que falta
                // com muito mais contexto que um crash na primeira tela.
                eprintln!("falha ao preparar os arquivos de execucao: {e}");
            }
            app.manage(AppState::new(recursos::raiz()));
            Ok(())
        })
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
            config_cmds::elenco_antigravity,
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
            config_cmds::modos_de_desempenho,
            config_cmds::definir_modo,
            config_cmds::galeria_listar,
            config_cmds::galeria_criar,
            config_cmds::galeria_adicionar,
            config_cmds::galeria_remover_item,
            config_cmds::galeria_remover_pasta,
            config_cmds::estado_imagem_local,
            config_cmds::baixar_motor_local,
            config_cmds::baixar_modelo_local,
            config_cmds::remover_modelo_local,
            config_cmds::salvar_design_system,
            video_cmds::video_listar,
            video_cmds::video_criar,
            video_cmds::video_adicionar,
            video_cmds::video_remover_item,
            video_cmds::video_remover_projeto,
            video_cmds::video_gerar,
            video_cmds::responder_narracao,
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar a janela da Postly");
}
