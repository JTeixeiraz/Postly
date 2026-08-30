//! Verificação e instalação de atualização.
//!
//! O caminho oficial do Tauri (`tauri-plugin-updater`) faz mais: baixa em
//! segundo plano, valida assinatura e troca o binário sozinho. Ele exige um par
//! de chaves, o CI assinando cada release e um endpoint publicado — e a chave
//! privada precisa existir antes da primeira release assinada.
//!
//! Enquanto isso não existe, este módulo faz o essencial sem infraestrutura
//! nova: pergunta ao GitHub qual é a última versão, compara com a que está
//! rodando e, se houver novidade, baixa o instalador da plataforma e o abre. A
//! troca em si fica com o instalador, que é quem sabe fazê-la em cada sistema.

use crate::platform;
use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const REPO: &str = "JTeixeiraz/Postly";

#[derive(Debug, Clone, Serialize)]
pub struct Atualizacao {
    pub disponivel: bool,
    pub versao_atual: String,
    pub versao_nova: Option<String>,
    pub notas: Option<String>,
    pub url_instalador: Option<String>,
    pub tamanho_bytes: Option<u64>,
}

/// Compara duas versões `x.y.z` numericamente.
///
/// Comparar como texto diria que "0.10.0" é anterior a "0.9.0", e o app pararia
/// de oferecer atualização exatamente quando o projeto passasse da nona versão
/// menor.
fn mais_nova(nova: &str, atual: &str) -> bool {
    // O sufixo depois do hífen é pré-release, e ele torna a versão MENOR:
    // `0.2.0-rc1` vem antes de `0.2.0`. Tratá-lo como mais um número faria o
    // aplicativo empurrar um candidato a lançamento por cima da versão final.
    let quebrar = |v: &str| -> (Vec<u64>, bool) {
        let limpo = v.trim_start_matches('v');
        let (core, resto) = limpo.split_once('-').unwrap_or((limpo, ""));
        let nums = core
            .split('.')
            .map(|p| p.parse().unwrap_or(0))
            .collect::<Vec<u64>>();
        (nums, !resto.is_empty())
    };
    let (a, pre_a) = quebrar(nova);
    let (b, pre_b) = quebrar(atual);

    for i in 0..a.len().max(b.len()) {
        let (x, y) = (
            a.get(i).copied().unwrap_or(0),
            b.get(i).copied().unwrap_or(0),
        );
        if x != y {
            return x > y;
        }
    }
    // Mesmo número: só é mais nova se ela é final e a que roda é pré-release.
    pre_b && !pre_a
}

/// O sufixo do instalador desta plataforma no conjunto de arquivos da release.
///
/// O SUFIXO CARREGA A ARQUITETURA, e nao so a extensao. Uma release de macOS
/// publica DOIS `.dmg` — `aarch64` para Apple Silicon e `x64` para Intel — e
/// casar so por `.dmg` entregaria o pacote da arquitetura errada para metade
/// dos Macs. O aplicativo baixado nem abriria, e o erro apareceria como se a
/// atualizacao estivesse corrompida.
///
/// A ordem dos padroes e a preferencia. No Windows o NSIS vem antes do MSI
/// porque instala sem elevacao.
fn padrao_do_instalador() -> &'static [&'static str] {
    match (
        cfg!(target_os = "windows"),
        cfg!(target_os = "macos"),
        cfg!(target_arch = "aarch64"),
    ) {
        (true, _, true) => &["arm64-setup.exe", "arm64_en-US.msi"],
        (true, _, false) => &["x64-setup.exe", "x64_en-US.msi"],
        (_, true, true) => &["aarch64.dmg"],
        (_, true, false) => &["x64.dmg"],
        // Linux: hoje a release so publica x86_64, mas o sufixo explicito evita
        // que um pacote de outra arquitetura seja servido no dia em que houver.
        (_, _, true) => &["aarch64.AppImage", "arm64.deb", "aarch64.rpm"],
        _ => &["amd64.AppImage", "amd64.deb", "x86_64.rpm"],
    }
}

