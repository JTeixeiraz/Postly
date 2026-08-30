//! Testes do grafo de contexto. As garantias verificadas aqui sao as que o
//! resto do sistema assume sem checar: ordenacao por peso, corte por limiar,
//! peso efetivo como produto do caminho, e teto de deslocamento por interacao.

use postly_lib::brain::Graph;

fn grafo_exemplo() -> Graph {
    let mut g = Graph::new();
    g.upsert_node("api", "repositorio", "Camada intermediaria.");
    g.upsert_node("politica", "regra", "Criterio de aceite obrigatorio.");
    g.upsert_node("schema", "contrato", "Formato das demandas.");
    g.upsert_node("runner", "infra", "Executor do CI.");
    g.upsert_node("longe", "isolado", "Alcancavel so via runner.");

    g.upsert_edge("api", "politica", "aplica", 0.92);
    g.upsert_edge("api", "schema", "depende_de", 0.81);
    g.upsert_edge("api", "runner", "citado_junto", 0.34);
    g.upsert_edge("runner", "longe", "dispara", 0.60);
    g
}

#[test]
fn vizinhanca_volta_ordenada_por_peso() {
    let g = grafo_exemplo();
    let vista = g.neighbors("api", 0.0, 10).expect("node existe");
    let pesos: Vec<f32> = vista.neighbors.iter().map(|n| n.weight).collect();

    assert_eq!(vista.neighbors[0].node, "politica");
    assert!(
        pesos.windows(2).all(|par| par[0] >= par[1]),
        "vizinhos precisam vir do maior para o menor peso, veio {pesos:?}"
    );
}

#[test]
fn limiar_corta_na_consulta_e_nao_no_modelo() {
    let g = grafo_exemplo();
    let vista = g.neighbors("api", 0.5, 10).unwrap();
    let nomes: Vec<&str> = vista.neighbors.iter().map(|n| n.node.as_str()).collect();

    assert!(nomes.contains(&"politica"));
    assert!(nomes.contains(&"schema"));
    assert!(
        !nomes.contains(&"runner"),
        "0.34 esta abaixo do limiar 0.5 e nao pode passar"
    );
}

#[test]
fn top_k_limita_a_vizinhanca() {
    let g = grafo_exemplo();
    assert_eq!(g.neighbors("api", 0.0, 2).unwrap().neighbors.len(), 2);
}

#[test]
fn peso_efetivo_e_o_produto_do_caminho() {
    let g = grafo_exemplo();
    let alcancados = g.traverse("api", 2, 0.01, 10);
    let longe = alcancados
        .iter()
        .find(|n| n.node == "longe")
        .expect("alcancavel em 2 saltos");

    // api -> runner (0.34) -> longe (0.60)
    let esperado = 0.34_f32 * 0.60;
    assert!(
        (longe.effective_weight - esperado).abs() < 1e-5,
        "esperava {esperado}, veio {}",
        longe.effective_weight
    );
    assert_eq!(longe.depth, 2);
}

#[test]
fn limiar_mata_a_expansao_antes_de_explodir() {
    let g = grafo_exemplo();
    // 0.34 * 0.60 = 0.204, abaixo de 0.25: o caminho morre no segundo salto.
    let alcancados = g.traverse("api", 3, 0.25, 20);
    assert!(
        !alcancados.iter().any(|n| n.node == "longe"),
        "caminho de peso efetivo 0.204 nao pode sobreviver a um limiar de 0.25"
    );
}

#[test]
fn uma_interacao_nao_desloca_o_peso_alem_do_teto() {
    let mut g = grafo_exemplo();
    let antes = g.edges.iter().find(|e| e.to == "runner").unwrap().weight;

    // Tenta empurrar 0.34 direto para 0.99 numa unica escrita.
    g.upsert_edge("api", "runner", "citado_junto", 0.99);
    let depois = g.edges.iter().find(|e| e.to == "runner").unwrap().weight;

    assert!(
        depois - antes <= 0.05 + 1e-6,
        "uma unica interacao moveu o peso em {}, acima do teto de 0.05",
        depois - antes
    );
    assert!(
        depois > antes,
        "o reforco precisa mover o peso na direcao pedida"
    );
}

