//! A ponte com o Remotion, do lado do Rust.
//!
//! O Remotion vive em Node, entao ele roda no sidecar — o mesmo padrao do
//! Playwright, e pela mesma razao: a biblioteca oficial e Node e nao ha porta
//! para Rust que valha o risco.
//!
//! NENHUM DOWNLOAD NOVO. O Remotion renderiza dentro de um Chromium headless, e
//! por padrao baixa um so dele. Mas o Postly ja provisiona o Chromium do
//! Playwright na primeira abertura, e ele serve — entao o caminho daquele
//! executavel e passado ao render. Sem isso, um app que ja tem dois navegadores
//! em disco baixaria um terceiro, contra o mandato de que nada que a pessoa nao
//! pediu e baixado.
//!
//! PROCESSO POR RENDER, e nao um canal vivo como o do navegador. Um render e
//! uma operacao longa e unica que termina com um arquivo; manter o processo de
//! pe depois disso seria segurar centenas de MB de Node e Chromium para nada.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

pub const EVENT: &str = "postly://render";

/// O que o sidecar manda enquanto trabalha.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct ProgressoRender {
    /// `empacotando` enquanto o bundler roda, `renderizando` depois.
    ///
    /// Sao duas fases porque a primeira nao tem porcentagem util e a segunda
    /// tem. Uma barra so ficaria parada em zero durante o bundle e a tela
    /// pareceria travada.
    pub fase: String,
    pub percent: f32,
    #[serde(default)]
    pub detalhe: String,
}

#[derive(Debug, Deserialize)]
struct Resposta {
    ok: bool,
    #[serde(default)]
    arquivo: Option<String>,
    #[serde(default)]
    duracao_s: Option<f32>,
    #[serde(default)]
    erro: Option<String>,
}

/// O video pronto.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VideoPronto {
    pub arquivo: String,
    pub bytes: u64,
    /// Duracao medida pelo sidecar depois do render — nao a soma das cenas.
    ///
    /// A diferenca importa: a soma e o que o roteiro pediu, e esta e o que o
    /// arquivo tem. Mostrar a soma seria repetir a intencao em vez de relatar
    /// o resultado.
    pub duracao_s: f32,
}

