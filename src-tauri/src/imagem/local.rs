//! Difusao na propria maquina, pelo `sd-cli` do stable-diffusion.cpp.
//!
//! E o unico provedor de arte que nao pede chave e nao manda nada para fora —
//! o que fecha o argumento do produto: com Ollama e este, a campanha inteira
//! roda sem que uma linha saia do computador.
//!
//! O preco e o tempo. Numa maquina sem GPU um modelo turbo leva minutos por
//! imagem, contra segundos de uma API. Por isso nada disso vem instalado: o
//! binario e o modelo somam gigabytes, e so descem quando a pessoa pede.
//!
//! O binario vem pronto das releases do projeto (nao ha compilacao), e os
//! pesos sao GGUF de arquivo unico. O mesmo padrao do Ollama e do Chromium.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;

use super::{GeneratedImage, ImageQuality};

/// Onde o binario e os modelos vivem.
pub fn raiz() -> PathBuf {
    crate::platform::current().data_dir().join("imagem-local")
}

pub fn bin_dir() -> PathBuf {
    raiz().join("bin")
}

pub fn modelos_dir() -> PathBuf {
    raiz().join("modelos")
}

/// O executavel, se ja foi baixado.
pub fn binario() -> Option<PathBuf> {
    let nome = if cfg!(windows) {
        "sd-cli.exe"
    } else {
        "sd-cli"
    };
    let p = bin_dir().join(nome);
    p.is_file().then_some(p)
}

/// Os modelos que ja estao no disco.
pub fn modelos_baixados() -> Vec<String> {
    let Ok(dir) = std::fs::read_dir(modelos_dir()) else {
        return Vec::new();
    };
    dir.flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.ends_with(".gguf").then_some(n)
        })
        .collect()
}

