//! A medição dos clipes: duração, formato e onde há som.
//!
//! POR QUE MEDIR EM VEZ DE PERGUNTAR AO MODELO. Um modelo de linguagem recebe
//! texto; ele não ouve o arquivo. Pedir "corte as pausas vazias" sem lhe dizer
//! onde elas estão é pedir algo que ele só pode **fingir** atender — e ele
//! finge, cortando em tempos inventados que produzem fala picotada.
//!
//! A medição vem do próprio Remotion (`getSilentParts`), que roda no compositor
//! já instalado para renderizar: nenhum ffmpeg externo, nenhum download novo.
//!
//! Medido num clipe com pausas conhecidas (fala 0–3s, pausa 3–5s, fala 5–8s,
//! pausa 8–10,5s, fala 10,5–13s):
//!
//! ```text
//! com_som: [0–3.02, 5.02–8.01, 10.52–13]  ·  pausas: 2
//! ```
//!
//! Erro de 20 ms. É informação boa o bastante para um corte.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

pub const EVENT: &str = "postly://analise";

/// Um trecho com som, em segundos do clipe original.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Trecho {
    pub de_s: f32,
    pub ate_s: f32,
}

/// Um clipe medido.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clipe {
    pub nome: String,
    #[serde(default)]
    pub duracao_s: f32,
    #[serde(default)]
    pub largura: u32,
    #[serde(default)]
    pub altura: u32,
    #[serde(default)]
    pub fps: f32,
    #[serde(default)]
    pub tem_audio: bool,
    #[serde(default)]
    pub com_som: Vec<Trecho>,
    #[serde(default)]
    pub pausas: usize,
    /// Um clipe ilegível não derruba os outros: quem subiu cinco vídeos com um
    /// corrompido no meio quer os quatro que servem, com aviso sobre o quinto.
    #[serde(default)]
    pub erro: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progresso {
    pub fase: String,
    pub percent: f32,
    #[serde(default)]
    pub detalhe: String,
}

#[derive(Debug, Deserialize)]
struct Resposta {
    ok: bool,
    #[serde(default)]
    clipes: Vec<Clipe>,
    #[serde(default)]
    erro: Option<String>,
}

/// Mede todos os clipes de um projeto.
///
/// Devolve lista vazia quando não há clipe — e isso não é falha: a maioria dos
/// vídeos é montada só com imagem, e um erro ali obrigaria a tratar o caso
/// normal como exceção.
pub async fn medir(
    app: &AppHandle,
    raiz: &Path,
    projeto: &super::assets::Projeto,
) -> Result<Vec<Clipe>, String> {
    if projeto.clipes.is_empty() {
        return Ok(Vec::new());
    }
    if !crate::platform::current().node_installed() {
        return Err(crate::idioma::msg(
            "Node nao encontrado no PATH. A analise dos clipes precisa dele.",
            "Node was not found on the PATH. Clip analysis needs it.",
        ));
    }

    let agente = raiz.join("sidecar").join("analise-agent.mjs");
    if !agente.is_file() {
        return Err(crate::idioma::msg(
            "O analisador de clipes nao esta instalado. Rode `npm ci --prefix sidecar`.",
            "The clip analyser is not installed. Run `npm ci --prefix sidecar`.",
        ));
    }

    let pedido = serde_json::json!({ "projeto": projeto.caminho });

    let mut filho = Command::new("node")
        .arg(&agente)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("falha ao iniciar o analisador: {e}"))?;

    // O bloco fecha o stdin no fim dele. Um `shutdown()` no `ChildStdin` do
    // tokio NAO fecha o descritor — so o drop fecha — e o agente le ate o EOF.
    // Foi exatamente assim que o render travou para sempre uma vez.
    {
        let mut entrada = filho
            .stdin
            .take()
            .ok_or_else(|| "o analisador nao aceitou entrada".to_string())?;
        entrada
            .write_all(format!("{pedido}\n").as_bytes())
            .await
            .map_err(|e| format!("falha ao enviar o pedido: {e}"))?;
        entrada.shutdown().await.ok();
    }

    let stdout = filho
        .stdout
        .take()
        .ok_or_else(|| "o analisador nao respondeu".to_string())?;
    let mut stderr = filho.stderr.take();
    let dreno = tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut buf = String::new();
        if let Some(s) = stderr.as_mut() {
            let _ = s.read_to_string(&mut buf).await;
        }
        buf
    });

    let mut linhas = BufReader::new(stdout).lines();
    let mut resultado: Option<Resposta> = None;
    let leitura = async {
        while let Ok(Some(linha)) = linhas.next_line().await {
            let linha = linha.trim();
            if linha.is_empty() {
                continue;
            }
            if let Ok(p) = serde_json::from_str::<Progresso>(linha) {
                let _ = app.emit(EVENT, p);
                continue;
            }
            if let Ok(r) = serde_json::from_str::<Resposta>(linha) {
                resultado = Some(r);
            }
        }
    };

    // Teto de espera, como o do render: medir um clipe longo leva um tempo, mas
    // um analisador travado não pode segurar o vídeo para sempre.
    if tokio::time::timeout(std::time::Duration::from_secs(900), leitura)
        .await
        .is_err()
    {
        return Err(crate::idioma::msg(
            "A analise dos clipes passou de 15 minutos e foi encerrada.",
            "Clip analysis went past 15 minutes and was stopped.",
        ));
    }

    let erro_bruto = dreno.await.unwrap_or_default();
    let _ = filho.wait().await;

    match resultado {
        Some(r) if r.ok => Ok(r.clipes),
        Some(r) => Err(r
            .erro
            .unwrap_or_else(|| crate::idioma::msg("A analise falhou.", "The analysis failed."))),
        None => Err(format!(
            "{} {}",
            crate::idioma::msg(
                "O analisador nao devolveu resultado.",
                "The analyser returned no result."
            ),
            erro_bruto.lines().next().unwrap_or("")
        )),
    }
}

