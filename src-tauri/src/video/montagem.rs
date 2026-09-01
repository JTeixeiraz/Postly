//! A montagem: o Motion Designer monta, o Auditor confere, o render entrega.
//!
//! Separado do `mod.rs` porque e o miolo compartilhado por dois caminhos que
//! comecam diferente: a primeira execucao chega aqui depois do gerente decidir
//! a linha; a revisao chega direto, com as notas da pessoa como correcao. O
//! que acontece daqui para a frente e identico nos dois, e duplicar faria toda
//! correcao precisar ser feita duas vezes.

use tauri::{AppHandle, Manager};

use super::{assets, prompts, render, spec, PedidoVideo, RelatorioVideo};
use crate::orchestrator::agent::AgentTurn;
use crate::orchestrator::roles::Role;
use crate::orchestrator::support::com_idioma;
use crate::orchestrator::transcript;

/// Quantas voltas o Motion Designer tem para agradar o Auditor.
///
/// Duas, e nao as tres da campanha: um roteiro de cenas tem menos superficie
/// para errar que uma peca com arte, e cada volta aqui custa um turno inteiro
/// de um modelo de nivel medio.
const MAX_RODADAS: u8 = 2;

#[allow(clippy::too_many_arguments)]
pub async fn montar(
    app: AppHandle,
    run: transcript::RunPaths,
    req: &PedidoVideo,
    projeto: &assets::Projeto,
    linha: String,
    // `correcoes_iniciais` sao as notas da pessoa, quando isto e uma revisao.
    correcoes_iniciais: Option<String>,
    mut step: usize,
    mut avisos: Vec<String>,
) -> Result<RelatorioVideo, String> {
    // ---- 3. Motion Designer monta, Auditor confere ----
    let mut correcoes: Option<String> = correcoes_iniciais;
    let mut roteiro: Option<spec::Roteiro> = None;
    let mut parecer = String::new();
    let mut aprovado = false;
    let mut rodadas = 0u8;

    for rodada in 1..=MAX_RODADAS {
        rodadas = rodada;

        step += 1;
        let turno = AgentTurn {
            app: &app,
            run: &run,
            step,
            role: Role::MotionDesigner,
            network: None,
            system: com_idioma(
                crate::idioma::msg(prompts::SYSTEM_MOTION_PT, prompts::SYSTEM_MOTION_EN),
                &req.idioma,
            ),
            prompt: prompts::prompt_motion(&linha, projeto, &req.proporcao, correcoes.as_deref()),
            json_mode: true,
            // Nunca: cargo que devolve JSON nao pensa. O orcamento de tokens
            // iria inteiro para o raciocinio e sobraria pouco para o JSON, que
            // e a unica parte que o programa le.
            pensar: false,
            images: Vec::new(),
        };
        let montagem = turno.execute().await?;
        avisos.extend(montagem.warnings);

        let bruto = montagem.json.ok_or_else(|| {
            crate::idioma::msg(
                "O Motion Designer nao devolveu JSON valido. Tente de novo ou use um \
                 modelo maior no nivel medio.",
                "The Motion Designer did not return valid JSON. Try again or pick a \
                 larger model for the medium tier.",
            )
        })?;

        let candidato: spec::Roteiro = serde_json::from_value::<spec::Roteiro>(bruto.clone())
            .map_err(|e| {
                crate::idioma::msg(
                    &format!("O roteiro veio num formato que nao da para executar: {e}"),
                    &format!("The script came back in a shape that cannot be executed: {e}"),
                )
            })?
            // Apara e preenche o que veio vazio ANTES de validar: a validacao
            // recusa o que nao tem conserto, e a normalizacao conserta o que tem.
            .normalizar();

        // A validacao acontece ANTES da auditoria, e nao depois: nao faz
        // sentido gastar um turno de auditor julgando o ritmo de um roteiro que
        // cita uma imagem que nao existe. E o erro dela e especifico o
        // bastante para virar a correcao da proxima volta.
        if let Err(e) = spec::validar(&candidato, projeto) {
            correcoes = Some(e.to_string());
            avisos.push(e.to_string());
            if rodada == MAX_RODADAS {
                return Err(e.to_string());
            }
            continue;
        }

        step += 1;
        let turno = AgentTurn {
            app: &app,
            run: &run,
            step,
            role: Role::Auditor,
            network: None,
            system: com_idioma(
                crate::idioma::msg(prompts::SYSTEM_AUDITOR_PT, prompts::SYSTEM_AUDITOR_EN),
                &req.idioma,
            ),
            prompt: prompts::prompt_auditor(
                &linha,
                &serde_json::to_string_pretty(&candidato).unwrap_or_default(),
                projeto.tem_narracao(),
            ),
            json_mode: true,
            pensar: false,
            images: Vec::new(),
        };
        let auditoria = turno.execute().await?;
        avisos.extend(auditoria.warnings);

        let v = auditoria.json.unwrap_or_else(|| serde_json::json!({}));
        parecer = v
            .get("parecer")
            .and_then(|x| x.as_str())
            .unwrap_or(&auditoria.handoff)
            .to_string();

        roteiro = Some(candidato);

        if v.get("aprovado").and_then(|x| x.as_bool()).unwrap_or(false) {
            aprovado = true;
            break;
        }

        correcoes = v
            .get("correcoes")
            .and_then(|x| x.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(str::to_string)
            .or_else(|| Some(parecer.clone()));

        if rodada == MAX_RODADAS {
            // O roteiro reprovado ainda renderiza. E a diferenca em relacao a
            // campanha, e ela e do produto: la o auditor barra uma publicacao
            // que vai para o mundo em nome de quem usa, e aqui o resultado e um
            // arquivo que so a pessoa vai ver. Barrar seria decidir por ela.
            avisos.push(crate::idioma::msg(
                "O auditor nao aprovou o roteiro. O video foi renderizado assim mesmo — \
                 leia o parecer antes de usar.",
                "The auditor did not approve the script. The video was rendered anyway — \
                 read the review before using it.",
            ));
        }
    }

    // ---- 4. Render ----
    let mut video = None;
    if let Some(r) = &roteiro {
        // A raiz sai do estado e nao de um parametro: ela e a mesma para o
        // processo inteiro, e passa-la por tres assinaturas so para chegar aqui
        // seria carregar um valor constante pelo caminho todo.
        let raiz = {
            let estado = app.state::<crate::state::AppState>();
            estado.app_root.clone()
        };
        match render::renderizar(&app, &raiz, r, projeto).await {
            Ok(v) => video = Some(v),
            Err(e) => avisos.push(format!(
                "{} {e}",
                crate::idioma::msg("Falha no render:", "Render failed:")
            )),
        }
    }

    Ok(RelatorioVideo {
        run_id: run.id,
        run_dir: run.dir,
        linha,
        roteiro,
        parecer,
        aprovado,
        rodadas,
        video,
        locucao: None,
        avisos,
    })
}
