//! A declaracao de video que o Motion Designer devolve, e como ela e lida.
//!
//! O cargo NAO escreve TSX. Ele devolve um JSON de cenas, e uma biblioteca de
//! componentes Remotion versionada em `motion/` renderiza. A escolha foi
//! deliberada e tem tres razoes, em ordem de peso:
//!
//! 1. A TESE DO PRODUTO EXIGE. O provedor padrao e o Ollama, e no nivel medio
//!    isso e um modelo pequeno rodando a poucos tokens por segundo. Um modelo
//!    desse porte nao escreve TSX que compila. Um recurso que so funcionasse
//!    com Claude Code ou Gemini CLI seria um recurso que a maioria de quem
//!    instala nao teria.
//!
//! 2. A FALHA FICA VISIVEL. JSON invalido e um erro que a tela mostra e que o
//!    teste pega. TSX que compila mas anima errado e um video estranho que
//!    ninguem sabe explicar.
//!
//! 3. NAO SE EXECUTA CODIGO ESCRITO PELO MODELO. O produto ja desliga as
//!    ferramentas dos provedores de fora justamente para o agente nao mexer no
//!    disco de ninguem. Compilar e rodar TSX gerado por ele desfaria isso pela
//!    porta dos fundos.
//!
//! O PRECO, dito em voz alta porque o produto nao esconde limite: a
//! criatividade fica presa ao catalogo de tipos de cena. O motion designer
//! escolhe o que se move e por que, dentro de um vocabulario fechado — nao
//! inventa uma cena que nao existe.
//!
//! TUDO E MEDIDO EM SEGUNDOS, NAO EM QUADROS. O cargo raciocina sobre tempo de
//! leitura e tempo de fala, que sao segundos; converter para quadro e trabalho
//! do render, que e quem sabe o fps. Pedir quadros ao modelo seria pedir uma
//! multiplicacao que ele erra e que ninguem precisava que ele fizesse.

use serde::{Deserialize, Serialize};

use super::direcao::{Direcao, Look};

/// Quanto tempo uma cena pode durar.
///
/// O piso existe porque uma cena de 0,2s e um flash que ninguem le. O teto
/// existe porque uma cena de um minuto e a tela parada — e os dois aparecem
/// quando um modelo pequeno erra a unidade e escreve `dur_s: 240` achando que
/// eram quadros.
const DUR_MIN_S: f32 = 0.6;
const DUR_MAX_S: f32 = 20.0;

/// Teto de cenas por video.
///
/// Nao e limite tecnico: e o ponto em que o render deixa de ser "alguns
/// minutos" e vira uma espera que a tela teria que explicar.
const MAX_CENAS: usize = 40;

/// O que uma cena pode ser.
///
/// Fechado de proposito. Um `tipo` livre viraria o modelo inventando
/// `"explosao-cinematica"` e o render caindo num ramo que nao existe — falha
/// no fim do processo, depois de a pessoa ja ter esperado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoCena {
    /// Titulo sobre fundo limpo. Abre e fecha o video.
    Titulo,
    /// Uma imagem com movimento lento de camera. O trabalho pesado do formato:
    /// e o que faz foto parada parecer video.
    KenBurns,
    /// Imagem inteira, parada, com legenda embaixo.
    Placa,
    /// Duas imagens lado a lado.
    Comparacao,
    /// So texto grande, para o momento em que a frase e o conteudo.
    Declaracao,
    /// Fecho com chamada para acao.
    Fecho,
}

