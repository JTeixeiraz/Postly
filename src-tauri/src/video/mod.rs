//! O video avulso: pedir um video, montar e renderizar.
//!
//! NAO E UMA CAMPANHA, e por isso nao reusa o pipeline dela. Nao ha rede
//! social, nao ha peca, nao ha publicacao e nao ha Diretor Geral para dividir
//! estrategia entre redes que nao existem. A pessoa quer um arquivo `.mp4`
//! para baixar e fazer o que quiser com ele.
//!
//! Tres cargos, nesta ordem:
//!
//! ```text
//!   Gerente de Setor      decide a linha do video
//!         │
//!         ├── [ha narracao na pasta?] ── nao ──▶ PERGUNTA e espera
//!         │                                      └─ quer voz? entrega roteiro
//!         │                                         + o link do ElevenLabs
//!   Motion Designer       monta o roteiro de cenas em JSON
//!         │
//!   Auditor               aprova ou devolve
//!         │
//!   render                sidecar Node + Remotion ──▶ .mp4
//! ```
//!
//! A TESE CONTINUA VALENDO. Cada turno passa pelo mesmo `AgentTurn` da
//! campanha: mede memoria, escolhe o modelo do nivel do cargo, sobe, grava a
//! conversa em Markdown, DESCARREGA e passa adiante so a mensagem. Um caminho
//! paralelo que mantivesse modelo residente seria emenda constitucional
//! disfarcada de tela nova.

pub mod assets;
pub mod direcao;
mod montagem;
pub mod narracao;
pub mod prompts;
pub mod render;
pub mod revisao;
pub mod spec;

use tauri::AppHandle;

use crate::orchestrator::agent::AgentTurn;
use crate::orchestrator::roles::Role;
use crate::orchestrator::support::com_idioma;
use crate::orchestrator::transcript;
use crate::state::AppState;
use montagem::montar;
use revisao::revisar;
pub use revisao::NotaDeCena;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PedidoVideo {
    /// O slug do projeto, que ja existe em disco com os assets dentro.
    pub projeto: String,
    pub objetivo: String,
    /// `16:9`, `9:16` ou `1:1`.
    pub proporcao: String,
    pub idioma: String,
    /// Raciocinio estendido no cargo que decide a linha.
    #[serde(default)]
    pub pensamento_estendido: bool,
    /// O que a pessoa apontou no roteiro anterior.
    ///
    /// Quando vem preenchido, o video NAO recomeca do zero: o gerente e pulado
    /// e o Motion Designer refaz a partir do roteiro que ja existe. Refazer a
    /// linha inteira jogaria fora o trabalho que a pessoa acabou de revisar, e
    /// ela receberia um video diferente em vez do mesmo video corrigido.
    #[serde(default)]
    pub notas: Vec<NotaDeCena>,
    /// O roteiro que as notas comentam. Anda junto delas, sempre.
    #[serde(default)]
    pub roteiro_anterior: Option<spec::Roteiro>,
    /// A linha do gerente da rodada anterior, para nao rodar o cargo de novo.
    #[serde(default)]
    pub linha_anterior: String,
}

