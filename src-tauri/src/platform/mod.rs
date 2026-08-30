//! Padrao Strategy para abstrair tudo que muda entre Windows, Linux e macOS.
//!
//! Nenhum outro modulo do sistema pode chamar `cfg!(target_os = ...)` nem montar
//! comando de shell na mao. Todo comportamento dependente de SO entra aqui, atras
//! do trait `PlatformStrategy`, e o resto do codigo fala so com o trait.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Windows,
    MacOS,
}

/// Um passo executavel de instalacao/otimizacao, ja resolvido para o SO corrente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    /// Se true, o passo pede elevacao de privilegio e so roda com consentimento.
    pub needs_elevation: bool,
}

impl Step {
    pub fn new(label: &str, program: &str, args: &[&str]) -> Self {
        Self {
            label: label.to_string(),
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            needs_elevation: false,
        }
    }

    /// Roda o passo transmitindo cada linha da saida enquanto ela acontece.
    ///
    /// O `run` normal usa `.output()`, que so retorna quando o comando termina.
    /// Instalar o Ollama baixa perto de um giga: com `.output()` a tela fica
    /// parada minutos sem sinal nenhum de vida, e a pessoa nao consegue
    /// distinguir download lento de processo travado. Aqui cada linha do
    /// instalador vira um evento.
    pub async fn run_streaming<F>(&self, mut on_line: F) -> Result<String, String>
    where
        F: FnMut(&str),
    {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let mut filho = tokio::process::Command::new(&self.program)
            .args(&self.args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("{}: falhou ao executar `{}`: {e}", self.label, self.program))?;

        let saida = filho.stdout.take().ok_or("stdout indisponivel")?;
        let erro = filho.stderr.take().ok_or("stderr indisponivel")?;
        let mut linhas_saida = BufReader::new(saida).lines();
        let mut linhas_erro = BufReader::new(erro).lines();
        let mut ultimas = Vec::new();

        // Os dois canais em paralelo: curl escreve progresso em stderr e o
        // gerenciador de pacote escreve em stdout. Ler so um perde metade.
        loop {
            tokio::select! {
                linha = linhas_saida.next_line() => match linha {
                    Ok(Some(l)) => { on_line(&l); ultimas.push(l); }
                    _ => break,
                },
                linha = linhas_erro.next_line() => match linha {
                    Ok(Some(l)) => { on_line(&l); ultimas.push(l); }
                    _ => break,
                },
            }
        }
        // Drena o que sobrou no outro canal depois que o primeiro fechou.
        while let Ok(Some(l)) = linhas_saida.next_line().await {
            on_line(&l);
            ultimas.push(l);
        }
        while let Ok(Some(l)) = linhas_erro.next_line().await {
            on_line(&l);
            ultimas.push(l);
        }

        let status = filho.wait().await.map_err(|e| e.to_string())?;
        let texto = ultimas.join("\n");
        if status.success() {
            Ok(texto)
        } else {
            Err(format!(
                "{}: {}",
                self.label,
                crate::platform::ultima_linha_util(&texto)
            ))
        }
    }

    pub fn elevated(mut self) -> Self {
        self.needs_elevation = true;
        self
    }

    pub fn run(&self) -> Result<String, String> {
        let out = Command::new(&self.program)
            .args(&self.args)
            .output()
            .map_err(|e| format!("{}: falhou ao executar `{}`: {e}", self.label, self.program))?;
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        if out.status.success() {
            Ok(stdout)
        } else {
            Err(format!(
                "{}: {}",
                self.label,
                if stderr.is_empty() { stdout } else { stderr }
            ))
        }
    }
}

/// Diretorio que a rotina de otimizacao pode limpar, com o quanto ele ocupa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReclaimTarget {
    pub label: String,
    pub path: String,
    pub bytes: u64,
    /// Limpar isto e reversivel (cache regeneravel) ou destrutivo?
    pub safe: bool,
}

/// Contrato unico que cada SO implementa.
pub trait PlatformStrategy: Send + Sync {
    fn id(&self) -> Platform;
    fn label(&self) -> &'static str;

    /// Nome do executavel do Ollama neste SO.
    fn ollama_binary(&self) -> &'static str;

    /// Passos para instalar o Ollama automaticamente neste SO.
    fn ollama_install_steps(&self) -> Vec<Step>;

    /// Passos para subir o servidor do Ollama em background.
    fn ollama_serve_step(&self) -> Step;

    /// Nome do executavel do Node (usado para o sidecar do Playwright).
    fn node_binary(&self) -> &'static str;

