//! A revisão: o que a pessoa apontou, e como isso volta para o Motion Designer.
//!
//! Isto é TODA a interação que a tela de edição oferece. Ela parece um editor —
//! monitor, régua, trilhas, cabeçote — mas a pessoa não arrasta nada: ela
//! seleciona uma cena, para o vídeo num instante e escreve o que está errado.
//! Arrastar clipe e mexer em keyframe é justamente o trabalho que este programa
//! existe para não pedir; quem fosse fazer isso usaria um editor de verdade.
//!
//! O arquivo junta as três pontas dessa ideia — o tipo que a tela manda, o
//! texto que o cargo recebe e o caminho que pula o gerente — porque elas só
//! fazem sentido juntas e mudam juntas.

use tauri::AppHandle;

use super::montagem::montar;
use super::{assets, spec, PedidoVideo, RelatorioVideo};
use crate::orchestrator::transcript;

/// Uma anotacao da pessoa sobre uma cena do roteiro.
///
/// E TODA a interacao que a tela de edicao oferece, e isso e o desenho: ela
/// APONTA, nao edita. Arrastar clipe e mexer em keyframe e justamente o
/// trabalho que este programa existe para nao pedir — se a pessoa fosse fazer
/// isso, ela usaria um editor de video de verdade.
///
/// A nota carrega a cena E o segundo porque as duas ancoras respondem
/// perguntas diferentes: "a cena 3 esta rapida demais" fala da cena inteira,
/// "aos 4,2s o texto cobre o rosto" fala de um instante. Ter so o indice
/// perderia a segunda; ter so o tempo obrigaria o modelo a descobrir de qual
/// cena a pessoa falava.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NotaDeCena {
    /// Indice da cena, comecando em 1 — e como a tela numera.
    pub cena: usize,
    /// Instante exato em que a pessoa parou o video, quando ela parou.
    #[serde(default)]
    pub segundo: Option<f32>,
    pub texto: String,
}

/// A revisao: o mesmo video, corrigido pelo que a pessoa apontou.
///
/// Pula o gerente e entra direto na montagem, com as notas como correcao e o
/// roteiro anterior como ponto de partida. E o que faz "refazer" devolver o
/// mesmo video melhor em vez de outro video.
pub(super) async fn revisar(
    app: AppHandle,
    run: transcript::RunPaths,
    req: PedidoVideo,
    projeto: assets::Projeto,
    avisos: Vec<String>,
) -> Result<RelatorioVideo, String> {
    let linha = req.linha_anterior.clone();
    let correcoes = bloco_de_notas(&req.notas, req.roteiro_anterior.as_ref());
    montar(app, run, &req, &projeto, linha, Some(correcoes), 0, avisos).await
}

/// As notas da pessoa, viradas em correção para o Motion Designer.
///
/// ANCORADAS NA CENA, e com o texto dela ao lado. Uma nota solta — "está rápido
/// demais" — obriga o modelo a adivinhar de qual cena a pessoa falava, e ele
/// adivinha errado justamente quando o roteiro é longo, que é quando a pessoa
/// mais precisa apontar. Com o título e a duração atual ao lado, a nota vira
/// uma instrução que não depende de memória.
///
/// O roteiro anterior vai INTEIRO logo depois, porque a instrução é corrigir e
/// não recomeçar: sem ele, o modelo montaria um roteiro novo que por acaso
/// atende às notas, e a pessoa receberia outro vídeo em vez do mesmo vídeo
/// melhor.
fn bloco_de_notas(notas: &[NotaDeCena], anterior: Option<&spec::Roteiro>) -> String {
    let mut out = vec![crate::idioma::msg(
        "A PESSOA REVISOU O VIDEO E APONTOU ISTO. Corrija o roteiro abaixo \
         atendendo a cada nota. NAO monte um roteiro novo: mantenha o que nao \
         foi apontado exatamente como esta, inclusive as cenas e a direcao \
         delas.",
        "THE PERSON REVIEWED THE VIDEO AND POINTED THIS OUT. Fix the script \
         below, addressing every note. DO NOT build a new script: keep whatever \
         was not flagged exactly as it is, including the scenes and their \
         direction.",
    )];

    for n in notas {
        // O índice vem da tela numerado a partir de 1; a cena vive num vetor a
        // partir de 0. Converter aqui e não na tela mantém o número que a
        // pessoa vê igual ao que ela clicou.
        let contexto = anterior
            .and_then(|r| r.cenas.get(n.cena.saturating_sub(1)))
            .map(|c| {
                format!(
                    " ({}, {:.1}s{})",
                    if c.titulo.trim().is_empty() {
                        "sem titulo"
                    } else {
                        c.titulo.trim()
                    },
                    c.dur_s,
                    match n.segundo {
                        Some(seg) => format!(", aos {seg:.1}s do video"),
                        None => String::new(),
                    }
                )
            })
            .unwrap_or_default();
        out.push(format!(
            "- {} {}{contexto}: {}",
            crate::idioma::msg("cena", "scene"),
            n.cena,
            n.texto.trim()
        ));
    }

    if let Some(r) = anterior {
        out.push(format!(
            "\n{}\n{}",
            crate::idioma::msg("O ROTEIRO A CORRIGIR:", "THE SCRIPT TO FIX:"),
            serde_json::to_string_pretty(r).unwrap_or_default()
        ));
    }

    out.join("\n")
}