/// Renderiza o roteiro e devolve o caminho do `.mp4`.
pub async fn renderizar(
    app: &AppHandle,
    app_root: &Path,
    roteiro: &super::spec::Roteiro,
    projeto: &super::assets::Projeto,
) -> Result<VideoPronto, String> {
    let estrategia = crate::platform::current();
    if !estrategia.node_installed() {
        return Err(crate::idioma::msg(
            "Node nao encontrado no PATH. O render do Remotion precisa dele.",
            "Node was not found on the PATH. The Remotion render needs it.",
        ));
    }

    let agente = app_root.join("sidecar").join("remotion-agent.mjs");
    if !agente.is_file() {
        return Err(crate::idioma::msg(
            "O sidecar de render nao esta instalado. Rode `npm ci --prefix sidecar`.",
            "The render sidecar is not installed. Run `npm ci --prefix sidecar`.",
        ));
    }

    // O Chromium do Playwright, quando ele ja esta em disco. `None` deixa o
    // Remotion resolver por conta propria — o que pode disparar um download, e
    // por isso a tela avisa antes em vez de fazer isso pelas costas.
    let chromium = match crate::navegador::status() {
        s if s.state == crate::navegador::EstadoNavegador::Pronto => s.caminho,
        _ => None,
    };

    let (largura, altura) = roteiro.dimensoes();
    let saida = destino(projeto);

    let pedido = serde_json::json!({
        "cmd": "renderizar",
        "projeto": projeto.caminho,
        "roteiro": roteiro,
        "saida": saida.to_string_lossy(),
        "largura": largura,
        "altura": altura,
        "chromium": chromium,
        "raiz_motion": app_root.join("motion").to_string_lossy(),
    });

    let mut filho = Command::new("node")
        .arg(&agente)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("falha ao iniciar o sidecar de render: {e}"))?;

    // O BLOCO EXISTE PARA O HANDLE SER DERRUBADO AQUI, e nao e estilo: e a
    // correcao de um deadlock que travava o render para sempre.
    //
    // O sidecar le o stdin ate o EOF (`process.stdin.on("end", ...)`). Um
    // `shutdown()` no `ChildStdin` do tokio NAO fecha o descritor — so o drop
    // fecha. Com o handle vivo ate o fim da funcao, o filho ficava eternamente
    // em `epoll_wait` esperando um EOF que nunca chegava: 0% de CPU, nenhum
    // evento de progresso, nenhum erro.
    //
    // Medido rodando o pipeline de verdade. O teste manual do sidecar nao
    // pegava isto porque usava `< arquivo`, que da EOF de graca — a falha so
    // existia pelo caminho do app, que e justamente o que importa.
    {
        let mut entrada = filho
            .stdin
            .take()
            .ok_or_else(|| "o sidecar de render nao aceitou entrada".to_string())?;
        entrada
            .write_all(format!("{pedido}\n").as_bytes())
            .await
            .map_err(|e| format!("falha ao enviar o roteiro: {e}"))?;
        entrada.shutdown().await.ok();
    }

    let stdout = filho
        .stdout
        .take()
        .ok_or_else(|| "o sidecar de render nao respondeu".to_string())?;

    // O stderr e drenado EM PARALELO, numa tarefa propria. Ler as duas saidas
    // em sequencia trava: o Remotion escreve bastante no stderr, e um pipe
    // cheio bloqueia o processo filho — que entao para de escrever no stdout,
    // que e onde estamos esperando. O render ficaria parado para sempre sem
    // erro nenhum, que e o pior jeito de falhar.
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

    // Uma linha JSON por evento: progresso enquanto trabalha, resultado no fim.
    // A ultima linha valida manda — se o sidecar morrer no meio, nao ha
    // resultado e o erro sai do stderr.
    let mut resultado: Option<Resposta> = None;
    let leitura = async {
        while let Ok(Some(linha)) = linhas.next_line().await {
            let linha = linha.trim();
            if linha.is_empty() {
                continue;
            }
            if let Ok(p) = serde_json::from_str::<ProgressoRender>(linha) {
                // O progresso e reconhecido antes da resposta porque as duas
                // sao JSON: uma linha de progresso tambem casaria com
                // `Resposta`, so que com todos os campos em `None`, e a tela
                // perderia a barra.
                let _ = app.emit(EVENT, p);
                continue;
            }
            if let Ok(r) = serde_json::from_str::<Resposta>(linha) {
                resultado = Some(r);
            }
        }
    };

    // TETO DE ESPERA. Sem ele, qualquer coisa que segure o sidecar deixa o
    // video parado para sempre com a tela dizendo "renderizando" — foi
    // exatamente assim que o deadlock do stdin se manifestou. E a mesma regra
    // da espera do turno de movimento: caminho sem teto e caminho que trava.
    //
    // Generoso de proposito: empacotar num processo novo leva minutos, e
    // derrubar um render por impaciencia joga fora o trabalho dos tres cargos.
    if tokio::time::timeout(std::time::Duration::from_secs(1800), leitura)
        .await
        .is_err()
    {
        return Err(crate::idioma::msg(
            "O render passou de 30 minutos sem terminar e foi encerrado.",
            "The render went past 30 minutes without finishing and was stopped.",
        ));
    }

    let status = filho
        .wait()
        .await
        .map_err(|e| format!("falha ao esperar o sidecar: {e}"))?;

    let erro_bruto = dreno.await.unwrap_or_default();

    let Some(r) = resultado else {
        return Err(format!(
            "{} {}",
            crate::idioma::msg(
                "O render terminou sem devolver resultado.",
                "The render finished without returning a result."
            ),
            primeira_linha_util(&erro_bruto)
        ));
    };

    if !r.ok || !status.success() {
        return Err(r
            .erro
            .filter(|e| !e.trim().is_empty())
            .unwrap_or_else(|| primeira_linha_util(&erro_bruto)));
    }

    let arquivo = r.arquivo.unwrap_or_else(|| saida.to_string_lossy().into());
    let bytes = std::fs::metadata(&arquivo).map(|m| m.len()).unwrap_or(0);
    if bytes == 0 {
        // Um arquivo de zero byte e o pior resultado possivel: o render diz que
        // deu certo e a pessoa baixa nada.
        return Err(crate::idioma::msg(
            "O render devolveu um arquivo vazio.",
            "The render returned an empty file.",
        ));
    }

    Ok(VideoPronto {
        arquivo,
        bytes,
        duracao_s: r.duracao_s.unwrap_or_else(|| roteiro.duracao_s()),
    })
}

/// O caminho do `.mp4` desta rodada.
///
/// Carimbado com a hora, e nao um `video.mp4` fixo: renderizar de novo nao pode
/// apagar o que a pessoa acabou de baixar sem querer.
fn destino(projeto: &super::assets::Projeto) -> PathBuf {
    let carimbo = chrono::Local::now().format("%Y%m%d-%H%M%S");
    projeto
        .caminho_de("saida")
        .join(format!("{}-{carimbo}.mp4", projeto.slug))
}

/// A primeira linha do stderr que nao seja rastro de pilha do Node.
fn primeira_linha_util(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("at "))
        .unwrap_or("")
        .chars()
        .take(240)
        .collect()
}