/// Gera a arte na maquina.
///
/// O `steps` vem do modelo, nao da qualidade pedida: um modelo turbo entrega
/// em 4 passos o que um comum precisa de 20, e mandar 20 num turbo so gasta
/// minutos a mais para piorar a imagem.
pub async fn gerar(
    prompt: &str,
    aspect_ratio: &str,
    qualidade: ImageQuality,
    out_dir: &Path,
) -> Result<GeneratedImage, String> {
    let Some(bin) = binario() else {
        return Err(crate::idioma::msg(
            "O gerador local ainda nao foi baixado. Va em Modelos e baixe o motor.",
            "The local generator has not been downloaded yet. Go to Models and download the engine.",
        ));
    };
    let modelos = modelos_baixados();
    let Some(modelo) = modelos.first() else {
        return Err(crate::idioma::msg(
            "Nenhum modelo de imagem baixado. Va em Modelos e baixe um.",
            "No image model downloaded. Go to Models and download one.",
        ));
    };
    let caminho_modelo = modelos_dir().join(modelo);
    let spec = super::catalogo_local::por_arquivo(modelo);

    let (largura, altura) = dimensoes(aspect_ratio, spec.map(|s| s.base).unwrap_or(512));
    let passos = spec.map(|s| s.passos).unwrap_or(20);
    // Qualidade alta rende mais passos, mas com teto: num turbo, passar do
    // dobro do recomendado deixa de melhorar e so cobra tempo.
    let passos = match qualidade {
        ImageQuality::Alta => (passos * 2).min(passos + 8),
        _ => passos,
    };
    let cfg = spec.map(|s| s.cfg).unwrap_or(7.0);

    std::fs::create_dir_all(out_dir).map_err(|e| format!("nao consegui criar {out_dir:?}: {e}"))?;
    let saida = out_dir.join(format!("local-{}.png", chrono::Local::now().timestamp()));

    let mut cmd = Command::new(&bin);
    // As bibliotecas do ggml ficam ao lado do binario, e o carregador nao olha
    // ali por conta propria.
    cmd.env("LD_LIBRARY_PATH", bin_dir());
    let saida_proc = cmd
        .args([
            "-M",
            "img_gen",
            "-m",
            &caminho_modelo.to_string_lossy(),
            "-p",
            prompt,
            "-o",
            &saida.to_string_lossy(),
            "-W",
            &largura.to_string(),
            "-H",
            &altura.to_string(),
            "--steps",
            &passos.to_string(),
            "--cfg-scale",
            &cfg.to_string(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|e| format!("falha ao iniciar o gerador local: {e}"))?;

    if !saida.is_file() {
        let erro = String::from_utf8_lossy(&saida_proc.stderr);
        return Err(format!(
            "{} {}",
            crate::idioma::msg(
                "O gerador local nao produziu imagem.",
                "The local generator produced no image."
            ),
            erro.lines().last().unwrap_or("").trim()
        ));
    }

    let bytes = std::fs::metadata(&saida).map(|m| m.len()).unwrap_or(0);
    Ok(GeneratedImage {
        path: saida.to_string_lossy().to_string(),
        bytes,
        model: spec
            .map(|s| s.nome.to_string())
            .unwrap_or_else(|| modelo.clone()),
        aspect_ratio: aspect_ratio.to_string(),
    })
}

/// Traduz a proporcao da rede para pixels multiplos de 64.
///
/// Difusao trabalha em blocos de 64: pedir 500x500 faz o modelo arredondar por
/// conta propria e devolver algo que nao e o que a rede espera.
fn dimensoes(aspect: &str, base: u32) -> (u32, u32) {
    let (w, h) = match aspect {
        "9:16" => (0.5625_f32, 1.0),
        "16:9" => (1.0, 0.5625),
        "4:5" => (0.8, 1.0),
        "1.91:1" => (1.0, 0.524),
        _ => (1.0, 1.0),
    };
    let arred = |v: f32| ((v * base as f32 / 64.0).round().max(1.0) as u32) * 64;
    (arred(w), arred(h))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn as_dimensoes_sao_sempre_multiplas_de_64() {
        for a in ["1:1", "9:16", "16:9", "4:5", "1.91:1", "desconhecida"] {
            for base in [512, 768, 1024] {
                let (w, h) = dimensoes(a, base);
                assert_eq!(w % 64, 0, "{a} em {base}: largura {w}");
                assert_eq!(h % 64, 0, "{a} em {base}: altura {h}");
                assert!(w > 0 && h > 0);
            }
        }
    }

    #[test]
    fn a_proporcao_pedida_e_respeitada_de_perto() {
        // Arredondar para multiplo de 64 desloca a proporcao; o teste garante
        // que o deslocamento e pequeno, e nao que a imagem sai quadrada.
        for (a, alvo) in [("9:16", 0.5625_f32), ("16:9", 1.0 / 0.5625), ("4:5", 0.8)] {
            let (w, h) = dimensoes(a, 768);
            let real = w as f32 / h as f32;
            assert!(
                (real - alvo).abs() / alvo < 0.12,
                "{a}: pedido {alvo:.3}, saiu {real:.3} ({w}x{h})"
            );
        }
    }
}

// ──────────────────────────────────────────────────────── baixar o que falta

use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
pub struct ProgressoLocal {
    /// `motor` ou o id do modelo — a tela precisa saber qual barra mover.
    pub alvo: String,
    pub baixado: u64,
    pub total: u64,
    pub percent: u8,
}

/// Baixa o executavel do stable-diffusion.cpp para este sistema.
///
/// So a variante de CPU. As de CUDA e ROCm passam de 250 MB e exigem driver
/// compativel — transformariam "baixar o motor" numa sessao de suporte, e quem
/// tem placa boa provavelmente prefere uma API mesmo.
pub async fn baixar_motor(app: AppHandle) -> Result<String, String> {
    let Some(variante) = super::catalogo_local::url_do_motor() else {
        return Err(crate::idioma::msg(
            "Sem build pronto para este sistema.",
            "No prebuilt binary for this system.",
        ));
    };

    // A release publica um nome com o hash do commit, entao o link fixo nao
    // serve: e preciso perguntar qual e o arquivo da versao mais recente.
    let api = "https://api.github.com/repos/leejet/stable-diffusion.cpp/releases/latest";
    let cliente = reqwest::Client::builder()
        .user_agent("postly")
        .build()
        .map_err(|e| e.to_string())?;
    let corpo: serde_json::Value = cliente
        .get(api)
        .send()
        .await
        .map_err(|e| format!("nao consegui falar com o GitHub: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let url = corpo["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find_map(|a| {
                let nome = a["name"].as_str()?;
                (nome.contains(variante) && nome.ends_with(".zip"))
                    .then(|| a["browser_download_url"].as_str())?
            })
        })
        .ok_or_else(|| format!("a release mais recente nao traz o pacote para {variante}"))?
        .to_string();

    let zip = raiz().join("motor.zip");
    std::fs::create_dir_all(raiz()).map_err(|e| e.to_string())?;
    baixar_com_progresso(&app, &cliente, &url, &zip, "motor").await?;

    // Descompacta com o `unzip`/`tar` do sistema: trazer um crate de zip para
    // uma operacao que acontece uma vez na vida do app seria peso permanente
    // por conveniencia momentanea.
    std::fs::create_dir_all(bin_dir()).map_err(|e| e.to_string())?;
    let ok = std::process::Command::new(if cfg!(windows) { "tar" } else { "unzip" })
        .args(if cfg!(windows) {
            vec![
                "-xf".to_string(),
                zip.to_string_lossy().to_string(),
                "-C".to_string(),
                bin_dir().to_string_lossy().to_string(),
            ]
        } else {
            vec![
                "-oq".to_string(),
                zip.to_string_lossy().to_string(),
                "-d".to_string(),
                bin_dir().to_string_lossy().to_string(),
            ]
        })
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let _ = std::fs::remove_file(&zip);
    if !ok {
        return Err(crate::idioma::msg(
            "Nao consegui descompactar o motor.",
            "Could not unpack the engine.",
        ));
    }

    #[cfg(unix)]
    if let Some(b) = binario() {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&b, std::fs::Permissions::from_mode(0o755));
    }

    binario()
        .map(|b| b.to_string_lossy().to_string())
        .ok_or_else(|| {
            crate::idioma::msg(
                "O pacote baixou, mas o executavel nao apareceu onde devia.",
                "The package downloaded, but the executable is not where it should be.",
            )
        })
}

/// Baixa os pesos de um modelo do catalogo.
pub async fn baixar_modelo(app: AppHandle, id: String) -> Result<String, String> {
    let spec =
        super::catalogo_local::por_id(&id).ok_or_else(|| format!("modelo desconhecido: {id}"))?;
    std::fs::create_dir_all(modelos_dir()).map_err(|e| e.to_string())?;
    let destino = modelos_dir().join(spec.arquivo);
    if destino.is_file() {
        return Ok(spec.arquivo.to_string());
    }

    // Grava em `.parcial` e so renomeia no fim: uma queda de conexao no meio
    // deixaria um GGUF truncado com o nome final, e o app o trataria como
    // baixado — falhando na primeira geracao, sem dizer por que.
    let parcial = destino.with_extension("parcial");
    let cliente = reqwest::Client::builder()
        .user_agent("postly")
        .build()
        .map_err(|e| e.to_string())?;
    baixar_com_progresso(&app, &cliente, spec.url, &parcial, &id).await?;
    std::fs::rename(&parcial, &destino).map_err(|e| e.to_string())?;
    Ok(spec.arquivo.to_string())
}

async fn baixar_com_progresso(
    app: &AppHandle,
    cliente: &reqwest::Client,
    url: &str,
    destino: &Path,
    alvo: &str,
) -> Result<(), String> {
    use futures_util::StreamExt;

    let resp = cliente
        .get(url)
        .send()
        .await
        .map_err(|e| format!("falha ao baixar: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("o servidor respondeu {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(0);
    let mut arquivo = std::fs::File::create(destino).map_err(|e| e.to_string())?;
    let mut fluxo = resp.bytes_stream();
    let mut baixado = 0u64;
    let mut ultimo = 0u8;

    while let Some(pedaco) = fluxo.next().await {
        let pedaco = pedaco.map_err(|e| format!("a conexao caiu no meio: {e}"))?;
        use std::io::Write;
        arquivo.write_all(&pedaco).map_err(|e| e.to_string())?;
        baixado += pedaco.len() as u64;
        let percent = (baixado * 100).checked_div(total).unwrap_or(0) as u8;
        // Um evento por ponto percentual: emitir a cada pedaco encheria a
        // ponte com milhares de mensagens para desenhar a mesma barra.
        if percent != ultimo {
            ultimo = percent;
            let _ = app.emit(
                "postly://imagem-local",
                ProgressoLocal {
                    alvo: alvo.to_string(),
                    baixado,
                    total,
                    percent,
                },
            );
        }
    }
    Ok(())
}
