//! Comandos de configuracao: preferencias, referencias, skills, provedor e
//! idioma.
//!
//! Separados dos comandos de operacao (diagnostico, campanha, cerebro) porque
//! sao dois momentos: aqui a pessoa decide como o sistema vai rodar, la ela
//! opera. O `commands.rs` estava virando o arquivo onde tudo cai.

use crate::ollama::catalog;
use crate::ollama::client;

// ------------------------------------------------------------- preferencias

#[tauri::command]
pub fn preferencias() -> crate::prefs::Prefs {
    crate::prefs::load()
}

/// Liga ou desliga a escolha manual de modelo por cargo.
#[tauri::command]
pub fn definir_modo_avancado(ligado: bool) -> Result<crate::prefs::Prefs, String> {
    let mut p = crate::prefs::load();
    p.avancado = ligado;
    crate::prefs::save(&p)?;
    Ok(p)
}

/// Fixa (ou solta, com `tag` vazia) o modelo de um cargo.
#[tauri::command]
pub fn definir_modelo_do_cargo(cargo: String, tag: String) -> Result<crate::prefs::Prefs, String> {
    let mut p = crate::prefs::load();
    if tag.trim().is_empty() {
        p.modelos.remove(&cargo);
    } else {
        if !catalog::CATALOG.iter().any(|m| m.tag == tag) {
            return Err(format!("modelo desconhecido: {tag}"));
        }
        p.modelos.insert(cargo, tag);
    }
    crate::prefs::save(&p)?;
    Ok(p)
}

/// Apaga um modelo baixado, devolvendo o espaco em disco.
#[tauri::command]
pub async fn remover_modelo(tag: String) -> Result<(), String> {
    client::delete_model(&tag).await
}

// ------------------------------------------------------- referencias e marca

#[tauri::command]
pub fn salvar_referencia(
    nome: String,
    dados: String,
    tipo: crate::referencias::TipoReferencia,
    nota: String,
) -> Result<crate::prefs::Prefs, String> {
    let r = crate::referencias::salvar(&nome, &dados, tipo, &nota)?;
    let mut p = crate::prefs::load();
    p.referencias.push(r);
    crate::prefs::save(&p)?;
    Ok(p)
}

#[tauri::command]
pub fn remover_referencia(id: String) -> Result<crate::prefs::Prefs, String> {
    let mut p = crate::prefs::load();
    if let Some(pos) = p.referencias.iter().position(|r| r.id == id) {
        let r = p.referencias.remove(pos);
        crate::referencias::remover(&r.caminho)?;
    }
    crate::prefs::save(&p)?;
    Ok(p)
}

/// A nota diz ao modelo o que olhar na imagem. Sem comando proprio, editar a
/// nota exigiria reenviar a imagem inteira.
#[tauri::command]
pub fn anotar_referencia(id: String, nota: String) -> Result<crate::prefs::Prefs, String> {
    let mut p = crate::prefs::load();
    if let Some(r) = p.referencias.iter_mut().find(|r| r.id == id) {
        r.nota = nota;
    }
    crate::prefs::save(&p)?;
    Ok(p)
}

#[tauri::command]
pub fn salvar_design_system(
    ds: crate::referencias::DesignSystem,
) -> Result<crate::prefs::Prefs, String> {
    let mut p = crate::prefs::load();
    p.ds = ds;
    crate::prefs::save(&p)?;
    Ok(p)
}

/// O frontend avisa qual idioma esta na tela; as mensagens de erro do backend
/// passam a sair na mesma lingua.
#[tauri::command]
pub fn definir_idioma(idioma: String) {
    crate::idioma::definir(&idioma);
}

// ----------------------------------------------------------------- provedor

#[derive(serde::Serialize)]
pub struct StatusProvedor {
    pub provedor: crate::prefs::Provedor,
    /// O Claude Code esta instalado nesta maquina?
    pub claude_disponivel: bool,
    pub claude_versao: Option<String>,
    /// Caminho do binario que sera executado. E a prova, na tela, de que o
    /// turno roda por um processo local e nao por uma chamada de API.
    pub claude_caminho: Option<String>,
    /// Nome da variavel de credencial encontrada no ambiente, se houver.
    /// Ela e removida do processo filho; o aviso existe para a pessoa saber
    /// que o turno NAO vai usar a chave que ela talvez esperasse usar.
    pub credencial_ignorada: Option<String>,
}

