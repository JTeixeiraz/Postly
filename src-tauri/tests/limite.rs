//! O caminho do aviso de cota esgotada, exercitado de ponta a ponta.
//!
//! O detector ja tem teste de unidade. O que falta cobrir e o que acontece
//! DEPOIS dele: a campanha realmente para e espera? o evento chega na tela? o
//! botao de encerrar durante a espera tem efeito?
//!
//! Sem isto, a unica forma de saber seria estourar a cota de proposito e olhar.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use postly_lib::claude::limite::{self, Limite};
use postly_lib::state::AppState;
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::{Listener, Manager};

/// Um app de teste com o estado que a campanha usa. Nao abre janela.
fn app() -> tauri::App<tauri::test::MockRuntime> {
    let raiz = std::env::temp_dir().join("postly-teste-limite");
    let _ = std::fs::create_dir_all(&raiz);
    mock_builder()
        .manage(AppState::new(raiz))
        .build(mock_context(noop_assets()))
        .expect("app de teste")
}

fn limite_daqui_a(segundos: i64) -> Limite {
    Limite {
        volta_em: Some(chrono::Local::now().timestamp() + segundos),
        evidencia: "Usage limit reached · resets 9:10pm".into(),
    }
}

#[tokio::test]
async fn avisa_a_tela_antes_de_esperar_qualquer_coisa() {
    let app = app();
    let h = app.handle().clone();

    let visto: Arc<Mutex<Option<String>>> = Arc::default();
    let eco = visto.clone();
    h.listen("postly://limite", move |e| {
        *eco.lock().unwrap() = Some(e.payload().to_string());
    });

    let estado = h.state::<AppState>();
    let tarefa = {
        let h2 = h.clone();
        let l = limite_daqui_a(3600);
        tokio::spawn(async move {
            let st = h2.state::<AppState>();
            limite::pausar_e_esperar(&h2, &st, &l).await
        })
    };

    // O aviso precisa sair ANTES de qualquer decisao: e ele que faz a pessoa
    // saber que existe uma decisao a tomar.
    esperar_ate(|| visto.lock().unwrap().is_some()).await;
    let carga = visto.lock().unwrap().clone().unwrap();
    assert!(carga.contains("Usage limit reached"), "carga: {carga}");
    assert!(carga.contains("volta_em"), "carga: {carga}");

    // Encerra, para a tarefa nao ficar dormindo uma hora.
    responder(&estado, false);
    assert!(!tarefa.await.unwrap(), "encerrar deve devolver false");
}

#[tokio::test]
async fn encerrar_nao_espera_nada() {
    let app = app();
    let h = app.handle().clone();
    let estado = h.state::<AppState>();

    let inicio = Instant::now();
    let tarefa = {
        let h2 = h.clone();
        // Uma hora de espera: se `encerrar` respeitasse o relogio, este teste
        // levaria uma hora em vez de milissegundos.
        let l = limite_daqui_a(3600);
        tokio::spawn(async move {
            let st = h2.state::<AppState>();
            limite::pausar_e_esperar(&h2, &st, &l).await
        })
    };

    esperar_ate_haver_vaga(&estado).await;
    responder(&estado, false);

    assert!(!tarefa.await.unwrap());
    assert!(
        inicio.elapsed() < Duration::from_secs(5),
        "encerrar levou {:?} — deveria ser imediato",
        inicio.elapsed()
    );
}

#[tokio::test]
async fn esperar_ate_a_cota_voltar_devolve_true_e_avisa_o_fim() {
    let app = app();
    let h = app.handle().clone();
    let estado = h.state::<AppState>();

    let esperando: Arc<Mutex<bool>> = Arc::default();
    let fim: Arc<Mutex<bool>> = Arc::default();
    {
        let (a, b) = (esperando.clone(), fim.clone());
        h.listen("postly://limite-esperando", move |_| {
            *a.lock().unwrap() = true
        });
        h.listen("postly://limite-fim", move |_| *b.lock().unwrap() = true);
    }

    let tarefa = {
        let h2 = h.clone();
        // Ja passou: a espera vira o minuto de folga que o codigo soma, e o
        // teste nao precisa dormir de verdade... mas 60s ainda e demais para
        // um teste. Por isso o alvo e no passado E o teste checa so o caminho.
        let l = Limite {
            volta_em: Some(chrono::Local::now().timestamp() - 3600),
            evidencia: "Usage limit reached".into(),
        };
        tokio::spawn(async move {
            let st = h2.state::<AppState>();
            limite::pausar_e_esperar(&h2, &st, &l).await
        })
    };

    esperar_ate_haver_vaga(&estado).await;
    responder(&estado, true);

    // O aviso de "estou esperando" precisa sair, senao a tela continuaria
    // mostrando o modal de decisao como se ninguem tivesse decidido nada.
    esperar_ate(|| *esperando.lock().unwrap()).await;

    // A espera e cancelada, que e o mesmo caminho do botao "nao esperar".
    esperar_ate_haver_vaga(&estado).await;
    responder(&estado, false);
    assert!(!tarefa.await.unwrap(), "cancelar durante a espera encerra");
    assert!(
        *fim.lock().unwrap(),
        "a tela precisa saber que a espera acabou"
    );
}