#[test]
fn peso_nunca_satura_em_um() {
    let mut g = Graph::new();
    g.upsert_node("a", "t", "");
    g.upsert_node("b", "t", "");
    g.upsert_edge("a", "b", "rel", 0.9);
    for _ in 0..200 {
        g.reinforce("a", "b", 0.05);
    }
    let peso = g.edges[0].weight;
    assert!(
        peso < 1.0,
        "peso saturou em {peso}: a ordenacao perde sentido"
    );
}

#[test]
fn decaimento_remove_aresta_morta() {
    let mut g = Graph::new();
    g.upsert_node("a", "t", "");
    g.upsert_node("b", "t", "");
    g.upsert_edge("a", "b", "rel", 0.03);
    // Empurra o ultimo uso para tres anos atras.
    g.edges[0].last_used = chrono::Utc::now().timestamp() - 60 * 60 * 24 * 365 * 3;

    let removidas = g.decay();
    assert_eq!(removidas, 1);
    assert!(
        g.edges.is_empty(),
        "aresta abaixo do piso precisa sair do grafo"
    );
}

#[test]
fn remover_node_leva_junto_as_arestas() {
    let mut g = grafo_exemplo();
    assert!(g.remove_node("runner"));
    assert!(
        !g.edges
            .iter()
            .any(|e| e.from == "runner" || e.to == "runner"),
        "aresta orfa sobreviveu a remocao do node"
    );
}

#[test]
fn recall_encontra_semente_e_traz_vizinhanca() {
    let g = grafo_exemplo();
    let vistas = g.recall(&["criterio".to_string()], 5);
    assert!(
        vistas.iter().any(|v| v.node == "politica"),
        "recall precisa achar o node pelo texto do contexto, nao so pelo id"
    );
}

#[test]
fn ida_e_volta_pelo_bloco_de_prompt_preserva_os_pesos() {
    let g = grafo_exemplo();
    let vistas = g.recall(&["camada".to_string()], 3);
    let bloco = Graph::as_prompt_block(&vistas);
    assert!(
        bloco.contains("0.92"),
        "o peso precisa chegar ao prompt: {bloco}"
    );
}

/// O artefato em disco: serializa, compacta, grava, le de volta.
///
/// Roda contra o caminho real do sistema, porque e exatamente esse caminho que
/// a promessa de "sem banco de dados" depende. Verifica tambem que o que fica
/// no disco esta compactado, e nao em texto legivel.
#[tokio::test]
async fn o_artefato_faz_ida_e_volta_pelo_disco_compactado() {
    use postly_lib::brain::store::{artifact_path, BrainHandle};

    let handle = BrainHandle::load();
    let marca = format!(
        "prova_{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let contexto = "Texto sentinela que nao pode aparecer em claro no arquivo.";

    handle
        .write(|g| {
            g.upsert_node(&marca, "teste", contexto);
            g.upsert_edge(&marca, "postly", "prova_de_ida_e_volta", 0.77);
        })
        .await
        .expect("a escrita precisa persistir");

    let bytes = std::fs::read(artifact_path()).expect("o artefato precisa existir em disco");
    assert_eq!(
        &bytes[..4],
        b"AGB1",
        "o cabecalho identifica a versao do artefato"
    );
    assert!(
        !String::from_utf8_lossy(&bytes).contains(contexto),
        "o contexto apareceu em claro: o artefato nao esta compactado"
    );

    // Recarrega do zero: so passa se descompactar e desserializar de volta.
    let recarregado = BrainHandle::load();
    let vista = recarregado
        .read(|g| g.neighbors(&marca, 0.0, 5))
        .await
        .expect("o node precisa sobreviver ao ciclo de disco");

    assert_eq!(vista.context, contexto);
    assert!(vista.neighbors.iter().any(|v| v.node == "postly"));

    let stats = recarregado.stats().await;
    assert!(stats.compressed_bytes > 0);
    assert!(
        stats.compressed_bytes < stats.raw_bytes,
        "compactado ({}) precisa ser menor que serializado ({})",
        stats.compressed_bytes,
        stats.raw_bytes
    );
    println!(
        "cerebro: {} nodes, {} arestas, {} bytes serializados -> {} bytes em disco ({:.0}%)",
        stats.nodes,
        stats.edges,
        stats.raw_bytes,
        stats.compressed_bytes,
        stats.ratio * 100.0
    );

    // Nao deixa lixo no cerebro real da maquina.
    recarregado.write(|g| g.remove_node(&marca)).await.unwrap();
}
