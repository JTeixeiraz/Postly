//! Comandos da aba de auditoria: registrar desempenho, ler e analisar.
//!
//! A coleta tem dois caminhos de proposito. Digitar a mao sempre funciona e
//! alcanca o alcance, que so existe no painel profissional da rede. Raspar e
//! rapido e quebra: e o mesmo problema dos seletores de publicacao, agora
//! aplicado a numeros que mudam de lugar toda semana. Um nao substitui o
//! outro, e a tela diz de onde veio cada registro.

use serde::Deserialize;

use crate::metricas::{self, LeituraDaRede, Origem, Registro};
use crate::orchestrator::roles::Network;
use crate::state::AppState;

#[tauri::command]
pub fn listar_metricas() -> Vec<Registro> {
    metricas::load()
}

#[tauri::command]
pub fn leitura_desempenho() -> Vec<LeituraDaRede> {
    metricas::ler_tudo(&metricas::load())
}

/// O que a tela manda ao registrar. O `id` e o `coletado_em` nascem aqui: sao
/// do sistema, e deixar a tela escolher abriria caminho para dois registros
/// com o mesmo id sobrescreverem um ao outro.
#[derive(Debug, Deserialize)]
pub struct NovaMetrica {
    /// Preenchido quando esta editando um registro existente.
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub run_id: String,
    pub rede: String,
    pub publicado_em: String,
    #[serde(default)]
    pub url: String,
    pub conceito: String,
    #[serde(default)]
    pub impressoes: u64,
    #[serde(default)]
    pub curtidas: u64,
    #[serde(default)]
    pub comentarios: u64,
    #[serde(default)]
    pub compartilhamentos: u64,
    #[serde(default)]
    pub salvamentos: u64,
    #[serde(default)]
    pub cliques: u64,
}

fn agora() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
}

fn novo_id() -> String {
    format!("m{}", chrono::Utc::now().format("%Y%m%dT%H%M%S%3f"))
}

#[tauri::command]
pub fn registrar_metrica(m: NovaMetrica) -> Result<Vec<Registro>, String> {
    if m.conceito.trim().is_empty() {
        return Err(crate::idioma::msg(
            "Descreva em uma linha o que era a peca: e por esse texto que a analise \
             correlaciona o que funcionou.",
            "Describe in one line what the piece was: that text is what the analysis \
             correlates performance against.",
        ));
    }
    if m.rede.trim().is_empty() {
        return Err(crate::idioma::msg("Escolha a rede.", "Pick the network."));
    }

    let mut regs = metricas::load();
    let reg = Registro {
        id: if m.id.trim().is_empty() {
            novo_id()
        } else {
            m.id.clone()
        },
        run_id: m.run_id,
        rede: m.rede,
        publicado_em: m.publicado_em,
        url: m.url,
        conceito: m.conceito.trim().to_string(),
        impressoes: m.impressoes,
        curtidas: m.curtidas,
        comentarios: m.comentarios,
        compartilhamentos: m.compartilhamentos,
        salvamentos: m.salvamentos,
        cliques: m.cliques,
        origem: Origem::Manual,
        coletado_em: agora(),
    };

    match regs.iter().position(|r| r.id == reg.id) {
        Some(i) => regs[i] = reg,
        None => regs.push(reg),
    }
    metricas::save(&regs)?;
    Ok(regs)
}

#[tauri::command]
pub fn remover_metrica(id: String) -> Result<Vec<Registro>, String> {
    let mut regs = metricas::load();
    regs.retain(|r| r.id != id);
    metricas::save(&regs)?;
    Ok(regs)
}

