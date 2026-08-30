//! Cerebro compartilhado dos agentes: grafo de contexto ponderado, sem banco.
//!
//! Regras que a estrutura precisa sustentar:
//!
//! 1. Aresta nunca e binaria. Todo vizinho volta acompanhado do peso da relacao
//!    com o node consultado, ja ordenado do maior para o menor.
//! 2. A consulta corta na origem (limiar minimo + top-k), para o modelo receber
//!    poucas entidades relevantes em vez de uma vizinhanca achatada.
//! 3. Em profundidade > 1 o peso efetivo de um caminho e o PRODUTO dos pesos
//!    percorridos, com limiar de corte: a expansao morre sozinha.
//! 4. Peso e mutavel: reforca quando a relacao e usada e confirmada, decai
//!    quando fica sem uso. Com teto por interacao, para uma unica execucao nao
//!    deslocar a ordenacao inteira.

pub mod store;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Nenhum peso chega a 1.0: saturacao destroi a ordenacao.
const WEIGHT_MAX: f32 = 0.99;
/// Abaixo disto a aresta e considerada morta e sai do grafo.
const WEIGHT_MIN: f32 = 0.02;
/// Teto de deslocamento que UMA interacao pode causar num peso.
const MAX_DELTA_PER_INTERACTION: f32 = 0.05;
/// Fracao do peso perdida por dia sem uso.
const DAILY_DECAY: f32 = 0.01;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "type")]
    pub node_type: String,
    pub context: String,
    pub created_at: i64,
    pub updated_at: i64,
    /// Quantas vezes este node foi entregue a um agente.
    pub hits: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub weight: f32,
    /// Quantas vezes esta aresta foi efetivamente percorrida.
    pub uses: u32,
    pub last_used: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Graph {
    pub nodes: BTreeMap<String, Node>,
    pub edges: Vec<Edge>,
    pub schema_version: u32,
    pub updated_at: i64,
}

/// Um vizinho ja resolvido: relacao, peso e contexto, prontos para o prompt.
#[derive(Debug, Clone, Serialize)]
pub struct NeighborView {
    pub node: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub weight: f32,
    pub context: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeView {
    pub node: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub context: String,
    pub neighbors: Vec<NeighborView>,
}

/// Node alcancado por travessia, com o peso acumulado do caminho.
#[derive(Debug, Clone, Serialize)]
pub struct ReachedNode {
    pub node: String,
    pub context: String,
    pub effective_weight: f32,
    pub depth: u8,
    pub path: Vec<String>,
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
            schema_version: 1,
            updated_at: now(),
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    // ---------------------------------------------------------------- escrita

    pub fn upsert_node(&mut self, id: &str, node_type: &str, context: &str) {
        let ts = now();
        self.nodes
            .entry(id.to_string())
            .and_modify(|n| {
                // Contexto novo substitui o antigo; o node e mutavel por design.
                if !context.is_empty() {
                    n.context = context.to_string();
                }
                n.updated_at = ts;
            })
            .or_insert_with(|| Node {
                node_type: node_type.to_string(),
                context: context.to_string(),
                created_at: ts,
                updated_at: ts,
                hits: 0,
            });
        self.updated_at = ts;
    }

    /// Cria a aresta ou move o peso existente na direcao do valor informado,
    /// sempre respeitando o teto por interacao.
    pub fn upsert_edge(&mut self, from: &str, to: &str, edge_type: &str, weight: f32) {
        let ts = now();
        let target = weight.clamp(WEIGHT_MIN, WEIGHT_MAX);
        if let Some(edge) = self
            .edges
            .iter_mut()
            .find(|e| e.from == from && e.to == to && e.edge_type == edge_type)
        {
            let delta =
                (target - edge.weight).clamp(-MAX_DELTA_PER_INTERACTION, MAX_DELTA_PER_INTERACTION);
            edge.weight = (edge.weight + delta).clamp(WEIGHT_MIN, WEIGHT_MAX);
            edge.uses += 1;
            edge.last_used = ts;
        } else {
            self.edges.push(Edge {
                from: from.to_string(),
                to: to.to_string(),
                edge_type: edge_type.to_string(),
                weight: target,
                uses: 1,
                last_used: ts,
            });
        }
        self.updated_at = ts;
    }

    /// Reforca (delta > 0) ou contradiz (delta < 0) uma relacao ja usada.
    pub fn reinforce(&mut self, from: &str, to: &str, delta: f32) {
        let bounded = delta.clamp(-MAX_DELTA_PER_INTERACTION, MAX_DELTA_PER_INTERACTION);
        let ts = now();
        for edge in self
            .edges
            .iter_mut()
            .filter(|e| e.from == from && e.to == to)
        {
            edge.weight = (edge.weight + bounded).clamp(WEIGHT_MIN, WEIGHT_MAX);
            edge.uses += 1;
            edge.last_used = ts;
        }
        self.updated_at = ts;
    }

    /// Aplica decaimento proporcional ao tempo parado e remove arestas mortas.
    /// Sem isto, reforco continuo satura tudo em 0.99 e a ordenacao perde sentido.
    pub fn decay(&mut self) -> usize {
        let ts = now();
        for edge in self.edges.iter_mut() {
            let days_idle = ((ts - edge.last_used) as f32 / 86_400.0).max(0.0);
            if days_idle < 1.0 {
                continue;
            }
            edge.weight = (edge.weight * (1.0 - DAILY_DECAY).powf(days_idle)).max(0.0);
        }
        let before = self.edges.len();
        self.edges.retain(|e| e.weight >= WEIGHT_MIN);
        self.updated_at = ts;
        before - self.edges.len()
    }

