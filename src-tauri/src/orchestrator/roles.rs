//! Cargos da postly e o organograma que rege quem fala com quem.
//!
//! A regra estrutural: o nivel do modelo e proporcional ao que o cargo entrega.
//! Quem decide precisa de raciocinio; quem executa precisa de obediencia.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Decisao, estrategia, julgamento. Modelo mais potente que couber.
    Alto,
    /// Critica e verificacao. Intermediario.
    Medio,
    /// Execucao de tarefa ja especificada. Pode ser fraco.
    Baixo,
}

impl Tier {
    /// Ordem de rebaixamento quando falta memoria para o nivel ideal.
    pub fn degradation_path(&self) -> &'static [Tier] {
        match self {
            Tier::Alto => &[Tier::Alto, Tier::Medio, Tier::Baixo],
            Tier::Medio => &[Tier::Medio, Tier::Baixo],
            Tier::Baixo => &[Tier::Baixo],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// So existe quando ha mais de uma rede social na campanha.
    DiretorGeral,
    /// Um por rede social. Decisao criativa, analise de mercado e concorrencia.
    GerenteSetor,
    /// Unico para toda a campanha. Executa o que o gerente mandou.
    Criador,
    /// Confere alucinacao e aderencia ao briefing, junto com o gerente.
    Auditor,
    /// Opcional. So entra quando o gerente (ou o diretor) julga que a peca
    /// pede movimento. Nao decide se havera animacao: quem decide e quem manda
    /// nele, e a pessoa confirma antes do turno rodar.
    MotionDesigner,
}

impl Role {
    pub fn tier(&self) -> Tier {
        match self {
            // Decide a estrategia macro entre varias redes: o mais forte de todos.
            Role::DiretorGeral => Tier::Alto,
            // Analisa mercado e decide a linha criativa: tambem precisa raciocinar.
            Role::GerenteSetor => Tier::Alto,
            // Recebe briefing pronto e produz. Nao decide nada sozinho.
            Role::Criador => Tier::Baixo,
            // Critica com rigor, mas dentro de criterios que ja recebeu.
            Role::Auditor => Tier::Medio,
            // Recebe a peca pronta e o motivo do movimento, e traduz isso em
            // um roteiro de animacao. Nao e execucao cega (precisa escolher o
            // que se move e por que), nem decisao de campanha.
            Role::MotionDesigner => Tier::Medio,
        }
    }

    /// Quanta variacao este cargo pode se dar.
    ///
    /// E caracteristica do cargo, como o nivel: estrategia e criacao pedem
    /// variacao, auditoria pede rigor, e movimento pede invencao dentro de uma
    /// peca ja fechada — menos solto que o criador, mais que o auditor.
    pub fn temperatura(&self) -> f32 {
        match self {
            Role::DiretorGeral | Role::GerenteSetor => 0.8,
            Role::Criador => 0.9,
            Role::MotionDesigner => 0.7,
            Role::Auditor => 0.2,
        }
    }

    /// Por que este cargo tem este nivel. Aparece na tela do elenco, onde a
    /// pergunta "por que o Haiku no criador?" e a primeira que surge.
    pub fn porque_este_nivel(&self) -> (&'static str, &'static str) {
        match self {
            Role::DiretorGeral => (
                "Decide a estrategia entre varias redes. E o julgamento mais caro da campanha.",
                "Decides strategy across networks. The most expensive judgment call in the campaign.",
            ),
            Role::GerenteSetor => (
                "Le mercado e define a linha criativa da rede. Precisa raciocinar, nao so escrever.",
                "Reads the market and sets the creative line for its network. Needs reasoning, not just writing.",
            ),
            Role::Criador => (
                "Recebe um briefing fechado e produz. Nao decide nada, entao nao precisa de raciocinio caro.",
                "Gets a closed brief and produces. It decides nothing, so it needs no expensive reasoning.",
            ),
            Role::Auditor => (
                "Critica com rigor, mas dentro de criterios que ja recebeu prontos.",
                "Reviews rigorously, but against criteria it was handed.",
            ),
            Role::MotionDesigner => (
                "Escolhe o que se move e por que. Interpreta a peca, mas nao redefine a campanha.",
                "Chooses what moves and why. It interprets the piece without redefining the campaign.",
            ),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Role::DiretorGeral => "Diretor Geral",
            Role::GerenteSetor => "Gerente de Setor",
            Role::Criador => "Criador de Conteudo",
            Role::Auditor => "Auditor",
            Role::MotionDesigner => "Motion Designer",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Role::DiretorGeral => "diretor-geral",
            Role::GerenteSetor => "gerente-setor",
            Role::Criador => "criador",
            Role::Auditor => "auditor",
            Role::MotionDesigner => "motion-designer",
        }
    }

    /// Para quem este cargo tem permissao de enviar mensagem. O middleware
    /// recusa qualquer entrega fora deste organograma.
    pub fn may_send_to(&self) -> &'static [Role] {
        match self {
            Role::DiretorGeral => &[Role::GerenteSetor],
            // O gerente fala com o criador, responde ao auditor na revisao, e
            // e o unico que pode acionar o motion designer.
            Role::GerenteSetor => &[Role::Criador, Role::Auditor, Role::MotionDesigner],
            Role::Criador => &[Role::Auditor],
            // Fecha com o gerente; quando ha varias redes, quem fecha e o diretor.
            Role::Auditor => &[Role::GerenteSetor, Role::DiretorGeral],
            // Entrega o roteiro de volta a quem o acionou.
            Role::MotionDesigner => &[Role::GerenteSetor],
        }
    }

    pub fn can_send_to(&self, other: Role) -> bool {
        self.may_send_to().contains(&other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Instagram,
    Facebook,
    Tiktok,
    Linkedin,
    X,
}

impl Network {
    pub fn label(&self) -> &'static str {
        match self {
            Network::Instagram => "Instagram",
            Network::Facebook => "Facebook",
            Network::Tiktok => "TikTok",
            Network::Linkedin => "LinkedIn",
            Network::X => "X",
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Network::Instagram => "instagram",
            Network::Facebook => "facebook",
            Network::Tiktok => "tiktok",
            Network::Linkedin => "linkedin",
            Network::X => "x",
        }
    }

    pub fn home_url(&self) -> &'static str {
        match self {
            Network::Instagram => "https://www.instagram.com/",
            Network::Facebook => "https://www.facebook.com/",
            Network::Tiktok => "https://www.tiktok.com/",
            Network::Linkedin => "https://www.linkedin.com/feed/",
            Network::X => "https://x.com/home",
        }
    }

    /// Restricoes de formato que entram no briefing do criador.
    pub fn format_hint(&self) -> &'static str {
        match self {
            Network::Instagram => "Imagem quadrada 1:1 ou vertical 4:5. Legenda ate 2200 caracteres, com quebra de linha e ate 30 hashtags.",
            Network::Facebook => "Imagem 1.91:1 horizontal. Legenda conversacional, sem excesso de hashtag.",
            Network::Tiktok => "Formato vertical 9:16. Texto curto e direto, gancho nos 2 primeiros segundos.",
            Network::Linkedin => "Imagem 1.91:1. Tom profissional, primeira linha precisa segurar antes do 'ver mais'.",
            Network::X => "Imagem 16:9. Texto ate 280 caracteres.",
        }
    }

    pub fn aspect_ratio(&self) -> &'static str {
        match self {
            Network::Instagram => "1:1",
            Network::Facebook => "16:9",
            Network::Tiktok => "9:16",
            Network::Linkedin => "16:9",
            Network::X => "16:9",
        }
    }
}