impl TipoCena {
    /// Quantas imagens este tipo consome.
    ///
    /// O render precisa saber para nao pedir um arquivo que a cena nao tem, e
    /// a validacao precisa saber para recusar antes de renderizar em vez de
    /// depois.
    pub fn imagens_necessarias(&self) -> usize {
        match self {
            TipoCena::Titulo | TipoCena::Declaracao | TipoCena::Fecho => 0,
            TipoCena::KenBurns | TipoCena::Placa => 1,
            TipoCena::Comparacao => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cena {
    pub tipo: TipoCena,
    /// Duracao em segundos. Ver `DUR_MIN_S` e `DUR_MAX_S`.
    pub dur_s: f32,
    #[serde(default)]
    pub titulo: String,
    #[serde(default)]
    pub subtitulo: String,
    /// Nomes de arquivo da pasta `imagens/`, nao caminhos.
    ///
    /// O modelo recebe a lista de nomes e devolve nomes. Deixa-lo escrever
    /// caminho seria dar a ele um jeito de apontar para fora da pasta do
    /// projeto, e o produto ja aprendeu essa licao com o slug da galeria.
    #[serde(default)]
    pub imagens: Vec<String>,
    /// Trecho da narracao que cai nesta cena, quando ha narracao.
    #[serde(default)]
    pub narracao: String,
    /// COMO esta cena se parece — ver `direcao.rs`.
    ///
    /// E o que separa este sistema de um template: sem ela, duas cenas do mesmo
    /// tipo sairiam identicas no olhar, por mais que a montagem mudasse.
    #[serde(default)]
    pub direcao: Direcao,
}

/// O video inteiro, do jeito que o Motion Designer o declara.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roteiro {
    pub cenas: Vec<Cena>,
    /// Arquivo da pasta `audio/` que toca por baixo, ou vazio.
    #[serde(default)]
    pub trilha: String,
    /// Proporcao pedida: `16:9`, `9:16` ou `1:1`.
    #[serde(default = "padrao_proporcao")]
    pub proporcao: String,
    /// Por que o video e assim. Vai para a tela e para a transcricao — um
    /// roteiro sem justificativa nao da para criticar.
    #[serde(default)]
    pub racional: String,
    /// A direcao do video inteiro, que cascateia por cima das cenas.
    #[serde(default)]
    pub look: Look,
}

fn padrao_proporcao() -> String {
    "16:9".to_string()
}

impl Roteiro {
    /// Apara o que da para aparar e preenche o que veio vazio.
    ///
    /// RODA ANTES DA VALIDACAO, e a ordem importa. A validacao recusa o que nao
    /// tem conserto (imagem inexistente, duracao absurda); isto conserta o que
    /// tem. Um modelo pequeno quase sempre devolve as cenas SEM o bloco de
    /// direcao — recusar por isso seria o recurso quebrar exatamente na maquina
    /// que o produto existe para atender.
    ///
    /// O padrao vem do INDICE da cena, entao um roteiro sem direcao nenhuma
    /// ainda sai alternando em vez de repetindo. Ver `Direcao::padrao_da_cena`.
    pub fn normalizar(mut self) -> Self {
        self.look = self.look.aparar();
        for (i, c) in self.cenas.iter_mut().enumerate() {
            // `Direcao::default()` e o sinal de "o modelo nao disse nada": a
            // serializacao preenche assim quando o bloco falta inteiro. Quando
            // ele disse algo, respeitamos — mesmo que so um campo.
            c.direcao = if c.direcao == Direcao::default() {
                Direcao::padrao_da_cena(i)
            } else {
                c.direcao.aparar()
            };
        }
        self
    }

    pub fn duracao_s(&self) -> f32 {
        self.cenas.iter().map(|c| c.dur_s).sum()
    }

    /// As dimensoes que a proporcao pede.
    ///
    /// Numeros pares em ambos os eixos: um codec H.264 recusa dimensao impar,
    /// e a falha sairia do ffmpeg no fim do render em vez daqui.
    pub fn dimensoes(&self) -> (u32, u32) {
        match self.proporcao.trim() {
            "9:16" => (1080, 1920),
            "1:1" => (1080, 1080),
            _ => (1920, 1080),
        }
    }
}

/// Por que o roteiro nao serve.
///
/// Enum e nao `String` porque a tela responde diferente a cada caso: um
/// roteiro sem cena pede outra rodada do cargo, e um que cita imagem
/// inexistente pede que a pessoa suba o arquivo. Reconhecer isso pelo texto da
/// mensagem quebraria na primeira traducao.
#[derive(Debug, Clone, PartialEq)]
pub enum ErroRoteiro {
    SemCenas,
    CenasDemais(usize),
    DuracaoForaDaFaixa {
        cena: usize,
        dur_s: f32,
    },
    FaltaImagem {
        cena: usize,
        precisa: usize,
        tem: usize,
    },
    ImagemInexistente {
        cena: usize,
        nome: String,
    },
    TrilhaInexistente(String),
}

impl std::fmt::Display for ErroRoteiro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = match self {
            ErroRoteiro::SemCenas => crate::idioma::msg(
                "O roteiro voltou sem nenhuma cena.",
                "The script came back with no scenes.",
            ),
            ErroRoteiro::CenasDemais(n) => crate::idioma::msg(
                &format!("O roteiro tem {n} cenas, e o teto e {MAX_CENAS}."),
                &format!("The script has {n} scenes, and the cap is {MAX_CENAS}."),
            ),
            ErroRoteiro::DuracaoForaDaFaixa { cena, dur_s } => crate::idioma::msg(
                &format!(
                    "A cena {cena} dura {dur_s:.1}s, fora da faixa de {DUR_MIN_S} a {DUR_MAX_S}s."
                ),
                &format!(
                    "Scene {cena} lasts {dur_s:.1}s, outside the {DUR_MIN_S}–{DUR_MAX_S}s range."
                ),
            ),
            ErroRoteiro::FaltaImagem { cena, precisa, tem } => crate::idioma::msg(
                &format!("A cena {cena} precisa de {precisa} imagem(ns) e recebeu {tem}."),
                &format!("Scene {cena} needs {precisa} image(s) and got {tem}."),
            ),
            ErroRoteiro::ImagemInexistente { cena, nome } => crate::idioma::msg(
                &format!("A cena {cena} pede a imagem \"{nome}\", que nao esta na pasta do video."),
                &format!(
                    "Scene {cena} asks for image \"{nome}\", which is not in the video folder."
                ),
            ),
            ErroRoteiro::TrilhaInexistente(nome) => crate::idioma::msg(
                &format!("A trilha \"{nome}\" nao esta na pasta de audio deste video."),
                &format!("Track \"{nome}\" is not in this video's audio folder."),
            ),
        };
        write!(f, "{t}")
    }
}

