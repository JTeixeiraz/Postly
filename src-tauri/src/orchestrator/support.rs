//! Auxiliares do pipeline: navegador, parsing da saida do criador e escrita no cerebro.
//!
//! Separado de `mod.rs` para manter cada arquivo legivel: la fica a ordem dos
//! cargos, aqui fica a mecanica que sustenta cada passo.

use super::roles::Network;
use super::{CampaignRequest, PecaFinal};
use crate::state::AppState;
use crate::vault;

/// Abre a rede, garante sessao e coleta a observacao de campo para o gerente.
pub async fn observar(
    state: &AppState,
    rede: Network,
    req: &CampaignRequest,
    avisos: &mut Vec<String>,
) -> String {
    let sessao = match state.browser.open(rede, false).await {
        Ok(s) => s,
        Err(e) => {
            avisos.push(format!("Navegador nao abriu em {}: {e}", rede.label()));
            return String::new();
        }
    };

    if !sessao.logged_in {
        let cofre = vault::load();
        let cred = req
            .credenciais
            .get(rede.slug())
            .cloned()
            .or_else(|| cofre.credentials.get(rede.slug()).cloned());

        match cred {
            Some(c) => {
                if let Err(e) = state.browser.login(rede, &c.username, &c.password).await {
                    avisos.push(format!("Login em {} falhou: {e}", rede.label()));
                    return String::new();
                }
            }
            None => {
                avisos.push(format!(
                    "Sem sessao e sem credenciais para {}. Faca login na janela que abriu.",
                    rede.label()
                ));
                return String::new();
            }
        }
    }

    state.browser.research(rede, 8).await.unwrap_or_else(|e| {
        avisos.push(format!(
            "Observacao de campo em {} falhou: {e}",
            rede.label()
        ));
        String::new()
    })
}

