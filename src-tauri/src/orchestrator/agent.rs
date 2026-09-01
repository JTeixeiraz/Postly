//! Execucao de UM turno de agente.
//!
//! O ciclo completo, e a razao de o sistema ser um middleware e nao um chat:
//!
//!   mede RAM livre -> escolhe o modelo mais forte do nivel do cargo que caiba
//!   -> baixa se faltar -> sobe -> envia prompt base + mensagem do cargo anterior
//!   -> recebe a resposta -> grava a conversa inteira em .md -> DESCARREGA o
//!   modelo -> devolve apenas a mensagem que atravessa.
//!
//! Nenhum modelo fica residente entre turnos. Numa maquina de 32 GB sem GPU,
//! essa e a diferenca entre rodar um gerente de 27B e nao rodar nada.

use serde::Serialize;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};

use super::prompts;
use super::roles::{Network, Role};
use super::transcript::{self, RunPaths, TurnRecord};
use crate::hardware;
use crate::ollama::{catalog, client};

pub const EVENT: &str = "postly://estagio";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    MedindoMemoria,
    EscolhendoModelo,
    BaixandoModelo,
    Pensando,
    Descarregando,
    Concluido,
    Falhou,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageEvent {
    pub step: usize,
    pub role: &'static str,
    pub network: Option<&'static str>,
    pub stage: Stage,
    pub model: Option<String>,
    pub detail: String,
    pub available_ram_bytes: u64,
    pub percent: Option<f32>,
    /// A mensagem que atravessa para o proximo cargo, no evento de conclusao.
    /// Truncada: a interface mostra o inicio do despacho, o arquivo .md guarda
    /// a conversa inteira.
    pub handoff: Option<String>,
}

fn emit(app: &AppHandle, event: StageEvent) {
    let _ = app.emit(EVENT, event);
}

pub struct AgentTurn<'a> {
    pub app: &'a AppHandle,
    pub run: &'a RunPaths,
    pub step: usize,
    pub role: Role,
    pub network: Option<Network>,
    pub system: String,
    pub prompt: String,
    /// Cargo que precisa devolver estrutura, e nao prosa.
    pub json_mode: bool,
    /// Raciocinio explicito antes de responder. Melhora decisao estrategica e
    /// custa caro em CPU; nunca vale para quem so devolve JSON.
    pub pensar: bool,
    /// Imagens em base64 para modelos com visao (o auditor conferindo a arte).
    pub images: Vec<String>,
}

/// A falha de um turno rodado por um CLI de fora.
///
/// Existe para o `executar_fora` nao precisar de um `match` por provedor em
/// cada ponto onde trata erro. A distincao que interessa e uma so, e e a mesma
/// nos dois: acabou a cota (que pode virar espera) ou foi outra coisa (que
/// encerra). Reconhecer isso pelo texto da mensagem quebraria na primeira
/// traducao — por isso enum, e nao `String`.
enum FalhaExterna {
    Limite(crate::claude::limite::Limite),
    Outro(String),
}

impl From<crate::claude::ErroTurno> for FalhaExterna {
    fn from(e: crate::claude::ErroTurno) -> Self {
        match e {
            crate::claude::ErroTurno::Limite(l) => FalhaExterna::Limite(l),
            crate::claude::ErroTurno::Outro(m) => FalhaExterna::Outro(m),
        }
    }
}

impl From<crate::gemini_cli::ErroTurno> for FalhaExterna {
    fn from(e: crate::gemini_cli::ErroTurno) -> Self {
        match e {
            crate::gemini_cli::ErroTurno::Limite(l) => FalhaExterna::Limite(l),
            crate::gemini_cli::ErroTurno::Outro(m) => FalhaExterna::Outro(m),
        }
    }
}

impl FalhaExterna {
    /// A mensagem que vai para a trilha, com o nome de quem falhou.
    ///
    /// O nome do provedor entra aqui e nao no tipo porque a mesma falha pode
    /// vir de qualquer um dos dois, e quem le a tela precisa saber qual foi.
    fn explicar(self, provedor: &str) -> String {
        match self {
            FalhaExterna::Limite(l) => format!(
                "{} {}",
                crate::idioma::msg(
                    &format!("A cota do {provedor} acabou."),
                    &format!("The {provedor} quota ran out."),
                ),
                l.evidencia.lines().next().unwrap_or("")
            ),
            FalhaExterna::Outro(m) => m,
        }
    }
}

