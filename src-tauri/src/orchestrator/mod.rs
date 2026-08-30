//! Pipeline da campanha: quem fala com quem, em que ordem, com qual modelo.
//!
//! O fluxo e sempre linear e sempre com um unico modelo residente por vez:
//!
//!   [Diretor Geral]* -> [Gerente de Setor] (um por rede, em sequencia)
//!   -> [Criador de Conteudo] (UM so, para todas as redes)
//!   -> [Auditor] -> [Gerente ou Diretor] -> aprova ou devolve ao Criador
//!   -> publicacao pelo Playwright -> gravacao no cerebro
//!
//!   * so existe quando ha mais de uma rede na campanha.

pub mod agent;
pub mod movimento;
pub mod prompts;
pub mod roles;
mod support;
mod tipos;
pub mod transcript;

use tauri::AppHandle;

use agent::AgentTurn;
use roles::{Network, Role};
pub use tipos::{CampaignReport, CampaignRequest, PecaFinal};

use crate::brain::Graph;
use crate::state::AppState;
use crate::vault;
use movimento::{descrever_peca, juntar, motivo_do_movimento, pedir_movimento};
use support::{
    com_idioma, descrever_pecas, gravar_no_cerebro, juntar_correcoes, montar_legenda, observar,
    parse_pecas, termos as termos_do_objetivo,
};