/// Confere o roteiro contra os arquivos que existem de verdade.
///
/// ISTO RODA ANTES DO RENDER, E E A RAZAO DE O MODULO EXISTIR. Um roteiro que
/// cita `produto-05.jpg` num projeto que so tem quatro imagens renderizaria um
/// quadro preto no meio do video — e a pessoa descobriria depois de esperar o
/// render inteiro, olhando um arquivo pronto e errado. Recusar aqui custa
/// segundos e diz exatamente o que falta.
pub fn validar(r: &Roteiro, projeto: &super::assets::Projeto) -> Result<(), ErroRoteiro> {
    if r.cenas.is_empty() {
        return Err(ErroRoteiro::SemCenas);
    }
    if r.cenas.len() > MAX_CENAS {
        return Err(ErroRoteiro::CenasDemais(r.cenas.len()));
    }

    let tem_imagem = |nome: &str| projeto.imagens.iter().any(|i| i.nome == nome);

    for (i, c) in r.cenas.iter().enumerate() {
        // Numerada a partir de 1: a mensagem vai para a tela, e "cena 0" nao
        // corresponde a nada que a pessoa consiga contar no roteiro.
        let cena = i + 1;

        if !(DUR_MIN_S..=DUR_MAX_S).contains(&c.dur_s) {
            return Err(ErroRoteiro::DuracaoForaDaFaixa {
                cena,
                dur_s: c.dur_s,
            });
        }

        let precisa = c.tipo.imagens_necessarias();
        if c.imagens.len() < precisa {
            return Err(ErroRoteiro::FaltaImagem {
                cena,
                precisa,
                tem: c.imagens.len(),
            });
        }
        for nome in c.imagens.iter().take(precisa) {
            if !tem_imagem(nome) {
                return Err(ErroRoteiro::ImagemInexistente {
                    cena,
                    nome: nome.clone(),
                });
            }
        }
    }

    if !r.trilha.trim().is_empty() && !projeto.audio.iter().any(|a| a.nome == r.trilha) {
        return Err(ErroRoteiro::TrilhaInexistente(r.trilha.clone()));
    }

    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::video::assets::{Item, Projeto};

    fn item(nome: &str) -> Item {
        Item {
            nome: nome.into(),
            caminho: format!("/tmp/v/{nome}"),
            bytes: 1,
        }
    }

    fn projeto() -> Projeto {
        Projeto {
            slug: "v".into(),
            nome: "v".into(),
            caminho: "/tmp/v".into(),
            imagens: vec![item("a.png"), item("b.png")],
            audio: vec![item("trilha.mp3")],
            narracao: vec![],
            saidas: vec![],
            bytes: 4,
        }
    }

    fn cena(tipo: TipoCena, imagens: &[&str]) -> Cena {
        Cena {
            tipo,
            dur_s: 3.0,
            titulo: "t".into(),
            subtitulo: String::new(),
            imagens: imagens.iter().map(|s| s.to_string()).collect(),
            narracao: String::new(),
            direcao: Direcao::default(),
        }
    }

    fn roteiro(cenas: Vec<Cena>) -> Roteiro {
        Roteiro {
            cenas,
            trilha: String::new(),
            proporcao: "16:9".into(),
            racional: String::new(),
            look: Look::default(),
        }
    }

    #[test]
    fn imagem_que_nao_existe_e_recusada_antes_do_render() {
        // Sem isto o render produziria um quadro preto no meio do video, e a
        // pessoa so descobriria depois de esperar o arquivo inteiro ficar
        // pronto — olhando um resultado errado sem saber por que.
        let r = roteiro(vec![cena(TipoCena::KenBurns, &["nao-existe.png"])]);
        assert_eq!(
            validar(&r, &projeto()),
            Err(ErroRoteiro::ImagemInexistente {
                cena: 1,
                nome: "nao-existe.png".into()
            })
        );
    }

    #[test]
    fn cena_que_precisa_de_duas_imagens_nao_passa_com_uma() {
        let r = roteiro(vec![cena(TipoCena::Comparacao, &["a.png"])]);
        assert_eq!(
            validar(&r, &projeto()),
            Err(ErroRoteiro::FaltaImagem {
                cena: 1,
                precisa: 2,
                tem: 1
            })
        );
    }

    #[test]
    fn duracao_em_quadros_disfarcada_de_segundos_e_pega() {
        // O erro classico do modelo pequeno: escrever 90 achando que sao
        // quadros. Sem o teto, isso viraria uma cena de um minoto e meio com a
        // tela parada, e o video sairia com trinta minutos de duracao.
        let mut r = roteiro(vec![cena(TipoCena::Titulo, &[])]);
        r.cenas[0].dur_s = 90.0;
        assert!(matches!(
            validar(&r, &projeto()),
            Err(ErroRoteiro::DuracaoForaDaFaixa { cena: 1, .. })
        ));
    }

    #[test]
    fn cena_que_pisca_tambem_e_recusada() {
        let mut r = roteiro(vec![cena(TipoCena::Titulo, &[])]);
        r.cenas[0].dur_s = 0.1;
        assert!(matches!(
            validar(&r, &projeto()),
            Err(ErroRoteiro::DuracaoForaDaFaixa { cena: 1, .. })
        ));
    }

    #[test]
    fn trilha_inexistente_nao_passa() {
        let mut r = roteiro(vec![cena(TipoCena::Titulo, &[])]);
        r.trilha = "sumida.mp3".into();
        assert_eq!(
            validar(&r, &projeto()),
            Err(ErroRoteiro::TrilhaInexistente("sumida.mp3".into()))
        );
    }

    #[test]
    fn roteiro_valido_passa() {
        let mut r = roteiro(vec![
            cena(TipoCena::Titulo, &[]),
            cena(TipoCena::KenBurns, &["a.png"]),
            cena(TipoCena::Comparacao, &["a.png", "b.png"]),
            cena(TipoCena::Fecho, &[]),
        ]);
        r.trilha = "trilha.mp3".into();
        assert_eq!(validar(&r, &projeto()), Ok(()));
        assert_eq!(r.duracao_s(), 12.0);
    }

    #[test]
    fn roteiro_sem_direcao_sai_da_normalizacao_com_cenas_diferentes() {
        // O caso real: modelo pequeno devolve as cenas sem o bloco de direcao.
        // Sem a normalizacao por indice, as seis sairiam identicas no olhar —
        // estrutura nova, aparencia de template.
        let r = roteiro(vec![cena(TipoCena::KenBurns, &["a.png"]); 6]).normalizar();
        let distintas: std::collections::HashSet<_> =
            r.cenas.iter().map(|c| c.direcao.movimento).collect();
        assert!(
            distintas.len() >= 5,
            "seis cenas sem direcao deviam variar, e vieram {distintas:?}"
        );
    }

    #[test]
    fn a_direcao_que_o_modelo_escreveu_nao_e_sobrescrita() {
        // A normalizacao preenche o que faltou; ela nao decide no lugar de quem
        // decidiu. Um modelo que escolheu "parado" quis silencio visual, e
        // trocar isso por um movimento de catalogo seria o sistema dirigindo
        // por cima do diretor.
        let mut c = cena(TipoCena::KenBurns, &["a.png"]);
        c.direcao = Direcao {
            movimento: crate::video::direcao::Movimento::Nenhum,
            ..Direcao::default()
        };
        let r = roteiro(vec![c]).normalizar();
        assert_eq!(
            r.cenas[0].direcao.movimento,
            crate::video::direcao::Movimento::Nenhum
        );
    }

    #[test]
    fn as_dimensoes_sao_sempre_pares() {
        // Um codec H.264 recusa dimensao impar, e a falha sairia do ffmpeg no
        // fim do render em vez de aqui.
        for p in ["16:9", "9:16", "1:1", "coisa-inventada"] {
            let r = Roteiro {
                cenas: vec![],
                trilha: String::new(),
                proporcao: p.into(),
                racional: String::new(),
                look: Look::default(),
            };
            let (w, h) = r.dimensoes();
            assert_eq!(w % 2, 0, "{p}: largura impar");
            assert_eq!(h % 2, 0, "{p}: altura impar");
        }
    }

    #[test]
    fn proporcao_desconhecida_cai_no_horizontal_em_vez_de_falhar() {
        // O modelo pode escrever "4:5" ou "vertical". Recusar o roteiro
        // inteiro por causa disso jogaria fora um trabalho de minutos por um
        // campo que tem padrao obvio.
        let r = Roteiro {
            cenas: vec![],
            trilha: String::new(),
            proporcao: "4:5".into(),
            racional: String::new(),
            look: Look::default(),
        };
        assert_eq!(r.dimensoes(), (1920, 1080));
    }
}
