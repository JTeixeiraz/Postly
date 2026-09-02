//! Roda o pipeline de vídeo de ponta a ponta, de verdade, sem depender de clique.
//!
//! POR QUE ISTO EXISTE. A janela do Postly é WebKitGTK, e clique sintético do
//! XTEST não chega nela — está medido na diretriz de verificação do projeto e
//! foi conferido de novo. Sem um caminho como este, a única forma de exercitar
//! o pipeline inteiro (gerente → narração → motion → auditor → render) seria à
//! mão, e um caminho que só se testa à mão é um caminho que para de ser testado.
//!
//! Usa o runtime de VERDADE e o `AppHandle` de verdade, então o que ele prova
//! vale para o app: os mesmos turnos, os mesmos eventos, o mesmo render.
//!
//!   cargo run --example rodar_video -- <slug-do-projeto> "<objetivo>" [proporcao]
//!
//! Roda SEMPRE no Antigravity, explicitamente. Um teste que herda a preferencia
//! global testa a maquina de quem roda, nao o codigo.

use tauri::{Listener, Manager};

fn main() {
    let mut args = std::env::args().skip(1);
    let projeto = args.next().unwrap_or_else(|| "cafe-especial".into());
    let objetivo = args.next().unwrap_or_else(|| {
        "Mostrar o cafe especial da casa para quem ja comprou uma vez, sem parecer anuncio.".into()
    });
    let proporcao = args.next().unwrap_or_else(|| "9:16".into());

    // A MESMA raiz que o app usa, e não uma conta própria a partir do `cwd`.
    // Duas noções de "onde fica o sidecar" foi exatamente o que deixou o
    // defeito do instalador passar despercebido.
    let raiz = postly_lib::recursos::raiz();
    println!("raiz: {}", raiz.display());

    tauri::Builder::default()
        .manage(postly_lib::state::AppState::new(raiz))
        .setup(move |app| {
            let handle = app.handle().clone();

            // A trilha impressa aqui é a mesma que a tela desenha: os eventos
            // vêm do mesmo `postly://estagio`. Se ela ficar muda aqui, fica
            // muda lá.
            handle.listen(postly_lib::orchestrator::agent::EVENT, |e| {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(e.payload()) {
                    println!(
                        "  [{}] {} · {}",
                        v["stage"].as_str().unwrap_or("?"),
                        v["role"].as_str().unwrap_or("?"),
                        v["detail"].as_str().unwrap_or("")
                    );
                }
            });
            handle.listen("postly://render", |e| {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(e.payload()) {
                    println!(
                        "  [render/{}] {:.0}%",
                        v["fase"].as_str().unwrap_or("?"),
                        v["percent"].as_f64().unwrap_or(0.0) * 100.0
                    );
                }
            });

            tauri::async_runtime::spawn(async move {
                let estado = handle.state::<postly_lib::state::AppState>();
                // ANTIGRAVITY EXPLICITO, e nao a preferencia global. Um teste
                // que herda configuracao testa a maquina de quem roda, nao o
                // codigo: no dia em que alguem deixar o Ollama selecionado, a
                // mesma execucao passaria a medir outra coisa sem avisar.
                let pedido = serde_json::json!({
                    "projeto": projeto,
                    "objetivo": objetivo,
                    "proporcao": proporcao,
                    "idioma": "pt",
                    "pensamento_estendido": false,
                    "provedor": "antigravity",
                });
                let req = serde_json::from_value(pedido).expect("pedido");

                println!("\n=== rodando ===");
                match postly_lib::video::rodar(handle.clone(), &estado, req).await {
                    Ok(r) => {
                        println!("\n=== LINHA ===\n{}", r.linha);
                        if let Some(rot) = &r.roteiro {
                            println!("\n=== ROTEIRO ({:.1}s) ===", rot.duracao_s());
                            for (i, c) in rot.cenas.iter().enumerate() {
                                println!(
                                    "  {}. {:?} {:.1}s · {:?}/{:?}/{:?} · {:?} · {}",
                                    i + 1,
                                    c.tipo,
                                    c.dur_s,
                                    c.direcao.movimento,
                                    c.direcao.foco,
                                    c.direcao.pouso,
                                    c.corte
                                        .as_ref()
                                        .map(|k| format!(
                                            "{} {:.1}–{:.1}",
                                            k.arquivo, k.de_s, k.ate_s
                                        ))
                                        .unwrap_or_else(|| format!("{:?}", c.imagens)),
                                    c.titulo
                                );
                            }
                            println!("look: {:?}", rot.look);
                        }
                        println!("\naprovado: {} · rodadas: {}", r.aprovado, r.rodadas);
                        if !r.parecer.is_empty() {
                            println!("parecer: {}", r.parecer);
                        }
                        for a in &r.avisos {
                            println!("aviso: {a}");
                        }
                        match &r.video {
                            Some(v) => println!(
                                "\nVIDEO: {} · {:.1}s · {} bytes",
                                v.arquivo, v.duracao_s, v.bytes
                            ),
                            None => println!("\nsem video"),
                        }
                    }
                    Err(e) => println!("\nFALHOU: {e}"),
                }
                std::process::exit(0);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("app");
}
