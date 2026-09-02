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

    /// O Antigravity CLI esta instalado nesta maquina?
    pub agy_disponivel: bool,
    pub agy_versao: Option<String>,
    pub agy_caminho: Option<String>,
    /// Variavel de credencial do Google encontrada no ambiente.
    ///
    /// Ao contrario da do Claude Code, esta NAO e removida do processo filho:
    /// quando existe, ela e a autenticacao de quem usa, nao um desvio. O aviso
    /// serve para a pessoa saber por qual conta o turno vai.
    pub agy_credencial_no_ambiente: Option<String>,
}

/// Um cargo e o modelo de um provedor externo que o assume.
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

/// O elenco quando quem executa e o Antigravity CLI.
///
/// Comando proprio, e nao um parametro no `elenco_claude`: os dois provedores
/// vivem em modulos separados e nada garante que o organograma deles siga
/// igual para sempre. Um comando por provedor deixa a divergencia possivel sem
/// obrigar a inventar um enum de despacho hoje.
#[tauri::command]
pub fn elenco_antigravity() -> Vec<VagaClaude> {
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
        let modelo = crate::antigravity::modelo_do_nivel(nivel).to_string();
        VagaClaude {
            cargo: r.slug().to_string(),
            nivel,
            rotulo: crate::antigravity::rotulo_do_modelo(&modelo).to_string(),
            modelo,
            porque: crate::idioma::msg(r.porque_este_nivel().0, r.porque_este_nivel().1),
        }
    })
    .collect()
}

#[tauri::command]
pub async fn status_provedor() -> StatusProvedor {
    let claude = crate::claude::disponivel();
    let agy = crate::antigravity::disponivel();
    StatusProvedor {
        provedor: crate::prefs::load().provedor,
        claude_disponivel: claude,
        claude_versao: if claude {
            crate::claude::versao().await
        } else {
            None
        },
        // A busca completa, e nao so o PATH: um app aberto pelo icone do menu
        // nao tem `~/.local/bin` no PATH, e a tela diria "nao encontrado" ao
        // lado de um provedor que acabou de rodar um turno.
        claude_caminho: crate::claude::localizar().map(|p| p.display().to_string()),
        credencial_ignorada: crate::claude::credencial_externa_no_ambiente(),

        agy_disponivel: agy,
        agy_versao: if agy {
            crate::antigravity::versao().await
        } else {
            None
        },
        agy_caminho: crate::antigravity::localizar().map(|p| p.display().to_string()),
        agy_credencial_no_ambiente: crate::antigravity::credencial_externa_no_ambiente(),
    }
}

/// O que cada modo faz nesta maquina, agora.
///
/// Os numeros sao calculados com a memoria livre do momento, e nao com uma
/// tabela fixa: "usa mais RAM" nao diz nada a quem tem 8 GB e a quem tem 64.
#[derive(serde::Serialize)]
pub struct CartaoModo {
    pub slug: String,
    /// Teto de memoria por modelo neste modo.
    pub teto_bytes: u64,
    /// O que a escolha automatica traria para o cargo que decide.
    pub modelo_alto: String,
    /// E para o que executa.
    pub modelo_baixo: String,
    /// Velocidade estimada do cargo que decide, nesta maquina.
    pub tps_alto: f32,
    pub ativo: bool,
}

#[tauri::command]
pub async fn modos_de_desempenho() -> Vec<CartaoModo> {
    use crate::prefs::ModoDesempenho as M;
    let perfil = crate::hardware::compute_profile();
    let instalados = crate::ollama::client::installed_models().await;
    let prefs = crate::prefs::load();
    let externo = prefs.provedor.externo();

    [M::Economico, M::Normal, M::Maximo]
        .into_iter()
        .map(|m| {
            let teto = crate::hardware::snapshot_com(m).live_budget_bytes;
            let nome = |tier| -> (String, f32) {
                // Nos provedores de fora a velocidade nao e medida em tokens
                // por segundo desta maquina: a inferencia acontece longe dela.
                // O zero e o que faz a tela mostrar so o modelo, sem inventar
                // uma vazao que nao mediu.
                if externo {
                    let id = match prefs.provedor {
                        crate::prefs::Provedor::Antigravity => {
                            crate::antigravity::rotulo_do_modelo(
                                crate::antigravity::modelo_do_nivel_com(tier, m),
                            )
                        }
                        _ => crate::claude::rotulo_do_modelo(crate::claude::modelo_do_nivel_com(
                            tier, m,
                        )),
                    };
                    return (id.to_string(), 0.0);
                }
                crate::ollama::catalog::pick(tier, teto, perfil.mode, m, false, &instalados)
                    .map(|(s, _)| {
                        let tps = crate::hardware::accelerator::estimated_tokens_per_second(
                            perfil.mode,
                            s.active_params_b,
                        );
                        (s.label.to_string(), tps)
                    })
                    .unwrap_or_else(|| ("-".into(), 0.0))
            };
            let (alto, tps) = nome(crate::orchestrator::roles::Tier::Alto);
            let (baixo, _) = nome(crate::orchestrator::roles::Tier::Baixo);
            CartaoModo {
                slug: m.slug().to_string(),
                teto_bytes: teto,
                modelo_alto: alto,
                modelo_baixo: baixo,
                tps_alto: tps,
                ativo: m == prefs.modo,
            }
        })
        .collect()
}