pub async fn run_campaign(
    app: AppHandle,
    state: &AppState,
    req: CampaignRequest,
) -> Result<CampaignReport, String> {
    if req.redes.is_empty() {
        return Err(crate::idioma::msg(
            "Selecione ao menos uma rede social.",
            "Pick at least one social network.",
        ));
    }
    if req.objetivo.trim().len() < 10 {
        return Err(crate::idioma::msg(
            "Descreva o objetivo com um pouco mais de detalhe.",
            "Describe the goal with a bit more detail.",
        ));
    }

    let cofre = vault::load();
    if cofre.gemini_api_key.trim().is_empty() {
        return Err(crate::idioma::msg(
            "Configure a chave da API do Gemini antes de iniciar a campanha.",
            "Set the Gemini API key before starting a campaign.",
        ));
    }

    // Identidade e referencias sao restricao de campanha: leem uma vez e valem
    // para todas as rodadas.
    let prefs = crate::prefs::load();
    let identidade = prefs.ds.bloco();
    // A pasta escolhida SOMA com as referencias avulsas, nao substitui: quem
    // subiu uma foto so para esta campanha ainda quer o material do produto.
    let mut referencias = prefs.referencias.clone();
    if !req.pasta.trim().is_empty() {
        referencias.extend(crate::galeria::como_referencias(&req.pasta));
    }
    let refs_texto = crate::referencias::bloco_descritivo(&referencias);
    // So o material da propria marca vira imagem no turno. Referencia de estilo
    // fica no texto: mandar a arte de outra marca para o modelo copiar e o
    // caminho mais curto para a peca sair com logotipo alheio.
    let refs_proprias: Vec<_> = referencias
        .iter()
        .filter(|r| r.tipo == crate::referencias::TipoReferencia::Propria)
        .cloned()
        .collect();
    let refs_imagens = crate::referencias::como_base64(&refs_proprias);

    let run = transcript::start_run(&req.objetivo, &req.redes)?;
    let mut avisos: Vec<String> = Vec::new();
    let mut step = 0usize;

    // ---- contexto do cerebro, injetado no inicio de cada prompt ----
    let termos = termos_do_objetivo(&req.objetivo, &req.redes);
    let contexto = state
        .brain
        .read(|g| {
            let views = g.recall(&termos, 8);
            let ids: Vec<String> = views.iter().map(|v| v.node.clone()).collect();
            (Graph::as_prompt_block(&views), ids)
        })
        .await;
    let (bloco_cerebro, nodes_usados) = contexto;
    let _ = state.brain.write(|g| g.mark_hits(&nodes_usados)).await;

    // ---- 1. Diretor Geral, so quando ha mais de uma rede ----
    let multi = req.redes.len() > 1;
    let diretriz = if multi {
        step += 1;
        let turn = AgentTurn {
            app: &app,
            run: &run,
            step,
            role: Role::DiretorGeral,
            network: None,
            system: com_idioma(prompts::system_diretor_geral(), &req.idioma),
            prompt: prompts::prompt_diretor_geral(&bloco_cerebro, &req.objetivo, &req.redes),
            json_mode: false,
            pensar: req.pensamento_estendido,
            images: Vec::new(),
        };
        let result = turn.execute().await?;
        avisos.extend(result.warnings);
        Some(result.handoff)
    } else {
        None
    };

    // ---- 2. Um gerente por rede: observa o campo e decide a linha criativa ----
    let mut briefings: Vec<(Network, String)> = Vec::new();
    for rede in &req.redes {
        let pesquisa = observar(state, *rede, &req, &mut avisos).await;

        step += 1;
        let turn = AgentTurn {
            app: &app,
            run: &run,
            step,
            role: Role::GerenteSetor,
            network: Some(*rede),
            // O desempenho medido entra no system, nao no prompt: e regra de
            // como decidir, nao dado da tarefa desta campanha.
            system: com_idioma(
                juntar(
                    prompts::system_gerente(*rede),
                    crate::metricas::bloco_de_desempenho(rede.slug()),
                ),
                &req.idioma,
            ),
            prompt: prompts::prompt_gerente(
                &bloco_cerebro,
                &req.objetivo,
                *rede,
                diretriz.as_deref(),
                &pesquisa,
            ),
            json_mode: false,
            pensar: req.pensamento_estendido,
            images: Vec::new(),
        };
        let result = turn.execute().await?;
        avisos.extend(result.warnings);
        briefings.push((*rede, result.handoff));
    }

    let briefings_texto = briefings
        .iter()
        .map(|(rede, texto)| format!("### {} ({})\n{}", rede.label(), rede.slug(), texto))
        .collect::<Vec<_>>()
        .join("\n\n");

    // ---- 3-5. Criador -> imagem -> Auditor -> decisao conjunta, em rodadas ----
    let mut correcoes: Option<String> = None;
    let mut pecas: Vec<PecaFinal> = Vec::new();
    let mut aprovado = false;
    let mut parecer_auditor = String::new();
    let mut rodadas = 0u8;

    for rodada in 1..=req.max_rodadas.max(1) {
        rodadas = rodada;

        // 3. UM criador so, uma sessao, uma peca por rede.
        step += 1;
        let turn = AgentTurn {
            app: &app,
            run: &run,
            step,
            role: Role::Criador,
            network: None,
            system: com_idioma(prompts::system_criador(req.redes.len()), &req.idioma),
            prompt: prompts::prompt_criador(
                &bloco_cerebro,
                &briefings_texto,
                correcoes.as_deref(),
                &identidade,
                &refs_texto,
            ),
            json_mode: true,
            pensar: false,
            images: refs_imagens.clone(),
        };
        let criacao = turn.execute().await?;
        avisos.extend(criacao.warnings);

        let json = criacao.json.ok_or_else(|| {
            crate::idioma::msg(
                "O Criador nao devolveu JSON valido. Tente de novo ou use um modelo \
                 maior no nivel de execucao.",
                "The Creator did not return valid JSON. Try again or pick a larger \
                 model for the execution tier.",
            )
        })?;
        pecas = parse_pecas(&json, &req.redes)?;

        // 4. A imagem e obrigatoria e nasce aqui, antes da auditoria: o auditor
        //    precisa julgar a peca de verdade, nao a promessa dela.
        let media_dir = std::path::PathBuf::from(&run.media_dir);
        for peca in pecas.iter_mut() {
            let aspect = req
                .redes
                .iter()
                .find(|r| r.slug() == peca.rede)
                .map(|r| r.aspect_ratio())
                .unwrap_or("1:1");
            // O pipeline nao sabe qual servico esta ativo: pergunta ao
            // despacho, que le a preferencia e a chave do cofre.
            match crate::imagem::gerar(
                crate::prefs::load().provedor_imagem,
                &cofre,
                &peca.prompt_imagem,
                aspect,
                req.qualidade_imagem,
                &media_dir,
            )
            .await
            {
                Ok(img) => peca.imagem = Some(img),
                Err(e) => avisos.push(format!(
                    "{} {}: {e}",
                    crate::idioma::msg("Imagem de", "Image for"),
                    peca.rede
                )),
            }
        }

        let pecas_texto = descrever_pecas(&pecas);

        // 5. Auditor.
        step += 1;
        let imagens_b64 = pecas
            .iter()
            .filter_map(|p| p.imagem.as_ref())
            .filter_map(|img| std::fs::read(&img.path).ok())
            .map(|bytes| {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(bytes)
            })
            .collect::<Vec<_>>();

        let turn = AgentTurn {
            app: &app,
            run: &run,
            step,
            role: Role::Auditor,
            network: None,
            system: com_idioma(prompts::system_auditor(), &req.idioma),
            prompt: prompts::prompt_auditor(&bloco_cerebro, &briefings_texto, &pecas_texto),
            json_mode: true,
            pensar: false,
            images: imagens_b64,
        };
        let auditoria = turn.execute().await?;
        avisos.extend(auditoria.warnings);

        let parecer = auditoria
            .json
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        let auditor_aprovou = parecer
            .get("aprovado")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        parecer_auditor = parecer
            .get("mensagem_para_gerente")
            .and_then(|v| v.as_str())
            .unwrap_or(&auditoria.handoff)
            .to_string();

        // 6. A decisao e conjunta: o auditor aponta, o gerente (ou o diretor,
        //    quando ha varias redes) decide.
        step += 1;
        let (role_decisor, system, network) = if multi {
            (
                Role::DiretorGeral,
                com_idioma(prompts::system_diretor_validacao(), &req.idioma),
                None,
            )
        } else {
            let rede = req.redes[0];
            (
                Role::GerenteSetor,
                com_idioma(prompts::system_gerente_validacao(rede), &req.idioma),
                Some(rede),
            )
        };
        let turn = AgentTurn {
            app: &app,
            run: &run,
            step,
            role: role_decisor,
            network,
            system,
            prompt: prompts::prompt_gerente_validacao(
                &briefings_texto,
                &pecas_texto,
                &parecer_auditor,
            ),
            json_mode: true,
            pensar: false,
            images: Vec::new(),
        };
        let decisao = turn.execute().await?;
        avisos.extend(decisao.warnings);

        let veredito = decisao.json.unwrap_or_else(|| serde_json::json!({}));
        let decisor_aprovou = veredito
            .get("aprovado")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if auditor_aprovou && decisor_aprovou {
            aprovado = true;
            break;
        }

        correcoes = Some(juntar_correcoes(&parecer, &veredito));
        if rodada == req.max_rodadas.max(1) {
            avisos.push(crate::idioma::msg(
                "Limite de rodadas atingido sem aprovacao. As pecas ficaram gravadas, \
                 mas nao foram publicadas.",
                "Round limit reached without approval. The pieces were saved but not \
                 published.",
            ));
        }
    }

    // ---- 6b. Motion designer, quando o gerente pediu e a pessoa aceitou ----
    //
    // Fica entre a aprovacao e a publicacao de proposito: animar uma peca que
    // ainda pode ser reprovada seria trabalho jogado fora, e depois de publicar
    // ja e tarde.
    if aprovado {
        for peca in pecas.iter_mut() {
            let Some(rede) = req.redes.iter().find(|r| r.slug() == peca.rede).copied() else {
                continue;
            };
            let briefing = briefings
                .iter()
                .find(|(r, _)| *r == rede)
                .map(|(_, b)| b.clone())
                .unwrap_or_default();

            let Some(motivo) = motivo_do_movimento(&briefing) else {
                continue;
            };

            // A campanha para aqui ate a pessoa responder. A notificacao do
            // sistema sai junto do evento, porque um modal atras de outra
            // janela e o mesmo que nao perguntar.
            if !pedir_movimento(&app, state, rede, &motivo).await {
                peca.motion_pedido = true;
                continue;
            }

            step += 1;
            let turn = AgentTurn {
                app: &app,
                run: &run,
                step,
                role: Role::MotionDesigner,
                network: Some(rede),
                system: com_idioma(prompts::system_motion(), &req.idioma),
                prompt: prompts::prompt_motion(
                    &briefing,
                    &descrever_peca(peca),
                    &motivo,
                    rede.aspect_ratio(),
                ),
                json_mode: false,
                pensar: false,
                images: Vec::new(),
            };
            match turn.execute().await {
                Ok(r) => {
                    avisos.extend(r.warnings);
                    peca.motion_pedido = true;
                    peca.roteiro_motion = Some(r.handoff);
                }
                Err(e) => avisos.push(format!(
                    "{} {}: {e}",
                    crate::idioma::msg("Falha no turno de motion em", "Motion turn failed on"),
                    rede.label()
                )),
            }
        }
    }

    // ---- 7. Publicacao ----
    if aprovado {
        for peca in pecas.iter_mut() {
            let Some(rede) = req.redes.iter().find(|r| r.slug() == peca.rede).copied() else {
                continue;
            };
            let Some(imagem) = peca.imagem.as_ref() else {
                peca.detalhe_publicacao = crate::idioma::msg(
                    "Sem imagem gerada; publicacao cancelada.",
                    "No image was generated; publishing cancelled.",
                );
                continue;
            };
            let legenda = montar_legenda(peca);
            match state
                .browser
                .publish(rede, &imagem.path, &legenda, req.simular)
                .await
            {
                Ok(outcome) => {
                    peca.publicado = outcome.published;
                    peca.detalhe_publicacao = outcome.detail;
                    peca.screenshot = outcome.screenshot;
                }
                Err(e) => {
                    peca.detalhe_publicacao = e.clone();
                    avisos.push(format!(
                        "{} {}: {e}",
                        crate::idioma::msg("Falha ao publicar em", "Failed to publish on"),
                        rede.label()
                    ));
                }
            }
        }
    }

    // ---- 8. O que a campanha aprendeu volta para o cerebro ----
    gravar_no_cerebro(state, &run.id, &req, &pecas, aprovado).await?;

    transcript::append_index(
        &run,
        &format!(
            "\n---\n\n**Resultado:** {} em {} rodada(s).\n\n**Parecer do auditor:** {}\n",
            if aprovado { "aprovado" } else { "reprovado" },
            rodadas,
            parecer_auditor.trim()
        ),
    )?;

    // O resultado vai para o disco ANTES de qualquer outra coisa poder falhar.
    // Sem este arquivo, fechar a janela apagava a legenda, as hashtags e o
    // caminho da arte: sobrava so a conversa do modelo em markdown.
    transcript::write_result(
        &run,
        &transcript::RunResult {
            id: run.id.clone(),
            objetivo: req.objetivo.clone(),
            redes: req.redes.iter().map(|r| r.slug().to_string()).collect(),
            aprovado,
            rodadas,
            simulado: req.simular,
            parecer_auditor: parecer_auditor.clone(),
            avisos: avisos.clone(),
            pecas: pecas.clone(),
            encerrada_em: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        },
    )?;

    if req.salvar_credenciais && !req.credenciais.is_empty() {
        let mut cofre = vault::load();
        for (slug, cred) in &req.credenciais {
            cofre.credentials.insert(slug.clone(), cred.clone());
        }
        vault::save(&cofre)?;
    }

    Ok(CampaignReport {
        run_id: run.id,
        run_dir: run.dir,
        index_path: run.index,
        pecas,
        rodadas,
        aprovado,
        parecer_auditor,
        avisos,
    })
}