/// Um cargo e o modelo Claude que o assume.
#[derive(Debug, Clone, serde::Serialize)]
pub struct VagaClaude {
    pub cargo: String,
    pub nivel: crate::orchestrator::roles::Tier,
    pub modelo: String,
    /// Nome curto para a tela: "Opus 5", nao "claude-opus-5".
    pub rotulo: String,
    pub porque: String,
}

/// O elenco quando quem executa e o Claude Code.
///
/// Existe pelo mesmo motivo do elenco do Ollama: a pessoa precisa ver quem
/// assume cada cargo antes de rodar. O eixo muda (nao ha memoria a medir, nao
/// ha download a fazer), mas a pergunta e a mesma.
#[tauri::command]
pub fn elenco_claude() -> Vec<VagaClaude> {
    use crate::orchestrator::roles::Role;
    [
        Role::DiretorGeral,
        Role::GerenteSetor,
        Role::MotionDesigner,
        Role::Criador,
        Role::Auditor,
    ]
    .iter()
    .map(|r| {
        let nivel = r.tier();
        let modelo = crate::claude::modelo_do_nivel(nivel).to_string();
        VagaClaude {
            cargo: r.slug().to_string(),
            nivel,
            rotulo: crate::claude::rotulo_do_modelo(&modelo).to_string(),
            modelo,
            porque: crate::idioma::msg(r.porque_este_nivel().0, r.porque_este_nivel().1),
        }
    })
    .collect()
}

#[tauri::command]
pub async fn status_provedor() -> StatusProvedor {
    let disponivel = crate::claude::disponivel();
    StatusProvedor {
        provedor: crate::prefs::load().provedor,
        claude_disponivel: disponivel,
        claude_versao: if disponivel { crate::claude::versao().await } else { None },
        claude_caminho: crate::platform::current()
            .which("claude")
            .map(|p| p.display().to_string()),
        credencial_ignorada: crate::claude::credencial_externa_no_ambiente(),
    }
}

#[tauri::command]
pub fn definir_provedor(provedor: crate::prefs::Provedor) -> Result<crate::prefs::Prefs, String> {
    if provedor == crate::prefs::Provedor::ClaudeCode && !crate::claude::disponivel() {
        return Err(crate::idioma::msg(
            "Claude Code nao encontrado no PATH.",
            "Claude Code was not found on PATH.",
        ));
    }
    let mut p = crate::prefs::load();
    p.provedor = provedor;
    crate::prefs::save(&p)?;
    Ok(p)
}

// ------------------------------------------------------------------- skills

#[tauri::command]
pub fn salvar_skill(
    id: String,
    nome: String,
    texto: String,
    cargo: String,
    ativa: bool,
) -> Result<crate::prefs::Prefs, String> {
    if nome.trim().is_empty() {
        return Err(crate::idioma::msg("De um nome a skill.", "Give the skill a name."));
    }
    let mut p = crate::prefs::load();
    match p.skills.iter_mut().find(|s| s.id == id) {
        Some(s) => {
            s.nome = nome;
            s.texto = texto;
            s.cargo = cargo;
            s.ativa = ativa;
        }
        None => p.skills.push(crate::prefs::Skill {
            id: format!("{}", chrono::Utc::now().timestamp_millis()),
            nome,
            texto,
            cargo,
            // Nasce desligada de proposito: uma instrucao nova nao deve mudar a
            // proxima campanha sem a pessoa dizer que quer.
            ativa,
        }),
    }
    crate::prefs::save(&p)?;
    Ok(p)
}

#[tauri::command]
pub fn remover_skill(id: String) -> Result<crate::prefs::Prefs, String> {
    let mut p = crate::prefs::load();
    p.skills.retain(|s| s.id != id);
    crate::prefs::save(&p)?;
    Ok(p)
}

/// Como ficaria o system do cargo com as skills ativas, para a pessoa ver o
/// que esta realmente mandando antes de rodar uma campanha inteira.
#[tauri::command]
pub fn previa_de_skills(cargo: String) -> String {
    crate::prefs::load().bloco_de_skills(&cargo)
}