pub fn parse_pecas(json: &serde_json::Value, redes: &[Network]) -> Result<Vec<PecaFinal>, String> {
    let array = json
        .get("pecas")
        .and_then(|v| v.as_array())
        .cloned()
        // Modelo pequeno as vezes devolve uma peca solta em vez do array.
        .unwrap_or_else(|| vec![json.clone()]);

    let mut out = Vec::new();
    for (i, item) in array.iter().enumerate() {
        let rede = item
            .get("rede")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase())
            .filter(|s| redes.iter().any(|r| r.slug() == s))
            .unwrap_or_else(|| redes.get(i).or(redes.first()).unwrap().slug().to_string());

        let texto = |key: &str| {
            item.get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .trim()
                .to_string()
        };
        let hashtags = item
            .get("hashtags")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|h| h.as_str())
                    .map(|h| {
                        if h.starts_with('#') {
                            h.to_string()
                        } else {
                            format!("#{h}")
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let legenda = texto("legenda");
        let prompt_imagem = texto("prompt_imagem");
        if legenda.is_empty() || prompt_imagem.is_empty() {
            return Err(format!(
                "Peca {} veio incompleta do Criador (falta legenda ou prompt de imagem).",
                i + 1
            ));
        }

        out.push(PecaFinal {
            motion_pedido: false,
            roteiro_motion: None,
            rede,
            conceito: texto("conceito"),
            prompt_imagem,
            legenda,
            hashtags,
            chamada_para_acao: texto("chamada_para_acao"),
            imagem: None,
            publicado: false,
            detalhe_publicacao: String::new(),
            screenshot: None,
        });
    }
    Ok(out)
}

pub fn descrever_pecas(pecas: &[PecaFinal]) -> String {
    pecas
        .iter()
        .map(|p| {
            format!(
                "### {}\n- Conceito: {}\n- Prompt da imagem: {}\n- Imagem gerada: {}\n- Legenda:\n{}\n- Hashtags: {}\n- CTA: {}",
                p.rede,
                p.conceito,
                p.prompt_imagem,
                p.imagem.as_ref().map(|i| i.path.as_str()).unwrap_or("NAO GERADA"),
                p.legenda,
                p.hashtags.join(" "),
                p.chamada_para_acao
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub fn montar_legenda(peca: &PecaFinal) -> String {
    if peca.hashtags.is_empty() {
        peca.legenda.clone()
    } else {
        format!("{}\n\n{}", peca.legenda, peca.hashtags.join(" "))
    }
}

pub fn juntar_correcoes(parecer: &serde_json::Value, veredito: &serde_json::Value) -> String {
    let listar = |v: &serde_json::Value, key: &str| -> Vec<String> {
        v.get(key)
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|i| i.as_str())
                    .map(|s| format!("- {s}"))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut linhas = Vec::new();
    linhas.extend(listar(parecer, "alucinacoes"));
    linhas.extend(listar(parecer, "desvios_do_briefing"));
    linhas.extend(listar(parecer, "correcoes"));
    linhas.extend(listar(veredito, "correcoes"));
    if let Some(motivo) = veredito.get("motivo").and_then(|v| v.as_str()) {
        linhas.push(format!("- Decisao do gestor: {motivo}"));
    }
    if linhas.is_empty() {
        "- A rodada foi reprovada sem correcao explicita. Refaca com mais aderencia ao briefing."
            .to_string()
    } else {
        linhas.join("\n")
    }
}

/// Escreve o resultado no grafo. Peca aprovada reforca a relacao entre o angulo
/// e a rede; peca reprovada a contradiz. E assim que o cerebro aprende sem que
/// nenhum peso de modelo seja tocado.
pub async fn gravar_no_cerebro(
    state: &AppState,
    run_id: &str,
    req: &CampaignRequest,
    pecas: &[PecaFinal],
    aprovado: bool,
) -> Result<(), String> {
    let campanha = format!("campanha_{run_id}");
    let objetivo = req.objetivo.trim().to_string();
    let pecas = pecas.to_vec();
    let redes: Vec<Network> = req.redes.clone();

    state
        .brain
        .write(move |g| {
            g.upsert_node(&campanha, "campanha", &objetivo);
            for rede in &redes {
                g.upsert_node(rede.slug(), "rede_social", rede.format_hint());
                g.upsert_edge(&campanha, rede.slug(), "publica_em", 0.7);
            }
            for peca in &pecas {
                if peca.conceito.is_empty() {
                    continue;
                }
                let id = slug(&peca.conceito);
                g.upsert_node(&id, "angulo_criativo", &peca.conceito);
                g.upsert_edge(&campanha, &id, "produziu", 0.65);
                // Aprovado sobe, reprovado desce. O teto por interacao impede que
                // uma unica campanha reordene o grafo inteiro.
                let alvo = if aprovado { 0.85 } else { 0.30 };
                g.upsert_edge(&peca.rede, &id, "funcionou_em", alvo);
                if !peca.chamada_para_acao.is_empty() {
                    let cta = slug(&peca.chamada_para_acao);
                    g.upsert_node(&cta, "chamada_para_acao", &peca.chamada_para_acao);
                    g.upsert_edge(&id, &cta, "usa", if aprovado { 0.75 } else { 0.35 });
                }
            }
            g.decay();
        })
        .await
        .map(|_| ())
}

/// Identificador estavel de node a partir de texto livre.
pub fn slug(text: &str) -> String {
    let cleaned: String = text
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    cleaned
        .split('_')
        .filter(|s| !s.is_empty())
        .take(6)
        .collect::<Vec<_>>()
        .join("_")
}

/// Anexa a clausula de idioma a um system prompt.
pub fn com_idioma(system: String, idioma: &str) -> String {
    format!("{system}\n\n{}", super::prompts::clausula_idioma(idioma))
}

/// Termos usados para acordar o cerebro antes do primeiro agente.
pub(super) fn termos(objetivo: &str, redes: &[Network]) -> Vec<String> {
    let mut out: Vec<String> = objetivo
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 4)
        .map(|w| w.to_lowercase())
        .collect();
    out.extend(redes.iter().map(|r| r.slug().to_string()));
    out.extend(["cargo".into(), "tatica".into()]);
    out.truncate(24);
    out
}