#[tauri::command]
pub fn definir_modo(slug: String) -> Result<(), String> {
    use crate::prefs::ModoDesempenho as M;
    let modo = match slug.as_str() {
        "economico" => M::Economico,
        "normal" => M::Normal,
        "maximo" => M::Maximo,
        outro => return Err(format!("modo desconhecido: {outro}")),
    };
    let mut p = crate::prefs::load();
    p.modo = modo;
    crate::prefs::save(&p)?;
    Ok(())
}

/// O estado do gerador local: o motor, os modelos e o que ja esta no disco.
#[derive(serde::Serialize)]
pub struct EstadoLocal {
    /// Caminho do executavel, quando ja foi baixado.
    pub motor: Option<String>,
    pub modelos: Vec<CartaoModeloLocal>,
    /// Quanto o conjunto baixado ocupa.
    pub bytes_em_disco: u64,
}

#[derive(serde::Serialize)]
pub struct CartaoModeloLocal {
    pub id: String,
    pub nome: String,
    pub bytes: u64,
    pub passos: u32,
    pub base: u32,
    pub nota: String,
    pub baixado: bool,
}

#[tauri::command]
pub fn estado_imagem_local() -> EstadoLocal {
    use crate::imagem::{catalogo_local, local};
    let baixados = local::modelos_baixados();
    let modelos = catalogo_local::MODELOS
        .iter()
        .map(|m| CartaoModeloLocal {
            id: m.id.to_string(),
            nome: m.nome.to_string(),
            bytes: m.bytes,
            passos: m.passos,
            base: m.base,
            nota: crate::idioma::msg(m.nota_pt, m.nota_en),
            baixado: baixados.iter().any(|b| b == m.arquivo),
        })
        .collect();
    let bytes = baixados
        .iter()
        .filter_map(|b| std::fs::metadata(local::modelos_dir().join(b)).ok())
        .map(|m| m.len())
        .sum();
    EstadoLocal {
        motor: local::binario().map(|p| p.to_string_lossy().to_string()),
        modelos,
        bytes_em_disco: bytes,
    }
}

#[tauri::command]
pub async fn baixar_motor_local(app: tauri::AppHandle) -> Result<String, String> {
    crate::imagem::local::baixar_motor(app).await
}

#[tauri::command]
pub async fn baixar_modelo_local(app: tauri::AppHandle, id: String) -> Result<String, String> {
    crate::imagem::local::baixar_modelo(app, id).await
}

