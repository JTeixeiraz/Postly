//! Os comandos da tela de video.
//!
//! Separado do `config_cmds` porque e outro momento de uso: la e configuracao
//! que se faz uma vez, aqui e um trabalho que roda e devolve arquivo.

use tauri::AppHandle;

use crate::video::{assets, narracao, PedidoVideo, RelatorioVideo};

#[tauri::command]
pub fn video_listar() -> Vec<assets::Projeto> {
    assets::listar()
}

#[tauri::command]
pub fn video_criar(nome: String) -> Result<assets::Projeto, String> {
    assets::criar(&nome)
}

/// Grava um arquivo numa das subpastas do projeto.
///
/// Devolve o projeto inteiro relido, e nao so o item gravado: a tela precisa
/// mostrar a pasta de narracao mudando de vazia para cheia no mesmo instante,
/// porque e isso que decide se o video vai parar para perguntar.
#[tauri::command]
pub fn video_adicionar(
    slug: String,
    pasta: String,
    nome: String,
    dados: String,
) -> Result<assets::Projeto, String> {
    assets::adicionar(&slug, &pasta, &nome, &dados)?;
    assets::ler(&slug).ok_or_else(|| {
        crate::idioma::msg(
            "Projeto de video nao encontrado.",
            "Video project not found.",
        )
    })
}

/// Copia arquivos que já estão no disco para dentro do projeto.
///
/// Existe separado do `video_adicionar` porque vídeo não cabe em base64: um
/// arquivo de meio giga viraria 660 MB de string na ponte IPC. O caminho real
/// vem do evento de arrastar-e-soltar do Tauri.
///
/// Erro num arquivo não derruba os outros: quem arrastou cinco vídeos com um
/// formato estranho no meio quer os quatro que servem, com aviso sobre o quinto.
#[tauri::command]
pub fn video_adicionar_caminhos(
    slug: String,
    pasta: String,
    caminhos: Vec<String>,
) -> Result<(assets::Projeto, Vec<String>), String> {
    let mut falhas = Vec::new();
    for c in &caminhos {
        if let Err(e) = assets::adicionar_por_caminho(&slug, &pasta, c) {
            falhas.push(format!("{}: {e}", c.rsplit('/').next().unwrap_or(c)));
        }
    }
    let p = assets::ler(&slug).ok_or_else(|| {
        crate::idioma::msg(
            "Projeto de video nao encontrado.",
            "Video project not found.",
        )
    })?;
    Ok((p, falhas))
}

/// Mede os clipes de um projeto sem montar vídeo nenhum.
///
/// A tela chama isto quando a pessoa sobe um vídeo, para mostrar na hora quanto
/// tempo é pausa. Ver o ganho ANTES de gerar é o que transforma "corta as
/// pausas" numa promessa verificável.
#[tauri::command]
pub async fn video_analisar(
    app: AppHandle,
    estado: tauri::State<'_, crate::state::AppState>,
    slug: String,
) -> Result<Vec<crate::video::analise::Clipe>, String> {
    let projeto = assets::ler(&slug).ok_or_else(|| {
        crate::idioma::msg(
            "Projeto de video nao encontrado.",
            "Video project not found.",
        )
    })?;
    let raiz = estado.app_root.clone();
    crate::video::analise::medir(&app, &raiz, &projeto).await
}

#[tauri::command]
pub fn video_remover_item(slug: String, caminho: String) -> Result<assets::Projeto, String> {
    assets::remover_item(&caminho)?;
    assets::ler(&slug).ok_or_else(|| {
        crate::idioma::msg(
            "Projeto de video nao encontrado.",
            "Video project not found.",
        )
    })
}

#[tauri::command]
pub fn video_remover_projeto(slug: String) -> Result<Vec<assets::Projeto>, String> {
    assets::remover_projeto(&slug)?;
    Ok(assets::listar())
}

#[tauri::command]
pub async fn video_gerar(
    app: AppHandle,
    estado: tauri::State<'_, crate::state::AppState>,
    req: PedidoVideo,
) -> Result<RelatorioVideo, String> {
    let r = crate::video::rodar(app.clone(), &estado, req).await;
    // Toda falha do video passa por aqui, como toda falha de campanha passa
    // pelo `falha.rs`: um caminho unico e o que garante que a tela sempre
    // recebe o motivo, em vez de o erro morrer numa tarefa sem dono.
    if let Err(e) = &r {
        crate::falha::anunciar(
            &app,
            &crate::idioma::msg("Geracao de video", "Video generation"),
            e,
            None,
        );
    }
    r
}

/// A resposta da pessoa a pergunta sobre narracao.
///
/// Devolve `Result` para a tela saber quando o clique chegou tarde — a espera
/// tem teto, e um botao que parece funcionar depois de o video ja ter seguido
/// sem voz seria pior que um botao que diz que perdeu a hora.
#[tauri::command]
pub fn responder_narracao(
    estado: tauri::State<'_, crate::state::AppState>,
    resposta: narracao::RespostaNarracao,
) -> Result<(), String> {
    let vaga = estado
        .resposta_narracao
        .lock()
        .map_err(|_| "estado indisponivel".to_string())?
        .take();
    match vaga {
        Some(tx) => tx.send(resposta).map_err(|_| {
            crate::idioma::msg(
                "O video ja tinha seguido sem esperar.",
                "The video had already moved on without waiting.",
            )
        }),
        None => Err(crate::idioma::msg(
            "Nao ha pergunta de narracao aberta.",
            "There is no open narration question.",
        )),
    }
}
