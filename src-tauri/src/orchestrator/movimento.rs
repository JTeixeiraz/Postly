//! O turno opcional de movimento: quem pede, como se pergunta, e a espera.
//!
//! Sai do orquestrador porque e um assunto fechado que atravessa tres mundos —
//! o texto do gerente, a notificacao do sistema e um clique na janela — e
//! misturar isso ao fluxo principal escondia os tres.

use tauri::{AppHandle, Emitter};

use super::roles::Network;
use super::PecaFinal;
use crate::platform;

/// Junta dois blocos de system, ignorando o segundo quando vazio.
pub fn juntar(base: String, extra: String) -> String {
    if extra.trim().is_empty() {
        base
    } else {
        format!("{base}\n\n{extra}")
    }
}

/// Le a ultima linha do briefing procurando a declaracao de movimento.
///
/// O gerente escreve `MOVIMENTO: sim - <motivo>` ou `MOVIMENTO: nao`. A busca
/// e de tras para frente e case-insensitive porque modelo pequeno as vezes
/// escreve `Movimento:` ou acrescenta uma linha depois; exigir a ultima linha
/// exata faria o recurso falhar em silencio na metade dos modelos do catalogo.
pub fn motivo_do_movimento(briefing: &str) -> Option<String> {
    let linha = briefing
        .lines()
        .rev()
        .find(|l| l.trim_start().to_lowercase().starts_with("movimento:"))?;
    let valor = linha.split_once(':')?.1.trim();
    let baixo = valor.to_lowercase();
    if baixo.starts_with("nao") || baixo.starts_with("não") || baixo.starts_with("no") {
        return None;
    }
    if !baixo.starts_with("sim") && !baixo.starts_with("yes") {
        return None;
    }
    // O motivo vem depois do primeiro travessao ou hifen. Sem motivo, ainda
    // vale como sim: quem decidiu foi o gerente, e a pessoa ainda confirma.
    let motivo = valor
        .split_once(['-', '\u{2013}', '\u{2014}'])
        .map(|(_, m)| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| {
            crate::idioma::msg(
                "O gerente marcou esta peca como candidata a movimento, sem detalhar.",
                "The manager flagged this piece for motion without giving a reason.",
            )
        });
    Some(motivo)
}

/// A peca em texto, do jeito que o motion designer precisa ler.
pub fn descrever_peca(peca: &PecaFinal) -> String {
    format!(
        "Conceito: {}\nLegenda: {}\nChamada para acao: {}\nArte: {}",
        peca.conceito.trim(),
        peca.legenda.trim(),
        peca.chamada_para_acao.trim(),
        peca.prompt_imagem.trim()
    )
}

/// Pergunta a pessoa se entra no turno de motion, e espera a resposta.
///
/// Devolve `false` em qualquer caminho que nao seja um sim explicito: recusa,
/// janela fechada, tempo esgotado. Animar sem confirmacao gastaria minutos de
/// maquina que ninguem pediu.
pub async fn pedir_movimento(
    app: &AppHandle,
    state: &crate::state::AppState,
    rede: Network,
    motivo: &str,
) -> bool {
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    {
        let Ok(mut vaga) = state.resposta_motion.lock() else {
            return false;
        };
        *vaga = Some(tx);
    }

    if let Some(step) = platform::current().notify_step(
        &crate::idioma::msg("Postly: decisao pendente", "Postly: decision pending"),
        &crate::idioma::msg(
            "O gerente pediu animacao para uma peca. Volte ao app para decidir.",
            "The manager asked for motion on a piece. Return to the app to decide.",
        ),
    ) {
        // Notificar nunca pode derrubar a campanha nem faze-la esperar.
        std::thread::spawn(move || {
            let _ = step.run();
        });
    }

    let _ = app.emit(
        "postly://motion",
        serde_json::json!({ "rede": rede.slug(), "motivo": motivo }),
    );

    // Teto de espera: sem ele, uma janela fechada deixaria a campanha parada
    // para sempre segurando o navegador e a pasta da execucao.
    match tokio::time::timeout(std::time::Duration::from_secs(600), rx).await {
        Ok(Ok(sim)) => sim,
        _ => {
            if let Ok(mut vaga) = state.resposta_motion.lock() {
                *vaga = None;
            }
            false
        }
    }
}
