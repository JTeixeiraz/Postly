//! Ponte com o Playwright.
//!
//! O Playwright vive em Node, entao ele roda como sidecar: um processo filho
//! que fala JSON-lines pelo stdin/stdout. O Rust nao carrega runtime de
//! navegador nenhum — o unico Chromium do sistema e o que o Playwright sobe, e
//! so quando um agente precisa dele.
//!
//! Todo acesso e serializado por um mutex: uma requisicao por vez. Nao ha ganho
//! em paralelizar aqui (o gargalo e a rede e a inferencia) e a serializacao
//! elimina a classe inteira de bug de resposta trocada.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;

use super::orchestrator::roles::Network;
use crate::platform;

#[derive(Debug, Serialize)]
struct Request {
    id: u64,
    cmd: &'static str,
    #[serde(flatten)]
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct Response {
    id: u64,
    ok: bool,
    #[serde(default)]
    data: serde_json::Value,
    #[serde(default)]
    error: Option<String>,
}

struct Channel {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

pub struct BrowserBridge {
    channel: Mutex<Option<Channel>>,
    next_id: std::sync::atomic::AtomicU64,
    sidecar_path: PathBuf,
    profiles_dir: PathBuf,
}

impl BrowserBridge {
    pub fn new(app_root: PathBuf) -> Self {
        Self {
            channel: Mutex::new(None),
            next_id: std::sync::atomic::AtomicU64::new(1),
            sidecar_path: app_root.join("sidecar").join("playwright-agent.mjs"),
            profiles_dir: platform::current().data_dir().join("browser"),
        }
    }

    fn id(&self) -> u64 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Sobe o sidecar sob demanda. Enquanto nenhum agente precisar de navegador,
    /// nao existe processo Node nem Chromium consumindo memoria.
    async fn ensure(&self, guard: &mut Option<Channel>) -> Result<(), String> {
        if guard.is_some() {
            return Ok(());
        }
        let strategy = platform::current();
        if !strategy.node_installed() {
            return Err(
                "Node nao encontrado no PATH. O sidecar do Playwright precisa dele para rodar."
                    .to_string(),
            );
        }
        if !self.sidecar_path.exists() {
            return Err(format!("sidecar ausente em {:?}", self.sidecar_path));
        }
        std::fs::create_dir_all(&self.profiles_dir).map_err(|e| e.to_string())?;

        let mut child = Command::new(strategy.node_binary())
            .arg(&self.sidecar_path)
            .env("POSTLY_PROFILES", &self.profiles_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("falha ao subir o sidecar do Playwright: {e}"))?;

        let stdin = child.stdin.take().ok_or("stdin do sidecar indisponivel")?;
        let stdout = BufReader::new(
            child
                .stdout
                .take()
                .ok_or("stdout do sidecar indisponivel")?,
        );
        *guard = Some(Channel {
            child,
            stdin,
            stdout,
        });
        Ok(())
    }

    async fn call(
        &self,
        cmd: &'static str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let mut guard = self.channel.lock().await;
        self.ensure(&mut guard).await?;
        let channel = guard.as_mut().ok_or("canal do sidecar nao inicializado")?;

        let id = self.id();
        let request = Request { id, cmd, payload };
        let mut line = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        line.push('\n');
        channel
            .stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("sidecar nao aceitou a requisicao: {e}"))?;
        channel.stdin.flush().await.map_err(|e| e.to_string())?;

        // Le ate encontrar a resposta com o id correspondente; linhas de log do
        // sidecar que nao sejam JSON valido sao ignoradas.
        let mut buffer = String::new();
        loop {
            buffer.clear();
            let read = channel
                .stdout
                .read_line(&mut buffer)
                .await
                .map_err(|e| format!("sidecar interrompeu a resposta: {e}"))?;
            if read == 0 {
                *guard = None;
                return Err("o sidecar do Playwright encerrou inesperadamente".into());
            }
            let Ok(resp) = serde_json::from_str::<Response>(buffer.trim()) else {
                continue;
            };
            if resp.id != id {
                continue;
            }
            return if resp.ok {
                Ok(resp.data)
            } else {
                Err(resp
                    .error
                    .unwrap_or_else(|| "erro desconhecido no sidecar".into()))
            };
        }
    }