pub struct AgentResult {
    pub model: String,
    pub raw: String,
    /// A unica mensagem que atravessa para o proximo cargo.
    pub handoff: String,
    pub json: Option<serde_json::Value>,
    pub transcript_path: String,
    pub warnings: Vec<String>,
}

impl<'a> AgentTurn<'a> {
    pub async fn execute(mut self) -> Result<AgentResult, String> {
        let started = Instant::now();

        // Skills da pessoa entram no system antes de qualquer coisa acontecer,
        // e valem para os dois provedores.
        let extra = crate::prefs::load().bloco_de_skills(self.role.slug());
        if !extra.is_empty() {
            self.system = format!("{}\n\n{}", self.system, extra);
        }
        let mut warnings = Vec::new();
        let role_label = self.role.label();
        let net_slug = self.network.map(|n| n.slug());

        let base = |stage: Stage, detail: String, ram: u64| StageEvent {
            step: self.step,
            role: role_label,
            network: net_slug,
            stage,
            model: None,
            detail,
            available_ram_bytes: ram,
            percent: None,
            handoff: None,
        };

        // Provedor externo: o caminho e outro e muito mais curto. Nao ha
        // memoria para medir, modelo para baixar nem nada para descarregar,
        // porque a inferencia acontece fora desta maquina.
        let provedor = crate::prefs::load().provedor;
        if provedor.externo() {
            return self.executar_fora(provedor, started, warnings).await;
        }

        // 1. Quanta memoria temos AGORA. Esta medicao acontece antes de todo
        //    turno, nao uma vez no boot: o navegador do Playwright pode ter
        //    subido no meio da campanha e comido varios GB.
        let perfil = hardware::compute_profile();
        let ram = perfil.ram.clone();
        emit(
            self.app,
            base(
                Stage::MedindoMemoria,
                format!(
                    "{} livres de {} totais",
                    hardware::human(ram.available_bytes),
                    hardware::human(ram.total_bytes)
                ),
                ram.available_bytes,
            ),
        );

        // 2. O nivel do modelo e proporcional ao que o cargo entrega. Damos
        //    preferencia ao que ja esta baixado; so caimos para download quando
        //    nada instalado serve ao nivel exigido.
        let installed = client::installed_models().await;
        let orcamento = perfil.live_budget_bytes;
        // Relido a cada turno, e nao guardado no comeco da campanha: a pessoa
        // pode trocar de modo com a campanha rodando, e a troca deve valer do
        // proximo cargo em diante.
        let modo = crate::prefs::load().modo;

        // Escolha manual, quando existe, vence a automatica: quem ligou a
        // configuracao avancada quer aquele modelo naquele cargo. O aviso sai
        // se ele nao couber na memoria deste instante, mas a escolha e mantida
        // em vez de trocada pelas costas.
        let manual = crate::prefs::load().modelo_de(self.role.slug());
        let (spec, degraded) = match manual {
            Some(escolhido) => {
                let aviso = (!escolhido.fits(orcamento)).then(|| {
                    format!(
                        "{} foi escolhido a mao e precisa de {}, mas ha {} livres. \
                         O turno vai usar disco como memoria e ficar lento.",
                        escolhido.label,
                        hardware::human(escolhido.footprint_bytes()),
                        hardware::human(orcamento)
                    )
                });
                (escolhido, aviso)
            }
            None => catalog::pick(self.role.tier(), orcamento, perfil.mode, modo, true, &installed)
                .or_else(|| {
                    catalog::pick(self.role.tier(), orcamento, perfil.mode, modo, false, &installed)
                })
            .ok_or_else(|| {
                format!(
                    "Nenhum modelo cabe em {} de memoria livre. Rode a otimizacao ou feche alguns programas.",
                    hardware::human(orcamento)
                )
            })?,
        };

        if let Some(warn) = &degraded {
            warnings.push(warn.clone());
        }

        let already = installed
            .iter()
            .any(|t| t == spec.tag || t.trim_end_matches(":latest") == spec.tag);
        emit(
            self.app,
            StageEvent {
                model: Some(spec.tag.to_string()),
                ..base(
                    Stage::EscolhendoModelo,
                    format!(
                        "{} para o cargo de {} (nivel {:?}, ocupa {}, ~{:.1} tok/s em {})",
                        spec.label,
                        role_label,
                        self.role.tier(),
                        hardware::human(spec.footprint_bytes()),
                        crate::hardware::accelerator::estimated_tokens_per_second(
                            perfil.mode,
                            spec.active_params_b
                        ),
                        perfil.mode_label
                    ),
                    ram.available_bytes,
                )
            },
        );

        // 3. Download sob demanda, com progresso.
        if !already {
            let app = self.app.clone();
            let step = self.step;
            let tag = spec.tag.to_string();
            client::pull(spec.tag, move |progress| {
                emit(
                    &app,
                    StageEvent {
                        step,
                        role: role_label,
                        network: net_slug,
                        stage: Stage::BaixandoModelo,
                        model: Some(tag.clone()),
                        detail: progress.status.clone(),
                        available_ram_bytes: 0,
                        percent: Some(progress.percent),
                        handoff: None,
                    },
                );
            })
            .await?;
        }

        // 4. Sobe, responde, e o keep_alive:"0" ja descarrega ao terminar.
        emit(
            self.app,
            StageEvent {
                model: Some(spec.tag.to_string()),
                ..base(
                    Stage::Pensando,
                    format!("{} trabalhando", role_label),
                    ram.available_bytes,
                )
            },
        );

        // Cargo que devolve JSON nunca pensa: o orcamento de tokens iria inteiro
        // para o raciocinio e a resposta voltaria vazia.
        let pensar = self.pensar && !self.json_mode;

        let options = client::GenerateOptions {
            temperature: self.role.temperatura(),
            num_ctx: 8192,
            // Pensar consome orcamento antes de a resposta comecar.
            num_predict: if pensar { 6144 } else { 2048 },
        };

        let response = client::generate(
            spec.tag,
            Some(&self.system),
            &self.prompt,
            options,
            self.json_mode,
            pensar,
            if spec.vision {
                self.images.clone()
            } else {
                Vec::new()
            },
        )
        .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                emit(
                    self.app,
                    base(
                        Stage::Falhou,
                        e.clone(),
                        hardware::snapshot().available_bytes,
                    ),
                );
                let _ = client::unload(spec.tag).await;
                return Err(e);
            }
        };

        if response.response.trim().is_empty() {
            let motivo = if response.thinking.trim().is_empty() {
                format!("{} devolveu resposta vazia.", spec.label)
            } else {
                format!(
                    "{} gastou o orcamento inteiro raciocinando e nao chegou a responder. \
                     Desligue o raciocinio estendido ou use um modelo menor neste cargo.",
                    spec.label
                )
            };
            emit(
                self.app,
                base(
                    Stage::Falhou,
                    motivo.clone(),
                    hardware::snapshot().available_bytes,
                ),
            );
            return Err(motivo);
        }

        if !self.images.is_empty() && !spec.vision {
            warnings.push(format!(
                "{} nao enxerga imagem; a auditoria visual foi feita apenas sobre o texto.",
                spec.label
            ));
        }

        // 5. Fecha a sessao de forma explicita. O keep_alive ja faz isso, mas
        //    uma chamada extra e barata e garante que nada ficou preso.
        emit(
            self.app,
            StageEvent {
                model: Some(spec.tag.to_string()),
                ..base(Stage::Descarregando, format!("liberando {}", spec.label), 0)
            },
        );
        let _ = client::unload(spec.tag).await;

        // 6. A conversa inteira vai para o disco; so a mensagem atravessa.
        let raw = response.response.clone();
        let (handoff, handoff_warning) = if self.json_mode {
            (raw.trim().to_string(), None)
        } else {
            prompts::extract_handoff(&raw)
        };
        if let Some(w) = handoff_warning {
            warnings.push(w);
        }
        let json = if self.json_mode {
            prompts::extract_json(&raw)
        } else {
            None
        };
        if self.json_mode && json.is_none() {
            warnings.push("O modelo nao devolveu JSON valido nesta rodada.".to_string());
        }

        let transcript_path = transcript::write_turn(
            self.run,
            &TurnRecord {
                step: self.step,
                role: self.role,
                network: self.network,
                model: spec.tag,
                system: &self.system,
                prompt: &self.prompt,
                output: &raw,
                thinking: &response.thinking,
                handoff: &handoff,
                elapsed_ms: started.elapsed().as_millis(),
                tokens_per_second: response.tokens_per_second(),
                ram_budget_bytes: orcamento,
                degraded: degraded.clone(),
            },
        )?;

        emit(
            self.app,
            StageEvent {
                model: Some(spec.tag.to_string()),
                handoff: Some(resumo(&handoff)),
                ..base(
                    Stage::Concluido,
                    format!(
                        "{} concluiu em {:.0}s a {:.1} tokens/s",
                        role_label,
                        started.elapsed().as_secs_f32(),
                        response.tokens_per_second()
                    ),
                    hardware::snapshot().available_bytes,
                )
            },
        );

        Ok(AgentResult {
            model: spec.tag.to_string(),
            raw,
            handoff,
            json,
            transcript_path,
            warnings,
        })
    }

    /// O mesmo turno, executado por um CLI de fora em vez do Ollama.
    ///
    /// A saida precisa ser identica em forma: mesma mensagem que atravessa,
    /// mesmo JSON quando o cargo devolve estrutura, mesma transcricao em disco.
    /// O orquestrador nao sabe nem precisa saber quem executou.
    ///
    /// Claude Code e Gemini CLI dividem este caminho porque o que muda entre
    /// eles cabe em tres pontos — o nome do modelo, a chamada do turno e o que
    /// a trilha mostra no fim. Duplicar o metodo por provedor faria toda
    /// correcao daqui precisar ser feita duas vezes, e a segunda seria
    /// esquecida.
    async fn executar_fora(
        self,
        provedor: crate::prefs::Provedor,
        started: Instant,
        mut warnings: Vec<String>,
    ) -> Result<AgentResult, String> {
        use crate::prefs::Provedor as P;

        let tier = self.role.tier();
        let modelo = match provedor {
            P::GeminiCli => crate::gemini_cli::modelo_do_nivel(tier),
            _ => crate::claude::modelo_do_nivel(tier),
        };
        let nome_do_provedor = match provedor {
            P::GeminiCli => "Gemini CLI",
            _ => "Claude Code",
        };
        let role_label = self.role.label();
        let net_slug = self.network.map(|n| n.slug());
        let step = self.step;

        // Closure propria: a do `execute` empresta `self`, e aqui `self` e
        // consumido.
        let base = move |stage: Stage, detail: String, ram: u64| StageEvent {
            step,
            role: role_label,
            network: net_slug,
            stage,
            model: None,
            detail,
            available_ram_bytes: ram,
            percent: None,
            handoff: None,
        };

        emit(
            self.app,
            StageEvent {
                model: Some(modelo.to_string()),
                ..base(
                    Stage::Pensando,
                    format!("{role_label} trabalhando no {nome_do_provedor}"),
                    0,
                )
            },
        );

        // Teto generoso: um briefing longo em Opus leva minutos, e derrubar o
        // turno por impaciencia gastaria o custo sem entregar nada.
        // A cota pode acabar no meio da campanha, e ai a resposta nao e
        // desistir: e perguntar. Quem escolhe esperar dorme dentro de
        // `pausar_e_esperar` ate a cota voltar, e o turno e refeito uma vez.
        // Uma vez so — se o limite reaparecer logo depois de voltar, insistir
        // viraria laco, e a campanha deve parar com o motivo na tela.
        let mut tentativa = 0;
        let (raw, custo_usd) = loop {
            // O unico ponto em que os dois provedores divergem de verdade: a
            // chamada e o custo. Tudo o que vem depois — o limite, a espera, a
            // transcricao, o despacho — e identico, e por isso vive fora do
            // `match`.
            //
            // O custo e `Option` porque o Gemini CLI nao reporta o valor do
            // turno como o Claude Code reporta. Mostrar `USD 0,000` ali seria
            // afirmar que o turno saiu de graca, e ele nao saiu: gastou cota da
            // assinatura de quem usa. A trilha omite o campo em vez de mentir.
            let saida: Result<(String, Option<f64>), FalhaExterna> = match provedor {
                P::GeminiCli => crate::gemini_cli::turno(tier, &self.system, &self.prompt, 900)
                    .await
                    .map(|t| (t.texto, None))
                    .map_err(FalhaExterna::from),
                _ => crate::claude::turno(tier, &self.system, &self.prompt, 900)
                    .await
                    .map(|t| (t.texto, Some(t.custo_usd)))
                    .map_err(FalhaExterna::from),
            };

            match saida {
                Ok(t) => break t,
                Err(FalhaExterna::Limite(l)) if tentativa == 0 => {
                    tentativa += 1;
                    emit(
                        self.app,
                        base(
                            Stage::Falhou,
                            crate::idioma::msg(
                                &format!(
                                    "A cota do {nome_do_provedor} acabou. Esperando a sua decisao."
                                ),
                                &format!(
                                    "The {nome_do_provedor} quota ran out. Waiting for your decision."
                                ),
                            ),
                            0,
                        ),
                    );
                    let estado = self.app.state::<crate::state::AppState>();
                    if !crate::claude::limite::pausar_e_esperar(self.app, &estado, &l).await {
                        let msg = FalhaExterna::Limite(l).explicar(nome_do_provedor);
                        emit(self.app, base(Stage::Falhou, msg.clone(), 0));
                        return Err(msg);
                    }
                }
                // Todo o resto encerra, inclusive o limite que reaparece na
                // segunda tentativa: insistir ali viraria laco, e a campanha
                // deve parar com o motivo na tela.
                Err(e) => {
                    let msg = e.explicar(nome_do_provedor);
                    emit(self.app, base(Stage::Falhou, msg.clone(), 0));
                    return Err(msg);
                }
            }
        };
        let (handoff, handoff_warning) = if self.json_mode {
            (raw.trim().to_string(), None)
        } else {
            prompts::extract_handoff(&raw)
        };
        if let Some(w) = handoff_warning {
            warnings.push(w);
        }
        let json = if self.json_mode {
            prompts::extract_json(&raw)
        } else {
            None
        };
        if self.json_mode && json.is_none() {
            warnings.push(crate::idioma::msg(
                "O modelo nao devolveu JSON valido nesta rodada.",
                "The model did not return valid JSON this round.",
            ));
        }
        if !self.images.is_empty() {
            warnings.push(crate::idioma::msg(
                &format!(
                    "As referencias em imagem nao vao para o {nome_do_provedor} nesta \
                     versao; elas entraram apenas como descricao no prompt."
                ),
                &format!(
                    "Image references are not sent to {nome_do_provedor} in this version; \
                     they went in as text description only."
                ),
            ));
        }

        let transcript_path = transcript::write_turn(
            self.run,
            &TurnRecord {
                step: self.step,
                role: self.role,
                network: self.network,
                model: modelo,
                system: &self.system,
                prompt: &self.prompt,
                output: &raw,
                thinking: "",
                handoff: &handoff,
                elapsed_ms: started.elapsed().as_millis(),
                tokens_per_second: 0.0,
                ram_budget_bytes: 0,
                degraded: None,
            },
        )?;

        emit(
            self.app,
            StageEvent {
                model: Some(modelo.to_string()),
                handoff: Some(resumo(&handoff)),
                ..base(
                    Stage::Concluido,
                    match custo_usd {
                        Some(c) => format!(
                            "{role_label} concluiu em {:.0}s · USD {c:.3}",
                            started.elapsed().as_secs_f32()
                        ),
                        None => format!(
                            "{role_label} concluiu em {:.0}s no {nome_do_provedor}",
                            started.elapsed().as_secs_f32()
                        ),
                    },
                    0,
                )
            },
        );

        Ok(AgentResult {
            model: modelo.to_string(),
            raw,
            handoff,
            json,
            transcript_path,
            warnings,
        })
    }
}

/// Recorte do despacho para a interface. O arquivo .md fica com o texto inteiro;
/// aqui basta o suficiente para a pessoa reconhecer o que foi passado adiante.
fn resumo(texto: &str) -> String {
    const TETO: usize = 600;
    let limpo = texto.trim();
    if limpo.chars().count() <= TETO {
        return limpo.to_string();
    }
    let corte: String = limpo.chars().take(TETO).collect();
    // Corta na ultima fronteira de palavra para nao partir no meio de uma.
    match corte.rfind(' ') {
        Some(i) => format!("{}...", &corte[..i]),
        None => format!("{corte}..."),
    }
}