impl PedidoVideo {
    /// Isto e uma revisao de um roteiro que ja existe?
    fn e_revisao(&self) -> bool {
        self.roteiro_anterior.is_some() && !self.notas.is_empty()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RelatorioVideo {
    pub run_id: String,
    pub run_dir: String,
    pub linha: String,
    pub roteiro: Option<spec::Roteiro>,
    pub parecer: String,
    pub aprovado: bool,
    pub rodadas: u8,
    /// O arquivo pronto, quando o render rodou.
    pub video: Option<render::VideoPronto>,
    /// O roteiro de locucao, quando a pessoa pediu narracao e ainda vai gravar.
    ///
    /// Quando isto vem preenchido, `video` vem vazio de proposito: nao ha o que
    /// renderizar ate o audio existir. Ver o comentario na etapa 2.
    pub locucao: Option<narracao::RoteiroDeLocucao>,
    pub avisos: Vec<String>,
}

pub async fn rodar(
    app: AppHandle,
    state: &AppState,
    req: PedidoVideo,
) -> Result<RelatorioVideo, String> {
    if req.objetivo.trim().len() < 10 {
        return Err(crate::idioma::msg(
            "Descreva o video com um pouco mais de detalhe.",
            "Describe the video with a bit more detail.",
        ));
    }

    let projeto = assets::ler(&req.projeto).ok_or_else(|| {
        crate::idioma::msg(
            "Projeto de video nao encontrado.",
            "Video project not found.",
        )
    })?;

    if projeto.imagens.is_empty() {
        // Recusado aqui e nao na validacao do roteiro: sem imagem o Motion
        // Designer so poderia montar cenas de texto, e a pessoa esperaria tres
        // turnos para receber um video que nao e o que ela queria.
        return Err(crate::idioma::msg(
            "Suba ao menos uma imagem na aba de assets antes de gerar o video.",
            "Add at least one image on the assets tab before generating the video.",
        ));
    }

    let run = transcript::start_run(&req.objetivo, &[])?;
    let mut avisos: Vec<String> = Vec::new();
    let mut step = 0usize;

    // ---- 1. O gerente decide a linha ----
    //
    // Ele ja sabe se ha voz: a pasta `narracao/` responde, e a resposta muda o
    // que ele escreve. Perguntar antes de o gerente falar deixaria a pergunta
    // sem contexto — ver a nota em `narracao.rs`.
    //
    // NUMA REVISAO ELE NAO RODA. A linha ja foi decidida e a pessoa acabou de
    // revisar o resultado dela; refazer a linha entregaria um video diferente
    // em vez do mesmo video corrigido — e gastaria o turno mais caro da
    // execucao para chegar de volta no mesmo lugar.
    if req.e_revisao() {
        return revisar(app, run, req, projeto, avisos).await;
    }

    step += 1;
    let turno = AgentTurn {
        app: &app,
        run: &run,
        step,
        role: Role::GerenteSetor,
        network: None,
        system: com_idioma(
            crate::idioma::msg(prompts::SYSTEM_GERENTE_PT, prompts::SYSTEM_GERENTE_EN),
            &req.idioma,
        ),
        prompt: prompts::prompt_gerente(&req.objetivo, &projeto, projeto.tem_narracao()),
        json_mode: false,
        pensar: req.pensamento_estendido,
        images: Vec::new(),
    };
    let r = turno.execute().await?;
    avisos.extend(r.warnings);
    let linha = r.handoff;

    // ---- 2. Nao ha voz gravada: a pessoa decide se quer ----
    //
    // Quando ela pede o roteiro, o VIDEO PARA AQUI. Nao ha o que renderizar
    // enquanto o audio nao existir, e montar as cenas agora seria monta-las
    // para texto na tela — exatamente o que teria que ser refeito quando a voz
    // chegasse. A pessoa gera a voz, larga na pasta e roda de novo; da segunda
    // vez a pasta responde sim e este bloco nao acontece.
    if !projeto.tem_narracao() {
        let pasta = projeto.caminho_de("narracao");
        if narracao::perguntar(&app, state, &linha, &pasta).await
            == narracao::RespostaNarracao::QueroRoteiro
        {
            step += 1;
            let alvo = 45; // segundos: um video de assets costuma caber nisso.
            let turno = AgentTurn {
                app: &app,
                run: &run,
                step,
                role: Role::MotionDesigner,
                network: None,
                system: com_idioma(
                    crate::idioma::msg(prompts::SYSTEM_LOCUCAO_PT, prompts::SYSTEM_LOCUCAO_EN),
                    &req.idioma,
                ),
                prompt: prompts::prompt_locucao(&req.objetivo, &linha, alvo),
                json_mode: false,
                pensar: false,
                images: Vec::new(),
            };
            let r = turno.execute().await?;
            avisos.extend(r.warnings);

            return Ok(RelatorioVideo {
                run_id: run.id,
                run_dir: run.dir,
                linha,
                roteiro: None,
                parecer: String::new(),
                aprovado: false,
                rodadas: 0,
                video: None,
                locucao: Some(narracao::RoteiroDeLocucao::novo(r.handoff, &pasta)),
                avisos,
            });
        }
    }

    montar(app, run, &req, &projeto, linha, None, step, avisos).await
}
