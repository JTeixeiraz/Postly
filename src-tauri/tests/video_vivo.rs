//! O roteiro de vídeo contra um modelo de verdade.
//!
//! Binário próprio e `#[ignore]` porque fala com serviço externo e gasta cota.
//!
//! O QUE ISTO PROVA, E QUE NENHUM TESTE DE UNIDADE PROVA: que um modelo real,
//! lendo os prompts reais, devolve um roteiro que **passa na validação** — com
//! nomes de arquivo que existem, durações em segundos dentro da faixa, e o
//! bloco de direção preenchido. Toda a aposta do formato JSON está aí: se o
//! modelo não conseguir produzir isso, o recurso não existe.
//!
//!   cargo test --test video_vivo -- --ignored --nocapture

use postly_lib::antigravity;
use postly_lib::orchestrator::roles::Tier;
use postly_lib::video::{assets, prompts, spec};

/// Um projeto de mentira, com os nomes de arquivo que o modelo deve citar.
fn projeto() -> assets::Projeto {
    let item = |n: &str| assets::Item {
        nome: n.into(),
        caminho: format!("/tmp/{n}"),
        bytes: 1,
    };
    assets::Projeto {
        slug: "cafe-especial".into(),
        nome: "Café Especial".into(),
        caminho: "/tmp/cafe-especial".into(),
        imagens: vec![item("graos.png"), item("xicara.png"), item("barista.png")],
        clipes: vec![],
        audio: vec![],
        narracao: vec![],
        saidas: vec![],
        bytes: 3,
    }
}

const OBJETIVO: &str =
    "Mostrar o café especial da casa para quem já comprou uma vez, sem parecer anúncio. \
     Vídeo curto, tom caloroso, para o Instagram.";

#[tokio::test]
#[ignore]
async fn o_modelo_devolve_um_roteiro_que_passa_na_validacao() {
    let p = projeto();

    // 1. O gerente decide a linha.
    let linha = antigravity::turno(
        Tier::Alto,
        &postly_lib::idioma::msg(prompts::SYSTEM_GERENTE_PT, prompts::SYSTEM_GERENTE_EN),
        &prompts::prompt_gerente(OBJETIVO, &p, false),
        300,
    )
    .await
    .expect("o turno do gerente falhou");
    println!(
        "\n=== LINHA ({} tokens) ===\n{}",
        linha.tokens_saida, linha.texto
    );

    // 2. O motion designer monta.
    let montagem = antigravity::turno(
        Tier::Medio,
        &postly_lib::idioma::msg(prompts::SYSTEM_MOTION_PT, prompts::SYSTEM_MOTION_EN),
        &prompts::prompt_motion(&linha.texto, &p, "9:16", None, &[]),
        300,
    )
    .await
    .expect("o turno do motion designer falhou");

    // 3. O JSON tem que virar `Roteiro` — este é o ponto do teste.
    let bruto = postly_lib::orchestrator::prompts::extract_json(&montagem.texto)
        .unwrap_or_else(|| panic!("nao veio JSON:\n{}", montagem.texto));
    let roteiro: spec::Roteiro = serde_json::from_value::<spec::Roteiro>(bruto)
        .unwrap_or_else(|e| panic!("JSON nao vira Roteiro: {e}\n{}", montagem.texto))
        .normalizar();

    println!("\n=== ROTEIRO ===");
    println!(
        "proporcao {} · {:.1}s · look {:?}",
        roteiro.proporcao,
        roteiro.duracao_s(),
        roteiro.look
    );
    for (i, c) in roteiro.cenas.iter().enumerate() {
        println!(
            "  {}. {:?} {:.1}s · {:?}/{:?}/{:?} · {:?} · {}",
            i + 1,
            c.tipo,
            c.dur_s,
            c.direcao.movimento,
            c.direcao.foco,
            c.direcao.pouso,
            c.imagens,
            c.titulo
        );
    }

    spec::validar(&roteiro, &p).expect("o roteiro nao passou na validacao");

    // A direção não pode ser a mesma em todas as cenas: seria o template que a
    // camada existe para evitar. Com uma cena só não há o que variar.
    if roteiro.cenas.len() > 2 {
        let movimentos: std::collections::HashSet<_> =
            roteiro.cenas.iter().map(|c| c.direcao.movimento).collect();
        assert!(
            movimentos.len() > 1,
            "todas as {} cenas com o mesmo movimento — o modelo nao dirigiu",
            roteiro.cenas.len()
        );
    }
}