/// Apaga um modelo baixado.
///
/// Existe porque os arquivos passam de 2 GB: sem isto, experimentar tres
/// modelos custaria 10 GB permanentes e a unica saida seria apagar a mao,
/// num diretorio que a pessoa nao sabe onde fica.
#[tauri::command]
pub fn remover_modelo_local(id: String) -> Result<(), String> {
    let spec = crate::imagem::catalogo_local::por_id(&id)
        .ok_or_else(|| format!("modelo desconhecido: {id}"))?;
    let caminho = crate::imagem::local::modelos_dir().join(spec.arquivo);
    if caminho.is_file() {
        std::fs::remove_file(&caminho).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ------------------------------------------------------------------ galeria

#[tauri::command]
pub fn galeria_listar() -> Vec<crate::galeria::Pasta> {
    crate::galeria::listar()
}

#[tauri::command]
pub fn galeria_criar(nome: String) -> Result<crate::galeria::Pasta, String> {
    crate::galeria::criar(&nome)
}

#[derive(serde::Deserialize)]
pub struct ArquivoEnviado {
    pub nome: String,
    pub dados: String,
}

#[tauri::command]
pub fn galeria_adicionar(
    slug: String,
    arquivos: Vec<ArquivoEnviado>,
    para_referencias: bool,
) -> Result<crate::galeria::Pasta, String> {
    // Erro num arquivo nao derruba os outros: quem arrastou dez fotos e tinha
    // um PDF no meio quer as nove que servem, com um aviso sobre a decima.
    let mut falhas = Vec::new();
    for a in &arquivos {
        if let Err(e) = crate::galeria::adicionar(&slug, &a.nome, &a.dados, para_referencias) {
            falhas.push(format!("{}: {e}", a.nome));
        }
    }
    let pasta = crate::galeria::ler(&slug)
        .ok_or_else(|| crate::idioma::msg("Pasta nao encontrada.", "Folder not found."))?;
    if !arquivos.is_empty() && falhas.len() == arquivos.len() {
        return Err(falhas.join(" · "));
    }
    Ok(pasta)
}

#[tauri::command]
pub fn galeria_remover_item(caminho: String) -> Result<(), String> {
    crate::galeria::remover_item(&caminho)
}

#[tauri::command]
pub fn galeria_remover_pasta(slug: String) -> Result<(), String> {
    crate::galeria::remover_pasta(&slug)
}

#[tauri::command]
pub fn definir_provedor(provedor: crate::prefs::Provedor) -> Result<crate::prefs::Prefs, String> {
    if provedor == crate::prefs::Provedor::ClaudeCode && !crate::claude::disponivel() {
        return Err(crate::idioma::msg(
            "Claude Code nao encontrado nesta maquina. Instale em claude.com/code.",
            "Claude Code was not found on this machine. Install it from claude.com/code.",
        ));
    }
    if provedor == crate::prefs::Provedor::Antigravity && !crate::antigravity::disponivel() {
        return Err(crate::idioma::msg(
            "Antigravity CLI nao encontrado nesta maquina. Instale em antigravity.google.",
            "Antigravity CLI was not found on this machine. Install it from antigravity.google.",
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
        return Err(crate::idioma::msg(
            "De um nome a skill.",
            "Give the skill a name.",
        ));
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

// ------------------------------------------------- provedor de imagem

/// Um serviço de geração de arte, do jeito que a tela precisa desenhar.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CartaoImagem {
    pub slug: String,
    pub label: String,
    /// Já rodou contra a API real? Só o Gemini rodou.
    pub verificado: bool,
    /// Autentica com um par id:segredo em vez de uma chave só.
    pub precisa_de_par: bool,
    pub url_da_chave: String,
    pub tem_chave: bool,
    /// Fim da chave guardada, para a pessoa reconhecer qual colou.
    pub dica: String,
    pub ativo: bool,
}

#[tauri::command]
pub fn provedores_de_imagem() -> Vec<CartaoImagem> {
    use crate::imagem::ProvedorImagem;
    let prefs = crate::prefs::load();
    let cofre = crate::vault::load();

    ProvedorImagem::todos()
        .iter()
        .map(|p| {
            let chave = cofre.chave_de(*p);
            CartaoImagem {
                slug: p.slug().to_string(),
                label: p.label().to_string(),
                verificado: p.verificado(),
                precisa_de_par: p.precisa_de_par(),
                url_da_chave: p.url_da_chave().to_string(),
                tem_chave: !chave.trim().is_empty(),
                // Só o fim: o começo de uma chave é o que identifica a conta.
                dica: if chave.len() > 6 {
                    format!("…{}", &chave[chave.len() - 4..])
                } else {
                    String::new()
                },
                ativo: prefs.provedor_imagem == *p,
            }
        })
        .collect()
}

fn provedor_por_slug(slug: &str) -> Result<crate::imagem::ProvedorImagem, String> {
    crate::imagem::ProvedorImagem::todos()
        .into_iter()
        .find(|p| p.slug() == slug)
        .ok_or_else(|| format!("provedor de imagem desconhecido: {slug}"))
}

#[tauri::command]
pub fn definir_provedor_imagem(slug: String) -> Result<Vec<CartaoImagem>, String> {
    let p = provedor_por_slug(&slug)?;
    let mut prefs = crate::prefs::load();
    prefs.provedor_imagem = p;
    crate::prefs::save(&prefs)?;
    Ok(provedores_de_imagem())
}

#[tauri::command]
pub fn salvar_chave_de_imagem(slug: String, chave: String) -> Result<Vec<CartaoImagem>, String> {
    let p = provedor_por_slug(&slug)?;
    let mut cofre = crate::vault::load();
    cofre.definir_chave(p, &chave);
    crate::vault::save(&cofre)?;
    Ok(provedores_de_imagem())
}

/// Testa a chave contra o serviço, sem gerar arte.
#[tauri::command]
pub async fn testar_provedor_imagem(slug: String) -> Result<String, String> {
    let p = provedor_por_slug(&slug)?;
    let chave = crate::vault::load().chave_de(p);
    if chave.trim().is_empty() {
        return Err(crate::idioma::msg(
            "Cole a chave antes de testar.",
            "Paste the key before testing.",
        ));
    }
    crate::imagem::validar(p, &chave).await
}