    pub fn remove_node(&mut self, id: &str) -> bool {
        let existed = self.nodes.remove(id).is_some();
        self.edges.retain(|e| e.from != id && e.to != id);
        self.updated_at = now();
        existed
    }

    // ---------------------------------------------------------------- leitura

    /// Node + vizinhanca ponderada, filtrada por limiar e top-k. Esta e a rota
    /// que os agentes mais usam.
    pub fn neighbors(&self, id: &str, min_weight: f32, top_k: usize) -> Option<NodeView> {
        let node = self.nodes.get(id)?;
        let mut neighbors: Vec<NeighborView> = self
            .edges
            .iter()
            .filter_map(|e| {
                // A relacao vale nos dois sentidos para efeito de contexto.
                let other = if e.from == id {
                    &e.to
                } else if e.to == id {
                    &e.from
                } else {
                    return None;
                };
                if e.weight < min_weight {
                    return None;
                }
                let target = self.nodes.get(other)?;
                Some(NeighborView {
                    node: other.clone(),
                    edge_type: e.edge_type.clone(),
                    weight: e.weight,
                    context: target.context.clone(),
                })
            })
            .collect();

        neighbors.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        neighbors.truncate(top_k);

        Some(NodeView {
            node: id.to_string(),
            node_type: node.node_type.clone(),
            context: node.context.clone(),
            neighbors,
        })
    }

    /// Travessia em profundidade com peso efetivo = produto dos pesos do caminho.
    /// O limiar poda a expansao antes de ela explodir combinatorialmente.
    pub fn traverse(
        &self,
        seed: &str,
        depth: u8,
        min_effective: f32,
        top_k: usize,
    ) -> Vec<ReachedNode> {
        let mut reached: HashMap<String, ReachedNode> = HashMap::new();
        let mut frontier: Vec<(String, f32, Vec<String>)> =
            vec![(seed.to_string(), 1.0, vec![seed.to_string()])];

        for level in 1..=depth {
            let mut next = Vec::new();
            for (current, acc_weight, path) in frontier.drain(..) {
                for edge in self.edges.iter() {
                    let other = if edge.from == current {
                        &edge.to
                    } else if edge.to == current {
                        &edge.from
                    } else {
                        continue;
                    };
                    if path.contains(other) {
                        continue;
                    }
                    let effective = acc_weight * edge.weight;
                    if effective < min_effective {
                        continue; // a expansao morre aqui
                    }
                    let Some(node) = self.nodes.get(other) else {
                        continue;
                    };
                    let mut new_path = path.clone();
                    new_path.push(other.clone());

                    let entry = reached.entry(other.clone()).or_insert_with(|| ReachedNode {
                        node: other.clone(),
                        context: node.context.clone(),
                        effective_weight: 0.0,
                        depth: level,
                        path: new_path.clone(),
                    });
                    // Guardamos o caminho mais forte ate cada node.
                    if effective > entry.effective_weight {
                        entry.effective_weight = effective;
                        entry.depth = level;
                        entry.path = new_path.clone();
                    }
                    next.push((other.clone(), effective, new_path));
                }
            }
            frontier = next;
            if frontier.is_empty() {
                break;
            }
        }

        let mut out: Vec<ReachedNode> = reached.into_values().collect();
        out.sort_by(|a, b| {
            b.effective_weight
                .partial_cmp(&a.effective_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out.truncate(top_k);
        out
    }

    /// Encontra nodes-semente por termos e devolve a vizinhanca deles. E o que
    /// monta o bloco de contexto injetado no inicio do prompt de cada agente.
    pub fn recall(&self, terms: &[String], top_k: usize) -> Vec<NodeView> {
        let needles: Vec<String> = terms
            .iter()
            .map(|t| t.to_lowercase())
            .filter(|t| t.len() >= 3)
            .collect();
        if needles.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(u32, String)> = self
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                let haystack = format!("{} {} {}", id, node.node_type, node.context).to_lowercase();
                let score = needles
                    .iter()
                    .filter(|n| haystack.contains(n.as_str()))
                    .count() as u32;
                if score == 0 {
                    None
                } else {
                    // Nodes muito usados sobem no desempate.
                    Some((score * 10 + node.hits.min(9), id.clone()))
                }
            })
            .collect();

        scored.sort_by_key(|x| std::cmp::Reverse(x.0));

        let mut seen = HashSet::new();
        scored
            .into_iter()
            .filter(|(_, id)| seen.insert(id.clone()))
            .take(top_k)
            .filter_map(|(_, id)| self.neighbors(&id, 0.35, 5))
            .collect()
    }

    /// Marca que estes nodes foram entregues a um agente.
    pub fn mark_hits(&mut self, ids: &[String]) {
        for id in ids {
            if let Some(node) = self.nodes.get_mut(id) {
                node.hits = node.hits.saturating_add(1);
            }
        }
    }

    /// Renderiza a vizinhanca como bloco de texto para injecao no prompt.
    pub fn as_prompt_block(views: &[NodeView]) -> String {
        if views.is_empty() {
            return "Nenhum conhecimento previo relevante no cerebro.".to_string();
        }
        let mut out = String::new();
        for view in views {
            out.push_str(&format!(
                "- [{}] {}: {}\n",
                view.node_type, view.node, view.context
            ));
            for neighbor in &view.neighbors {
                out.push_str(&format!(
                    "    -> ({:.2}) {} {}: {}\n",
                    neighbor.weight, neighbor.edge_type, neighbor.node, neighbor.context
                ));
            }
        }
        out
    }
}
