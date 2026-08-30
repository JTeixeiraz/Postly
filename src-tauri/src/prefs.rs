//! Preferencias da pessoa que usa o app.
//!
//! Fica separado do cofre de proposito: o cofre e cifrado porque guarda
//! segredo, e misturar preferencia com credencial faria toda leitura de
//! configuracao passar por decifragem sem motivo. Aqui e JSON simples.
//!
//! A regra do produto continua valendo: por padrao o sistema escolhe o modelo
//! sozinho a cada troca de cargo, medindo a memoria do momento. A escolha
//! manual e uma saida para quem sabe o que quer, nao o caminho normal.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::platform;

/// Instrucao extra que a pessoa acrescenta ao system prompt de um cargo.
///
/// E a valvula de escape do produto: quem tem um metodo proprio de copy, um
/// guia de marca ou um jeito de estruturar briefing pode ensinar isso ao cargo
/// sem tocar no codigo. Tambem e a forma mais rapida de estragar o resultado,
/// por isso a interface avisa e a skill nasce desligada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub nome: String,
    /// O texto que entra no prompt, literal.
    pub texto: String,
    /// slug do cargo, ou vazio para todos os cargos.
    #[serde(default)]
    pub cargo: String,
    #[serde(default)]
    pub ativa: bool,
}

/// Quem executa os turnos.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provedor {
    /// Modelos locais. Padrao: nada sai da maquina.
    #[default]
    Ollama,
    /// O Claude Code que a pessoa ja tem instalado. Muito mais rapido, custa
    /// dinheiro por turno e manda o prompt para fora.
    ClaudeCode,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Prefs {
    /// Quem executa os turnos. Ollama por padrao.
    #[serde(default)]
    pub provedor: Provedor,
    /// Quem gera a arte das pecas.
    #[serde(default)]
    pub provedor_imagem: crate::imagem::ProvedorImagem,
    /// Liga a escolha manual de modelo por cargo.
    #[serde(default)]
    pub avancado: bool,
    /// slug do cargo -> tag do modelo. So vale com `avancado` ligado.
    #[serde(default)]
    pub modelos: BTreeMap<String, String>,
    /// Identidade visual que o criador precisa respeitar.
    #[serde(default)]
    pub ds: crate::referencias::DesignSystem,
    /// Material de apoio: o proprio da marca e o de estilo.
    #[serde(default)]
    pub referencias: Vec<crate::referencias::Referencia>,
    /// Instrucoes extras que a pessoa escreveu para os cargos.
    #[serde(default)]
    pub skills: Vec<Skill>,
}

fn caminho() -> PathBuf {
    platform::current().data_dir().join("prefs.json")
}

pub fn load() -> Prefs {
    std::fs::read_to_string(caminho())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save(prefs: &Prefs) -> Result<(), String> {
    let caminho = caminho();
    if let Some(pai) = caminho.parent() {
        std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
    }
    let texto = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    std::fs::write(&caminho, texto).map_err(|e| format!("falha ao gravar preferencias: {e}"))
}

impl Prefs {
    /// Modelo escolhido a mao para um cargo, se a escolha manual estiver ligada
    /// e a tag ainda existir no catalogo.
    pub fn modelo_de(&self, cargo: &str) -> Option<&'static crate::ollama::catalog::ModelSpec> {
        if !self.avancado {
            return None;
        }
        let tag = self.modelos.get(cargo)?;
        crate::ollama::catalog::CATALOG
            .iter()
            .find(|m| m.tag == *tag)
    }
}

impl Prefs {
    /// Bloco de skills ativas para um cargo, pronto para o system prompt.
    ///
    /// Entra DEPOIS da doutrina e do organograma: instrucao da pessoa refina o
    /// que o sistema ja estabeleceu, e vem por ultimo justamente para poder
    /// contradizer um detalhe sem apagar a base.
    pub fn bloco_de_skills(&self, cargo: &str) -> String {
        let minhas: Vec<&Skill> = self
            .skills
            .iter()
            .filter(|s| s.ativa && !s.texto.trim().is_empty())
            .filter(|s| s.cargo.is_empty() || s.cargo == cargo)
            .collect();

        if minhas.is_empty() {
            return String::new();
        }

        let mut out = vec![
            "INSTRUCOES ADICIONAIS DE QUEM OPERA ESTE SISTEMA. Elas refinam o que \
             foi dito acima e nao substituem as regras de entrega nem o organograma."
                .to_string(),
        ];
        for s in minhas {
            out.push(format!("\n[{}]\n{}", s.nome.trim(), s.texto.trim()));
        }
        out.join("\n")
    }
}