/// Le os numeros visiveis das ultimas publicacoes da conta, pelo navegador.
///
/// Best-effort por natureza. O que volta sao curtidas e comentarios, que ficam
/// na propria peca; alcance costuma morar no painel profissional e continua
/// sendo trabalho de digitar. Registros vindos daqui ficam marcados como
/// raspagem para a tela poder mostrar de onde veio cada numero.
#[tauri::command]
pub async fn coletar_metricas(
    state: tauri::State<'_, AppState>,
    rede: Network,
    limite: u32,
) -> Result<Vec<Registro>, String> {
    let colhidos = state.browser.metrics(rede, limite.clamp(1, 25)).await?;
    if colhidos.is_empty() {
        return Err(crate::idioma::msg(
            "Nada foi lido da pagina. Isso costuma significar que a rede mudou o \
             layout: registre a mao por enquanto.",
            "Nothing could be read from the page. That usually means the network \
             changed its layout: enter the numbers manually for now.",
        ));
    }

    let mut regs = metricas::load();
    let momento = agora();
    let mut novos = 0usize;

    for c in colhidos {
        // A URL da publicacao e a unica identidade estavel entre coletas. Sem
        // ela, cada raspagem duplicaria o historico inteiro e a mediana viraria
        // ficcao.
        if let Some(i) = regs
            .iter()
            .position(|r| !r.url.is_empty() && r.url == c.url)
        {
            // Registro que ja existe so tem os numeros atualizados: um valor
            // digitado a mao (alcance, tipicamente) nao pode ser apagado por
            // uma coleta que nao sabe le-lo.
            regs[i].curtidas = c.curtidas;
            regs[i].comentarios = c.comentarios;
            if c.impressoes > 0 {
                regs[i].impressoes = c.impressoes;
            }
            regs[i].coletado_em = momento.clone();
            continue;
        }
        regs.push(Registro {
            id: format!("{}-{novos}", novo_id()),
            run_id: String::new(),
            rede: rede.slug().to_string(),
            publicado_em: c.publicado_em,
            url: c.url,
            conceito: c.resumo,
            impressoes: c.impressoes,
            curtidas: c.curtidas,
            comentarios: c.comentarios,
            compartilhamentos: 0,
            salvamentos: 0,
            cliques: 0,
            origem: Origem::Raspagem,
            coletado_em: momento.clone(),
        });
        novos += 1;
    }

    metricas::save(&regs)?;
    Ok(regs)
}

// ------------------------------------------------------------- a analise

/// Roda um turno de analise sobre o historico inteiro.
///
/// Nao usa a maquinaria de campanha de proposito: aquilo grava transcricao,
/// abre pasta de execucao e mexe no cerebro. Aqui e uma pergunta so, sem
/// efeito colateral, e o resultado a pessoa le na tela.
#[tauri::command]
pub async fn analisar_desempenho() -> Result<String, String> {
    let regs = metricas::load();
    if regs.len() < 3 {
        return Err(crate::idioma::msg(
            "Registre pelo menos tres publicacoes. Com menos que isso, qualquer \
             leitura seria opiniao com aparencia de dado.",
            "Log at least three posts. With fewer than that, any reading would be \
             opinion dressed up as data.",
        ));
    }

    let leituras = metricas::ler_tudo(&regs);
    let tabela = tabela_para_o_modelo(&regs, &leituras);
    let system = crate::idioma::msg(SYSTEM_ANALISE_PT, SYSTEM_ANALISE_EN);
    let prompt = format!(
        "{}\n\n{tabela}",
        crate::idioma::msg("Analise o historico abaixo.", "Analyse the history below.")
    );

    match crate::prefs::load().provedor {
        crate::prefs::Provedor::ClaudeCode => crate::claude::turno(
            crate::orchestrator::roles::Tier::Alto,
            &system,
            &prompt,
            600,
        )
        .await
        .map(|t| t.texto),
        crate::prefs::Provedor::Ollama => {
            let perfil = crate::hardware::compute_profile();
            let instalados = crate::ollama::client::installed_models().await;
            // `installed_only` ligado: a analise nao pode disparar um download
            // de 20 GB porque alguem clicou em "analisar".
            let (spec, _) = crate::ollama::catalog::pick(
                crate::orchestrator::roles::Tier::Alto,
                perfil.live_budget_bytes,
                perfil.mode,
                true,
                &instalados,
            )
            .ok_or_else(|| {
                crate::idioma::msg(
                    "Nenhum modelo instalado cabe na memoria livre agora. Feche algo \
                     ou baixe um modelo menor na aba Modelos.",
                    "No installed model fits in free memory right now. Close something \
                     or download a smaller model on the Models tab.",
                )
            })?;

            let r = crate::ollama::client::generate(
                spec.tag,
                Some(&system),
                &prompt,
                crate::ollama::client::GenerateOptions {
                    temperature: 0.3,
                    num_ctx: 8192,
                    num_predict: 2048,
                },
                false,
                false,
                Vec::new(),
            )
            .await?;
            // Descarrega como qualquer turno: a analise nao pode deixar um
            // modelo residente segurando memoria da proxima campanha.
            crate::ollama::client::unload(spec.tag).await.ok();
            Ok(r.response)
        }
    }
}