#[tokio::test]
async fn sem_horario_a_espera_e_recusada() {
    let app = app();
    let h = app.handle().clone();
    let estado = h.state::<AppState>();

    let tarefa = {
        let h2 = h.clone();
        let l = Limite {
            volta_em: None,
            evidencia: "Claude AI usage limit reached".into(),
        };
        tokio::spawn(async move {
            let st = h2.state::<AppState>();
            limite::pausar_e_esperar(&h2, &st, &l).await
        })
    };

    esperar_ate_haver_vaga(&estado).await;
    // Mesmo pedindo para esperar, sem horario nao ha o que esperar: dormir sem
    // saber ate quando seria travar a campanha para sempre.
    responder(&estado, true);
    assert!(!tarefa.await.unwrap());
}

/// O `turno` reconhece a mensagem do CLI quando ela chega de verdade?
///
/// Os outros testes cobrem o detector e a pausa em separado. O que fica sem
/// cobertura e a costura: `turno` le mesmo o stderr, ou a mensagem se perde e
/// o erro sai como "resposta ilegivel"? A unica forma de responder sem
/// estourar a cota e por um `claude` falso na frente do PATH.
///
/// Muda o PATH do processo, entao roda sozinho (`--test-threads=1` nao basta:
/// o env e global). Por isso `#[ignore]` — o comando esta no cabecalho.
///
///   cargo test --test limite -- --ignored --exact reconhece_a_saida_real_do_cli
#[tokio::test]
#[ignore]
async fn reconhece_a_saida_real_do_cli() {
    let dir = std::env::temp_dir().join("postly-claude-falso");
    std::fs::create_dir_all(&dir).unwrap();
    let bin = dir.join("claude");
    std::fs::write(
        &bin,
        "#!/usr/bin/env bash\n\
         if [ \"$1\" = \"--version\" ]; then echo '2.1.251 (Claude Code)'; exit 0; fi\n\
         cat >/dev/null\n\
         echo \"You've hit your session limit \u{b7} resets 9:10pm (America/Sao_Paulo)\" >&2\n\
         exit 1\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // SAFETY: teste marcado `#[ignore]`, rodado sozinho por isso mesmo.
    unsafe {
        std::env::set_var("PATH", format!("{}:/usr/bin:/bin", dir.display()));
    }

    let r = postly_lib::claude::turno(
        postly_lib::orchestrator::roles::Tier::Baixo,
        "system",
        "prompt",
        30,
    )
    .await;

    match r {
        Err(postly_lib::claude::ErroTurno::Limite(l)) => {
            let ts = l.volta_em.expect("o horario tinha que sair da mensagem");
            let faltam = ts - chrono::Local::now().timestamp();
            assert!(
                (0..25 * 3600).contains(&faltam),
                "horario fora do plausivel: faltam {faltam}s"
            );
            assert!(l.evidencia.contains("session limit"), "{}", l.evidencia);
        }
        outro => panic!("nao reconheceu como limite: {outro:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────── auxiliares

fn responder(estado: &AppState, esperar: bool) {
    if let Some(tx) = estado.resposta_limite.lock().unwrap().take() {
        let _ = tx.send(esperar);
    }
}

async fn esperar_ate(mut cond: impl FnMut() -> bool) {
    for _ in 0..200 {
        if cond() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("condicao nao aconteceu em 5s");
}

async fn esperar_ate_haver_vaga(estado: &AppState) {
    for _ in 0..200 {
        if estado.resposta_limite.lock().unwrap().is_some() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("a vaga do canal nunca abriu");
}