    /// Abre o navegador na rede indicada, reaproveitando o perfil persistido.
    /// Se ja houver sessao valida no perfil, nao pede login de novo.
    pub async fn open(&self, network: Network, headless: bool) -> Result<BrowserSession, String> {
        let data = self
            .call(
                "open",
                serde_json::json!({
                    "network": network.slug(),
                    "url": network.home_url(),
                    "headless": headless
                }),
            )
            .await?;
        Ok(BrowserSession {
            logged_in: data
                .get("loggedIn")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            url: data
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        })
    }

    pub async fn login(
        &self,
        network: Network,
        username: &str,
        password: &str,
    ) -> Result<BrowserSession, String> {
        let data = self
            .call(
                "login",
                serde_json::json!({
                    "network": network.slug(),
                    "username": username,
                    "password": password
                }),
            )
            .await?;
        Ok(BrowserSession {
            logged_in: data
                .get("loggedIn")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            url: data
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        })
    }

    /// Observacao de campo para o gerente: o que a conta e o nicho estao
    /// mostrando agora. Vira texto e entra no prompt.
    pub async fn research(&self, network: Network, limit: u32) -> Result<String, String> {
        let data = self
            .call(
                "research",
                serde_json::json!({ "network": network.slug(), "limit": limit }),
            )
            .await?;
        Ok(data
            .get("report")
            .and_then(|v| v.as_str())
            .unwrap_or("Sem observacao coletada.")
            .to_string())
    }

    /// Numeros visiveis das ultimas publicacoes da propria conta.
    ///
    /// Volta vazio quando a rede mudou o layout, e isso nao e erro: a tela
    /// oferece o registro manual, que e o caminho que nunca quebra.
    pub async fn metrics(&self, network: Network, limit: u32) -> Result<Vec<PostColhido>, String> {
        let data = self
            .call(
                "metrics",
                serde_json::json!({ "network": network.slug(), "limit": limit }),
            )
            .await?;
        Ok(
            serde_json::from_value(data.get("posts").cloned().unwrap_or(serde_json::json!([])))
                .unwrap_or_default(),
        )
    }

    pub async fn publish(
        &self,
        network: Network,
        image_path: &str,
        caption: &str,
        dry_run: bool,
    ) -> Result<PublishOutcome, String> {
        let data = self
            .call(
                "publish",
                serde_json::json!({
                    "network": network.slug(),
                    "imagePath": image_path,
                    "caption": caption,
                    "dryRun": dry_run
                }),
            )
            .await?;
        Ok(PublishOutcome {
            published: data
                .get("published")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            detail: data
                .get("detail")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            screenshot: data
                .get("screenshot")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }

    /// Fecha o navegador e derruba o sidecar, devolvendo a memoria.
    pub async fn shutdown(&self) {
        let mut guard = self.channel.lock().await;
        if guard.is_some() {
            let _ = self
                .call_locked(&mut guard, "shutdown", serde_json::json!({}))
                .await;
        }
        if let Some(mut channel) = guard.take() {
            let _ = channel.child.kill().await;
        }
    }

    async fn call_locked(
        &self,
        guard: &mut Option<Channel>,
        cmd: &'static str,
        payload: serde_json::Value,
    ) -> Result<(), String> {
        let Some(channel) = guard.as_mut() else {
            return Ok(());
        };
        let request = Request {
            id: self.id(),
            cmd,
            payload,
        };
        let mut line = serde_json::to_string(&request).map_err(|e| e.to_string())?;
        line.push('\n');
        let _ = channel.stdin.write_all(line.as_bytes()).await;
        let _ = channel.stdin.flush().await;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowserSession {
    pub logged_in: bool,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default)]
pub struct PostColhido {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub resumo: String,
    #[serde(default)]
    pub publicado_em: String,
    #[serde(default)]
    pub curtidas: u64,
    #[serde(default)]
    pub comentarios: u64,
    #[serde(default)]
    pub impressoes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublishOutcome {
    pub published: bool,
    pub detail: String,
    pub screenshot: Option<String>,
}
