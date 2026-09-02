//! Achar o `agy` desta maquina e preparar a pasta de onde ele roda.
//!
//! Separado do turno porque e outro momento: isto acontece uma vez, na tela de
//! configuracao, e responde "da para usar?". O turno acontece muitas vezes,
//! dentro da campanha, e responde "o que o cargo disse?".

use std::path::PathBuf;

/// Variaveis que desviam o turno para outra conta de cobranca.
///
/// AQUI ELAS SO AVISAM, e a diferenca em relacao ao Claude Code e medida, nao
/// esquecimento. O Claude Code remove `ANTHROPIC_API_KEY` do processo filho
/// porque la a variavel VENCE a assinatura e a pessoa passaria a pagar por
/// token sem saber.
///
/// No Antigravity CLI o metodo de autenticacao e escolhido em
/// `~/.gemini/settings.json` e VENCE a variavel de ambiente — probe nesta
/// maquina com `GEMINI_API_KEY` invalida: o CLI a ignorou e seguiu pelo OAuth
/// que estava salvo. E quando nao ha metodo escolhido, a variavel e a UNICA
/// autenticacao que a pessoa tem; remove-la quebraria quem funciona hoje para
/// proteger de um risco que a medicao mostrou nao existir.
const CREDENCIAIS_DE_FORA: &[&str] = &[
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "GOOGLE_GENAI_USE_VERTEXAI",
    "GOOGLE_GENAI_USE_GCA",
    "GOOGLE_CLOUD_PROJECT",
];

pub fn credencial_externa_no_ambiente() -> Option<String> {
    CREDENCIAIS_DE_FORA
        .iter()
        .find(|v| std::env::var_os(v).is_some_and(|x| !x.is_empty()))
        .map(|v| v.to_string())
}

/// Onde esta o `agy` desta maquina.
///
/// A busca tem os mesmos tres degraus do Claude Code, e pela mesma razao
/// medida la: um app aberto pelo icone do menu recebe o PATH da sessao
/// grafica, que nao le `.zshrc` nem `.profile`.
///
/// O caso do Gemini e ainda mais agudo, porque ele chega por npm. Nesta
/// maquina ele mora em `~/.nvm/versions/node/v20.20.0/bin/gemini` — um caminho
/// que so entra no PATH depois que o `nvm.sh` roda no perfil do shell. Um app
/// aberto pelo icone nunca veria.
///
/// O sucesso e memorizado e a falha nao: um binario que apareceu nao some, e a
/// pessoa pode instalar o Antigravity CLI com o app aberto.
static ENCONTRADO: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

pub fn localizar() -> Option<PathBuf> {
    if let Some(p) = ENCONTRADO.get() {
        return Some(p.clone());
    }
    let achado = no_path()
        .or_else(perguntar_ao_shell)
        .or_else(nos_lugares_conhecidos)?;
    let _ = ENCONTRADO.set(achado.clone());
    Some(achado)
}

fn no_path() -> Option<PathBuf> {
    crate::platform::current().which("agy")
}

/// `-l` e o que importa: faz o shell ler o perfil, que e onde o `nvm` monta o
/// PATH. Sem ele o shell herda o mesmo ambiente pobre do processo e a pergunta
/// nao acrescenta nada.
fn perguntar_ao_shell() -> Option<PathBuf> {
    if crate::platform::current().id() == crate::platform::Platform::Windows {
        return None;
    }
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let saida = std::process::Command::new(&shell)
        .args(["-lc", "command -v gemini"])
        .output()
        .ok()?;
    if !saida.status.success() {
        return None;
    }
    let p = PathBuf::from(String::from_utf8_lossy(&saida.stdout).trim().to_string());
    p.is_file().then_some(p)
}

/// Onde o npm costuma deixar o binario.
///
/// O `nvm` merece varredura propria: ele instala por versao de Node e a pasta
/// carrega o numero da versao no nome, entao nao ha caminho fixo para escrever.
fn nos_lugares_conhecidos() -> Option<PathBuf> {
    let casa = dirs::home_dir()?;
    if crate::platform::current().id() == crate::platform::Platform::Windows {
        return [
            casa.join("AppData/Roaming/npm/agy.exe"),
            casa.join("AppData/Roaming/npm/agy.cmd"),
        ]
        .into_iter()
        .find(|p| p.is_file());
    }

    let fixos = [
        casa.join(".npm-global/bin/gemini"),
        casa.join(".local/bin/gemini"),
        casa.join(".bun/bin/gemini"),
        PathBuf::from("/usr/local/bin/gemini"),
        PathBuf::from("/opt/homebrew/bin/gemini"),
    ];
    if let Some(p) = fixos.into_iter().find(|p| p.is_file()) {
        return Some(p);
    }

    let mut versoes: Vec<PathBuf> = std::fs::read_dir(casa.join(".nvm/versions/node"))
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
    // Da versao mais nova para a mais velha: quem tem duas instalacoes quer a
    // recente, e escolher pela ordem do diretorio seria escolher por acaso.
    versoes.sort();
    versoes
        .into_iter()
        .rev()
        .map(|v| v.join("bin/gemini"))
        .find(|p| p.is_file())
}

pub fn disponivel() -> bool {
    localizar().is_some()
}

pub async fn versao() -> Option<String> {
    let saida = tokio::process::Command::new(localizar()?)
        .arg("--version")
        .output()
        .await
        .ok()?;
    saida
        .status
        .success()
        .then(|| String::from_utf8_lossy(&saida.stdout).trim().to_string())
}

/// A pasta de onde o `agy` roda, com as configuracoes que o turno exige.
///
/// Nao e organizacao: o Antigravity CLI le `<cwd>/.gemini/settings.json` como
/// configuracao de PROJETO, que vence a do usuario. E o unico jeito de
/// desligar as ferramentas sem escrever no `~/.gemini` de quem usa — mexer na
/// configuracao pessoal de alguem para o nosso turno rodar seria abuso.
///
/// `tools.core: []` e o equivalente do `--disallowed-tools` do Claude Code, e
/// existe pelos mesmos dois motivos, na mesma ordem de importancia:
///   1. um agente de marketing nao tem por que mexer no disco de ninguem;
///   2. de quebra, encolhe o prompt que o CLI monta a cada chamada.
pub fn pasta_de_trabalho() -> Result<PathBuf, String> {
    let dir = crate::platform::current().data_dir().join("antigravity");
    let conf = dir.join(".gemini");
    std::fs::create_dir_all(&conf).map_err(|e| e.to_string())?;

    let settings = serde_json::json!({
        "tools": { "core": [] },
        // Sem isto o CLI mandaria estatistica de uso a cada turno, e o produto
        // promete que nada sai da maquina alem da inferencia pedida.
        "privacy": { "usageStatisticsEnabled": false },
        // Um autoupdate no meio da campanha trocaria o binario entre dois
        // turnos do mesmo pipeline.
        "general": { "enableAutoUpdate": false, "enableAutoUpdateNotification": false }
    });
    std::fs::write(
        conf.join("settings.json"),
        serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("falha ao preparar a pasta do Antigravity CLI: {e}"))?;
    Ok(dir)
}