    /// O npm e o npx que acompanham o Node. Servem para instalar o sidecar e
    /// baixar o Chromium; no Windows os dois sao scripts `.cmd`, e chamar sem
    /// a extensao devolve "programa nao encontrado".
    fn npm_binary(&self) -> &'static str {
        "npm"
    }
    fn npx_binary(&self) -> &'static str {
        "npx"
    }

    /// Onde o sistema guarda cerebro, transcricoes, midia e cofre.
    fn data_dir(&self) -> PathBuf;

    /// Caches do usuario que podem ser limpos para liberar memoria/disco.
    fn reclaim_targets(&self) -> Vec<ReclaimTarget>;

    /// Passo especifico do SO para devolver paginas de cache ao pool livre.
    /// `None` quando o SO nao expoe isso de forma segura.
    fn drop_caches_step(&self) -> Option<Step>;

    /// Abre um arquivo ou pasta no gerenciador padrao do SO.
    fn open_step(&self, path: &str) -> Step;

    /// Notificacao nativa do sistema.
    ///
    /// Existe porque uma campanha leva dezenas de minutos e a pessoa sai da
    /// frente da tela: um modal atras de outra janela e o mesmo que nao ter
    /// perguntado. Sai pelo notificador do proprio sistema em vez de um plugin,
    /// que traria dependencia e permissao novas para uma chamada so.
    ///
    /// Falhar aqui nunca derruba nada: notificacao e cortesia, e ha ambiente
    /// (servidor sem sessao grafica, container) onde ela simplesmente nao
    /// existe. O modal continua aparecendo de qualquer forma.
    fn notify_step(&self, titulo: &str, corpo: &str) -> Option<Step>;

    /// Placas de video que este SO consegue enxergar, e se o Ollama consegue
    /// usar cada uma. Lista vazia significa inferencia em CPU.
    fn detect_accelerators(&self) -> Vec<crate::hardware::accelerator::Accelerator>;

    /// Resolve um binario no PATH respeitando a convencao do SO.
    fn which(&self, binary: &str) -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;
        let exts: Vec<String> = if self.id() == Platform::Windows {
            std::env::var("PATHEXT")
                .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
                .split(';')
                .map(|s| s.to_lowercase())
                .collect()
        } else {
            vec![String::new()]
        };
        for dir in std::env::split_paths(&path) {
            for ext in &exts {
                let candidate = dir.join(format!("{binary}{ext}"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn ollama_installed(&self) -> bool {
        self.which(self.ollama_binary()).is_some()
    }

    fn node_installed(&self) -> bool {
        self.which(self.node_binary()).is_some()
    }
}

static STRATEGY: OnceLock<Box<dyn PlatformStrategy>> = OnceLock::new();

/// Fabrica: resolve a estrategia do SO corrente uma unica vez por processo.
pub fn current() -> &'static dyn PlatformStrategy {
    STRATEGY
        .get_or_init(|| {
            #[cfg(target_os = "linux")]
            {
                Box::new(linux::LinuxStrategy) as Box<dyn PlatformStrategy>
            }
            #[cfg(target_os = "windows")]
            {
                Box::new(windows::WindowsStrategy) as Box<dyn PlatformStrategy>
            }
            #[cfg(target_os = "macos")]
            {
                Box::new(macos::MacOsStrategy) as Box<dyn PlatformStrategy>
            }
        })
        .as_ref()
}

/// Executa um comando e devolve o stdout, ou `None` se o binario nao existe.
/// Usado pelas sondas de hardware, onde a ausencia do binario e resposta valida
/// e nao erro.
pub(crate) fn sonda(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Soma recursiva do tamanho de um diretorio, com teto de profundidade para
/// nao travar em arvores gigantes.
pub(crate) fn dir_size(path: &std::path::Path, depth: u8) -> u64 {
    if depth == 0 {
        return 0;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_file() {
            total += meta.len();
        } else if meta.is_dir() {
            total += dir_size(&entry.path(), depth - 1);
        }
    }
    total
}

pub(crate) fn target(label: &str, path: PathBuf, safe: bool) -> Option<ReclaimTarget> {
    if !path.is_dir() {
        return None;
    }
    let bytes = dir_size(&path, 4);
    if bytes == 0 {
        return None;
    }
    Some(ReclaimTarget {
        label: label.to_string(),
        path: path.to_string_lossy().to_string(),
        bytes,
        safe,
    })
}

/// A ultima linha com conteudo, para a mensagem de erro nao virar um despejo
/// de log inteiro na tela.
pub fn ultima_linha_util(saida: &str) -> String {
    saida
        .lines()
        .rev()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("sem detalhe")
        .chars()
        .take(200)
        .collect()
}