pub async fn verificar() -> Result<Atualizacao, String> {
    let atual = env!("CARGO_PKG_VERSION").to_string();
    let cliente = reqwest::Client::builder()
        // Curto de propósito: isto roda na abertura do aplicativo, e uma rede
        // ruim não pode segurar a primeira tela.
        .timeout(Duration::from_secs(8))
        .user_agent("postly-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = cliente
        .get(format!(
            "https://api.github.com/repos/{REPO}/releases/latest"
        ))
        .send()
        .await
        .map_err(|e| format!("falha ao consultar atualizacoes: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("o GitHub respondeu {}", resp.status()));
    }

    let corpo: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let tag = corpo
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let versao_nova = tag.trim_start_matches('v').to_string();

    if versao_nova.is_empty() || !mais_nova(&versao_nova, &atual) {
        return Ok(Atualizacao {
            disponivel: false,
            versao_atual: atual,
            versao_nova: None,
            notas: None,
            url_instalador: None,
            tamanho_bytes: None,
        });
    }

    let vazio = Vec::new();
    let ativos = corpo
        .get("assets")
        .and_then(|a| a.as_array())
        .unwrap_or(&vazio);

    // A ordem dos padrões é a preferência: no Windows o NSIS instala sem
    // elevação, então ele vem antes do MSI.
    let escolhido = padrao_do_instalador().iter().find_map(|sufixo| {
        ativos.iter().find(|a| {
            a.get("name")
                .and_then(|n| n.as_str())
                .is_some_and(|n| n.ends_with(sufixo))
        })
    });

    Ok(Atualizacao {
        disponivel: escolhido.is_some(),
        versao_atual: atual,
        versao_nova: Some(versao_nova),
        notas: corpo
            .get("body")
            .and_then(|v| v.as_str())
            .map(|s| s.chars().take(600).collect()),
        url_instalador: escolhido
            .and_then(|a| a.get("browser_download_url"))
            .and_then(|v| v.as_str())
            .map(String::from),
        tamanho_bytes: escolhido
            .and_then(|a| a.get("size"))
            .and_then(|v| v.as_u64()),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressoBaixa {
    pub baixado: u64,
    pub total: u64,
    pub percent: u8,
}

/// Baixa o instalador e o abre.
///
/// O aplicativo não se substitui sozinho: quem troca os arquivos é o
/// instalador de cada sistema, que sabe lidar com arquivo em uso, atalho e
/// registro. O que este código faz é entregá-lo pronto e sair da frente.
pub async fn instalar(app: AppHandle, url: String) -> Result<String, String> {
    use futures_util::StreamExt;

    let cliente = reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .user_agent("postly-updater")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = cliente
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("falha ao baixar a atualizacao: {e}"))?;
    let total = resp.content_length().unwrap_or(0);

    let nome = url.rsplit('/').next().unwrap_or("postly-update");
    let destino = std::env::temp_dir().join(nome);
    let mut arquivo = std::fs::File::create(&destino)
        .map_err(|e| format!("nao consegui gravar em {destino:?}: {e}"))?;

    let mut fluxo = resp.bytes_stream();
    let mut baixado = 0u64;
    let mut ultimo_aviso = 0u8;

    while let Some(pedaco) = fluxo.next().await {
        let pedaco = pedaco.map_err(|e| format!("a conexao caiu no meio: {e}"))?;
        use std::io::Write;
        arquivo.write_all(&pedaco).map_err(|e| e.to_string())?;
        baixado += pedaco.len() as u64;

        // Sem Content-Length o total vem zero, e aí não há percentual a mostrar:
        // `checked_div` devolve None e a barra fica indeterminada em 0.
        let percent = (baixado * 100).checked_div(total).unwrap_or(0) as u8;
        // Um evento por ponto percentual: emitir a cada pedaço encheria a
        // ponte com milhares de mensagens para desenhar a mesma barra.
        if percent != ultimo_aviso {
            ultimo_aviso = percent;
            let _ = app.emit(
                "postly://atualizacao",
                ProgressoBaixa {
                    baixado,
                    total,
                    percent,
                },
            );
        }
    }
    drop(arquivo);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // AppImage e .dmg precisam do bit de execução; para .deb e .rpm ele é
        // inofensivo.
        let _ = std::fs::set_permissions(&destino, std::fs::Permissions::from_mode(0o755));
    }

    let caminho = destino.to_string_lossy().to_string();
    platform::current()
        .open_step(&caminho)
        .run()
        .map_err(|e| format!("baixei em {caminho}, mas nao consegui abrir: {e}"))?;

    Ok(caminho)
}

#[cfg(test)]
mod testes {
    use super::{mais_nova, padrao_do_instalador};

    #[test]
    fn reconhece_versao_mais_nova() {
        assert!(mais_nova("0.2.0", "0.1.0"));
        assert!(mais_nova("1.0.0", "0.9.9"));
        assert!(mais_nova("0.1.1", "0.1.0"));
        assert!(mais_nova("v0.2.0", "0.1.0"), "o v da tag não conta");
    }

    #[test]
    fn nao_oferece_o_que_ja_esta_rodando() {
        assert!(!mais_nova("0.1.0", "0.1.0"));
        assert!(!mais_nova("0.1.0", "0.2.0"));
        assert!(!mais_nova("0.9.0", "1.0.0"));
    }

    /// Os nomes que a release publica de verdade (conferidos na v0.1.1).
    const ARTEFATOS: &[&str] = &[
        "Postly-0.1.1-1.x86_64.rpm",
        "Postly_0.1.1_aarch64.app.tar.gz",
        "Postly_0.1.1_aarch64.dmg",
        "Postly_0.1.1_amd64.AppImage",
        "Postly_0.1.1_amd64.deb",
        "Postly_0.1.1_x64-setup.exe",
        "Postly_0.1.1_x64.app.tar.gz",
        "Postly_0.1.1_x64.dmg",
        "Postly_0.1.1_x64_en-US.msi",
    ];

    #[test]
    fn o_padrao_desta_maquina_acha_exatamente_um_pacote() {
        // A release de macOS publica DOIS `.dmg`, um por arquitetura. Casar so
        // por extensao entregaria o errado para metade dos Macs — e o app
        // baixado nem abriria.
        let achados: Vec<&str> = padrao_do_instalador()
            .iter()
            .filter_map(|sufixo| ARTEFATOS.iter().find(|n| n.ends_with(sufixo)).copied())
            .collect();
        assert!(
            !achados.is_empty(),
            "nenhum pacote serve para esta plataforma; padroes: {:?}",
            padrao_do_instalador()
        );
        // O primeiro e o que o updater escolhe, e ele tem que ser desta
        // arquitetura.
        let escolhido = achados[0];
        if cfg!(target_os = "macos") {
            let esperado = if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "x64"
            };
            assert!(escolhido.contains(esperado), "macOS escolheu {escolhido}");
        }
        // E nunca um `.app.tar.gz`, que e o pacote de atualizacao interna do
        // Tauri e nao um instalador.
        assert!(!escolhido.ends_with(".app.tar.gz"), "escolheu {escolhido}");
    }

    #[test]
    fn nenhum_padrao_casa_com_pacote_de_outra_arquitetura() {
        for sufixo in padrao_do_instalador() {
            let casam: Vec<&&str> = ARTEFATOS.iter().filter(|n| n.ends_with(sufixo)).collect();
            assert!(
                casam.len() <= 1,
                "o sufixo {sufixo} casa com {} pacotes: {casam:?}",
                casam.len()
            );
        }
    }

    #[test]
    fn compara_por_numero_e_nao_por_texto() {
        // O caso que quebra comparação de string: "0.10.0" < "0.9.0" em ordem
        // alfabética, e o app deixaria de oferecer atualização exatamente ao
        // passar da nona versão menor.
        assert!(mais_nova("0.10.0", "0.9.0"));
        assert!(mais_nova("0.2.10", "0.2.9"));
        assert!(mais_nova("1.0.0", "0.10.0"));
    }

    #[test]
    fn lida_com_formatos_incompletos_ou_estranhos() {
        assert!(mais_nova("0.2", "0.1.9"), "faltando o patch");
        assert!(!mais_nova("", "0.1.0"), "tag vazia não é atualização");
        assert!(!mais_nova("nao-e-versao", "0.1.0"));
        // Pré-release nunca substitui a versão final de mesmo número: ninguém
        // deve ser empurrado de uma 0.2.0 estável para uma 0.2.0-rc1.
        assert!(!mais_nova("0.2.0-rc1", "0.2.0"));
        assert!(!mais_nova("0.2.0-1", "0.2.0"));
        // O contrário vale: quem está num pré-release deve receber a final.
        assert!(mais_nova("0.2.0", "0.2.0-rc1"));
    }
}