/// Os numeros em texto tabular.
///
/// Tabela e nao JSON: modelo pequeno erra menos lendo colunas alinhadas do que
/// contando chaves, e o custo em token e menor.
fn tabela_para_o_modelo(regs: &[Registro], leituras: &[LeituraDaRede]) -> String {
    let mut out = String::new();
    for l in leituras {
        out.push_str(&format!(
            "\n## {} — {} publicacoes, mediana {:.1} ({})\n",
            l.rede,
            l.publicacoes,
            l.mediana,
            match l.base {
                metricas::Base::Taxa => "interacao por mil impressoes",
                metricas::Base::Volume => "interacao bruta, sem alcance disponivel",
            }
        ));
        for item in &l.ranking {
            let r = regs.iter().find(|r| r.id == item.id);
            out.push_str(&format!(
                "- {:.2}x | {} | {} | curtidas {} comentarios {} compart. {}\n",
                item.multiplo,
                item.publicado_em,
                item.conceito,
                r.map(|r| r.curtidas).unwrap_or(0),
                r.map(|r| r.comentarios).unwrap_or(0),
                r.map(|r| r.compartilhamentos).unwrap_or(0),
            ));
        }
    }
    out
}

const SYSTEM_ANALISE_PT: &str = "Voce e o analista de desempenho da equipe de marketing. \
     Recebe o historico real de publicacoes de uma conta, ja ranqueado, e devolve uma \
     leitura curta e acionavel.\n\n\
     REGRA CENTRAL: o objetivo nunca e repetir o que funcionou. Repeticao satura \
     publico e algoritmo. Seu trabalho e extrair o PRINCIPIO por tras do que rendeu \
     (que tipo de gancho, que registro, que formato de argumento) para que a proxima \
     peca leve esse principio a um lugar novo. So recomende continuar na mesma linha \
     quando uma peca tiver batido tres vezes a mediana ou mais: nesse caso e veio, \
     nao sorte.\n\n\
     RIGOR: o multiplo ja vem calculado, nao recalcule. Com menos de quatro \
     publicacoes numa rede, diga que ainda nao ha base e pare por ali. Nao invente \
     causa que os dados nao sustentam: se dois conceitos parecidos renderam \
     diferente, diga que a amostra nao separa os dois.\n\n\
     ENTREGA, exatamente nesta ordem e sem introducao:\n\
     1. O QUE OS DADOS DIZEM — no maximo tres frases, cada uma amarrada a um numero.\n\
     2. PRINCIPIO A LEVAR ADIANTE — uma frase, sobre o mecanismo e nao sobre o tema.\n\
     3. O QUE PARAR DE FAZER — uma frase, apoiada no que ficou abaixo da mediana.\n\
     4. PROXIMO TESTE — uma hipotese concreta, com o numero que ela precisa bater.";

const SYSTEM_ANALISE_EN: &str = "You are the marketing team's performance analyst. You \
     receive an account's real posting history, already ranked, and return a short, \
     actionable reading.\n\n\
     CORE RULE: the goal is never to repeat what worked. Repetition saturates both \
     audience and algorithm. Your job is to extract the PRINCIPLE behind what \
     performed (what kind of hook, what register, what argument shape) so the next \
     piece can take that principle somewhere new. Only recommend staying on the same \
     line when a piece beat the median by three times or more: that is a vein, not \
     luck.\n\n\
     RIGOUR: multiples are precomputed, do not recompute them. With fewer than four \
     posts on a network, say there is no basis yet and stop. Do not invent causes the \
     data cannot support: if two similar concepts performed differently, say the \
     sample does not separate them.\n\n\
     DELIVERABLE, in this exact order and with no preamble:\n\
     1. WHAT THE DATA SAYS — at most three sentences, each tied to a number.\n\
     2. PRINCIPLE TO CARRY FORWARD — one sentence, about the mechanism, not the topic.\n\
     3. WHAT TO STOP DOING — one sentence, grounded in what fell below the median.\n\
     4. NEXT TEST — one concrete hypothesis, with the number it has to beat.";

// ------------------------------------------------------------------ motion

/// A resposta da pessoa ao modal de movimento.
///
/// Acorda a campanha que dorme em `pedir_movimento`. Se nao houver ninguem
/// esperando, nao e erro: a campanha pode ter estourado o tempo enquanto a
/// janela ficava aberta, e nesse caso o clique so nao tem para onde ir.
#[tauri::command]
pub fn responder_motion(state: tauri::State<'_, AppState>, aceitar: bool) -> Result<(), String> {
    let canal = state
        .resposta_motion
        .lock()
        .map_err(|_| "estado de motion indisponivel".to_string())?
        .take();
    if let Some(tx) = canal {
        let _ = tx.send(aceitar);
    }
    Ok(())
}
