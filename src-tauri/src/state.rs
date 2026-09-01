//! Estado compartilhado do processo.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::brain::store::BrainHandle;
use crate::browser::BrowserBridge;

pub struct AppState {
    /// Grafo vivo em memoria; o artefato em disco fica sempre compactado.
    pub brain: BrainHandle,
    /// Ponte com o Playwright. Nao sobe navegador nenhum ate alguem pedir.
    pub browser: BrowserBridge,
    pub app_root: PathBuf,
    /// Canal por onde a campanha, parada, espera a resposta da pessoa sobre
    /// entrar ou nao no turno de motion.
    ///
    /// Mora aqui porque a decisao atravessa dois mundos: quem pergunta e a
    /// campanha rodando em Rust, quem responde e um clique na janela. Um
    /// oneshot e o menor mecanismo que faz a campanha dormir sem ocupar CPU e
    /// acordar exatamente uma vez.
    pub resposta_motion: Mutex<Option<tokio::sync::oneshot::Sender<bool>>>,
    /// A vaga da resposta ao aviso de cota esgotada do Claude Code.
    ///
    /// `true` = esperar a cota voltar e seguir; `false` = encerrar agora. Vive
    /// aqui pelo mesmo motivo da vaga do motion: a campanha roda numa tarefa
    /// e a resposta chega por um comando, que sao dois mundos que so se falam
    /// pelo estado compartilhado.
    pub resposta_limite: Mutex<Option<tokio::sync::oneshot::Sender<bool>>>,
    /// A vaga da resposta sobre narracao, no fluxo de video.
    ///
    /// Vive aqui pelo mesmo motivo das outras duas: quem pergunta e o pipeline
    /// rodando numa tarefa, quem responde e um clique na janela, e os dois so
    /// se falam pelo estado compartilhado.
    pub resposta_narracao:
        Mutex<Option<tokio::sync::oneshot::Sender<crate::video::narracao::RespostaNarracao>>>,
}

impl AppState {
    pub fn new(app_root: PathBuf) -> Self {
        Self {
            brain: BrainHandle::load(),
            browser: BrowserBridge::new(app_root.clone()),
            app_root,
            resposta_motion: Mutex::new(None),
            resposta_limite: Mutex::new(None),
            resposta_narracao: Mutex::new(None),
        }
    }
}
