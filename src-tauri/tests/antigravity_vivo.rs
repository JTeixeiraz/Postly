//! O provedor Antigravity contra o binário `agy` de verdade desta máquina.
//!
//! Binário próprio, como o `ollama_vivo.rs`, porque fala com serviço externo:
//! assim ele não entra na lista de alvos do CI. E `#[ignore]` porque depende de
//! um `agy` instalado e autenticado — o comando está no cabeçalho de cada
//! teste.
//!
//! O que estes testes provam que os de unidade não provam: que as flags que o
//! provedor monta são aceitas pelo CLI real, que a pasta de trabalho com
//! `tools.core: []` não é recusada, e que o envelope que volta é o que o
//! `achar_envelope` sabe ler. Nenhuma dessas três coisas dá para conferir com
//! uma string escrita à mão.

use postly_lib::antigravity;
use postly_lib::orchestrator::roles::Tier;

/// O binário existe e responde à versão?
///
///   cargo test --test antigravity_vivo -- --ignored --exact acha_o_binario
#[tokio::test]
#[ignore]
async fn acha_o_binario() {
    let caminho = antigravity::localizar().expect("agy nao encontrado nesta maquina");
    println!("agy em: {}", caminho.display());
    let v = antigravity::versao()
        .await
        .expect("o CLI nao respondeu --version");
    println!("versao: {v}");
    assert!(!v.trim().is_empty());
}

/// Um turno de verdade, de ponta a ponta.
///
/// GASTA COTA da conta de quem roda — por isso `#[ignore]`, e por isso o prompt
/// é a menor coisa possível.
///
///   cargo test --test antigravity_vivo -- --ignored --exact um_turno_de_verdade
#[tokio::test]
#[ignore]
async fn um_turno_de_verdade() {
    let r = antigravity::turno(
        Tier::Baixo,
        "Voce responde com uma palavra so, em maiusculas, e nada mais.",
        "Diga OK.",
        180,
    )
    .await;

    match r {
        Ok(t) => {
            println!(
                "resposta: {:?} · {} tokens de entrada, {} de saida",
                t.texto, t.tokens_entrada, t.tokens_saida
            );
            assert!(!t.texto.trim().is_empty());
        }
        // A falha é impressa inteira de propósito: quando este teste falha, o
        // que interessa é a mensagem que a pessoa veria na tela do Postly, não
        // um `assert` dizendo que algo deu errado.
        Err(e) => panic!("o turno nao saiu — a tela mostraria isto:\n  {e}"),
    }
}
