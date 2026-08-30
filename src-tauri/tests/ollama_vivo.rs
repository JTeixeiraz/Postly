//! Testes contra o Ollama de verdade. Se o servidor nao estiver no ar, o teste
//! se declara inconclusivo em vez de falhar: nem toda maquina de CI tem modelo
//! baixado.

use postly_lib::ollama::client::{self, GenerateOptions};

async fn modelo_disponivel() -> Option<String> {
    client::version().await?;
    client::installed_models().await.into_iter().next()
}

#[tokio::test]
async fn o_corpo_da_requisicao_e_aceito_e_a_resposta_nao_vem_vazia() {
    let Some(modelo) = modelo_disponivel().await else {
        eprintln!("Ollama fora do ar ou sem modelo baixado; teste inconclusivo.");
        return;
    };

    let resposta = client::generate(
        &modelo,
        Some("Voce responde com uma unica palavra, sem pontuacao."),
        "Responda: ok",
        GenerateOptions { temperature: 0.1, num_ctx: 2048, num_predict: 32 },
        false,
        // Raciocinio desligado: e o que garante que o orcamento de tokens vai
        // para a resposta e nao para o pensamento.
        false,
        Vec::new(),
    )
    .await
    .expect("o Ollama precisa aceitar o corpo que o cliente monta");

    assert!(
        !resposta.response.trim().is_empty(),
        "resposta veio vazia — provavelmente o raciocinio comeu o orcamento"
    );
    println!(
        "{modelo}: {:?} a {:.2} tok/s",
        resposta.response.trim(),
        resposta.tokens_per_second()
    );
}

#[tokio::test]
async fn o_modelo_sai_da_memoria_depois_de_responder() {
    let Some(modelo) = modelo_disponivel().await else {
        eprintln!("Ollama fora do ar; teste inconclusivo.");
        return;
    };

    let _ = client::generate(
        &modelo,
        None,
        "oi",
        GenerateOptions { temperature: 0.1, num_ctx: 512, num_predict: 8 },
        false,
        false,
        Vec::new(),
    )
    .await;

    // keep_alive: 0 significa que nada pode continuar residente. E essa garantia
    // que permite subir o proximo cargo sem somar memoria.
    let carregados = client::loaded_models().await;
    assert!(
        carregados.is_empty(),
        "ficou modelo residente depois do turno: {:?}",
        carregados.iter().map(|m| &m.name).collect::<Vec<_>>()
    );
}
