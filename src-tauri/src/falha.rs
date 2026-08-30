//! Falha que interrompe a campanha: avisa na tela e fora dela.
//!
//! Uma campanha leva dezenas de minutos, e a pessoa sai da frente do
//! computador — é para isso que o revezamento existe. Um erro que aparece
//! apenas como uma nota no rodapé da tela é um erro que ninguém vê: quando a
//! pessoa volta, o que ela encontra é a mesma trilha parada de sempre, sem
//! saber se ainda está trabalhando ou se morreu há vinte minutos.
//!
//! Por isso toda falha que PARA a campanha sai por dois canais: um evento que
//! abre o modal, e uma notificação do sistema operacional. O aviso que não
//! interrompe continua no relatório — promovê-lo a modal treinaria a pessoa a
//! fechar modal sem ler.

use crate::platform;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
pub struct Falha {
    /// O que estava acontecendo quando parou, na língua da interface.
    pub etapa: String,
    /// A mensagem crua, que é o que permite investigar.
    pub detalhe: String,
    /// Onde procurar o rastro: a pasta da execução, quando ela chegou a existir.
    pub pasta: Option<String>,
    /// O que fazer a respeito. Vazio quando não há conselho honesto a dar.
    pub sugestao: Option<String>,
}

/// Traduz o erro cru em algo que a pessoa consiga agir em cima.
///
/// A mensagem original continua visível: ela é o que serve para reportar. O
/// que a sugestão faz é evitar que a pessoa tente adivinhar sozinha o que
/// "connection refused" quer dizer.
fn sugerir(detalhe: &str) -> Option<String> {
    let d = detalhe.to_lowercase();
    if d.contains("connection refused") || d.contains("failed to connect") {
        return Some(crate::idioma::msg(
            "O Ollama parou de responder. Volte a Preparacao e suba o servidor.",
            "Ollama stopped responding. Go back to Preparation and start the server.",
        ));
    }
    if d.contains("executable doesn't exist") || d.contains("playwright install") {
        return Some(crate::idioma::msg(
            "O navegador nao esta instalado. A Preparacao tem o botao para baixar.",
            "The browser is not installed. Preparation has the button to download it.",
        ));
    }
    if d.contains("cota") || d.contains("quota") || d.contains("429") {
        return Some(crate::idioma::msg(
            "A cota do gerador de imagem acabou. Troque de servico ou ative faturamento.",
            "The image generator quota ran out. Switch services or enable billing.",
        ));
    }
    if d.contains("chave")
        || d.contains("api key")
        || d.contains("unauthorized")
        || d.contains("401")
    {
        return Some(crate::idioma::msg(
            "A chave foi recusada. Confira em Modelos, no cartao do gerador de imagem.",
            "The key was refused. Check it under Models, in the image generator card.",
        ));
    }
    if d.contains("memoria") || d.contains("memory") || d.contains("cabe") {
        return Some(crate::idioma::msg(
            "Faltou memoria. Feche aplicativos ou use a tela Otimizar.",
            "Not enough memory. Close applications or use the Optimize screen.",
        ));
    }
    None
}

/// Anuncia a falha na tela e fora dela.
pub fn anunciar(app: &AppHandle, etapa: &str, detalhe: &str, pasta: Option<String>) {
    let falha = Falha {
        etapa: etapa.to_string(),
        detalhe: detalhe.to_string(),
        pasta,
        sugestao: sugerir(detalhe),
    };

    if let Some(step) = platform::current().notify_step(
        &crate::idioma::msg("Postly: a campanha parou", "Postly: the campaign stopped"),
        // A notificação do sistema tem espaço para uma linha: ela diz o que
        // aconteceu e manda voltar. O detalhe fica no modal.
        &crate::idioma::msg(
            "Algo deu errado no meio do percurso. Abra o app para ver.",
            "Something went wrong along the way. Open the app to see.",
        ),
    ) {
        // Notificar nunca pode atrasar nem derrubar o relato do erro.
        std::thread::spawn(move || {
            let _ = step.run();
        });
    }

    let _ = app.emit("postly://falha", falha);
}
