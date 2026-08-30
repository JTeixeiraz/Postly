//! Os dados que entram e saem de uma campanha.
//!
//! Separados do fluxo porque sao contrato, nao processo: a tela do app depende
//! da forma exata deles, e misturar isso ao pipeline fazia toda leitura do
//! fluxo comecar por sessenta linhas de campo.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::roles::Network;
use crate::gemini::{GeneratedImage, ImageQuality};
use crate::vault;

#[derive(Debug, Clone, Deserialize)]
pub struct CampaignRequest {
    pub objetivo: String,
    pub redes: Vec<Network>,
    /// slug da rede -> credenciais. Vazio quando ja existe sessao no perfil.
    #[serde(default)]
    pub credenciais: BTreeMap<String, vault::Credential>,
    #[serde(default)]
    pub salvar_credenciais: bool,
    #[serde(default = "default_quality")]
    pub qualidade_imagem: ImageQuality,
    /// Monta a publicacao ate o ultimo clique, sem enviar.
    #[serde(default)]
    pub simular: bool,
    #[serde(default = "default_rounds")]
    pub max_rodadas: u8,
    /// Liga o raciocinio explicito nos cargos de decisao. Melhora a estrategia
    /// e multiplica o tempo de cada turno em maquina sem GPU.
    #[serde(default)]
    pub pensamento_estendido: bool,
    /// Idioma em que os agentes escrevem a entrega. O andaime dos prompts fica
    /// em ingles, que e onde os modelos sao mais fortes; o que muda e a lingua
    /// do que a pessoa vai ler e publicar.
    #[serde(default = "default_idioma")]
    pub idioma: String,
}

fn default_quality() -> ImageQuality {
    ImageQuality::Rapida
}
fn default_rounds() -> u8 {
    2
}
fn default_idioma() -> String {
    "pt".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PecaFinal {
    pub rede: String,
    pub conceito: String,
    pub prompt_imagem: String,
    pub legenda: String,
    pub hashtags: Vec<String>,
    pub chamada_para_acao: String,
    pub imagem: Option<GeneratedImage>,
    pub publicado: bool,
    pub detalhe_publicacao: String,
    pub screenshot: Option<String>,
    /// O gerente marcou esta peca como candidata a movimento.
    #[serde(default)]
    pub motion_pedido: bool,
    /// Roteiro de animacao, quando a pessoa autorizou o turno.
    #[serde(default)]
    pub roteiro_motion: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignReport {
    pub run_id: String,
    pub run_dir: String,
    pub index_path: String,
    pub pecas: Vec<PecaFinal>,
    pub rodadas: u8,
    pub aprovado: bool,
    pub parecer_auditor: String,
    pub avisos: Vec<String>,
}