/// Quanto tempo o corte de todos os trechos com som daria.
///
/// Vai para a tela ao lado do total bruto: a diferença entre os dois é
/// exatamente o que a pessoa ganha cortando pausa, e um número mostra isso
/// melhor que qualquer explicação.
pub fn segundos_com_som(clipes: &[Clipe]) -> f32 {
    clipes
        .iter()
        .flat_map(|c| c.com_som.iter())
        .map(|t| (t.ate_s - t.de_s).max(0.0))
        .sum()
}

pub fn segundos_brutos(clipes: &[Clipe]) -> f32 {
    clipes.iter().map(|c| c.duracao_s).sum()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn clipe(dur: f32, trechos: &[(f32, f32)]) -> Clipe {
        Clipe {
            nome: "t.mp4".into(),
            duracao_s: dur,
            largura: 1920,
            altura: 1080,
            fps: 30.0,
            tem_audio: true,
            com_som: trechos
                .iter()
                .map(|(a, b)| Trecho {
                    de_s: *a,
                    ate_s: *b,
                })
                .collect(),
            pausas: trechos.len().saturating_sub(1),
            erro: None,
        }
    }

    #[test]
    fn a_conta_do_ganho_usa_os_trechos_medidos() {
        // Os números do clipe de prova: 13s brutos, com som em 0–3, 5–8 e
        // 10,5–13 → 8,5s de conteúdo. A tela mostra os dois lado a lado porque
        // "corta as pausas" não diz nada, e "13s viram 8,5s" diz.
        let c = vec![clipe(13.0, &[(0.0, 3.0), (5.0, 8.0), (10.5, 13.0)])];
        assert_eq!(segundos_brutos(&c), 13.0);
        assert_eq!(segundos_com_som(&c), 8.5);
    }

    #[test]
    fn clipe_sem_som_nao_estraga_a_conta() {
        // Um vídeo mudo é material legítimo — b-roll. Ele soma no bruto e não
        // soma no falado, e nenhuma das duas contas pode explodir por isso.
        let c = vec![clipe(10.0, &[]), clipe(4.0, &[(1.0, 3.0)])];
        assert_eq!(segundos_brutos(&c), 14.0);
        assert_eq!(segundos_com_som(&c), 2.0);
    }

    #[test]
    fn trecho_invertido_nao_vira_ganho_negativo() {
        // Defesa contra medição estranha: um intervalo ao contrário subtrairia
        // do total e a tela mostraria um vídeo que encolhe mais que o próprio
        // tamanho.
        let c = vec![clipe(5.0, &[(3.0, 1.0)])];
        assert_eq!(segundos_com_som(&c), 0.0);
    }
}
