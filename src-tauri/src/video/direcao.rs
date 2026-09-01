//! A DIREÇÃO: o que impede o video de ser um template.
//!
//! O `spec.rs` diz QUE cenas existem. Este modulo diz COMO cada uma delas se
//! parece — e e a diferenca entre um sistema que preenche lacunas e um que
//! dirige.
//!
//! O PROBLEMA QUE ISTO RESOLVE. Na primeira versao, o Motion Designer escolhia
//! quantas cenas, quais, em que ordem e com que duracao. Estrutura nova a cada
//! rodada — mas a APARENCIA de cada tipo era fixa no componente: uma
//! `ken_burns` sempre aproximava do centro, o texto sempre na mesma tarja. Dois
//! videos diferentes na montagem e identicos no olhar. Isso e template, mesmo
//! que a estrutura mude.
//!
//! A correcao nao e deixar o modelo escrever TSX — pelas tres razoes no topo do
//! `spec.rs`, que continuam valendo. E fazer o JSON descrever DIRECAO em vez de
//! so preencher campo. Um diretor de verdade escolhe de onde a camera parte,
//! para onde ela vai, onde o texto pousa e com que energia o corte acontece.
//! Cada uma dessas escolhas e um enum pequeno; juntas, elas abrem um espaco
//! grande o bastante para dois videos nunca se parecerem.
//!
//! TODO CAMPO TEM PADRAO. Um modelo pequeno vai esquecer metade deles, e um
//! roteiro recusado por falta de `foco` seria o recurso quebrando justamente
//! na maquina que o produto existe para atender. Faltou, entra o padrao — e o
//! padrao NAO e o mesmo para todo mundo: ele vem do indice da cena, para que um
//! roteiro sem direcao nenhuma ainda alterne em vez de repetir.

use serde::{Deserialize, Serialize};

/// Para onde a camera anda sobre uma imagem parada.
///
/// E o campo que mais muda o resultado: e o movimento que faz foto virar video.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Movimento {
    #[default]
    Aproximar,
    Afastar,
    VarrerEsquerda,
    VarrerDireita,
    Subir,
    Descer,
    /// Imagem parada de verdade. Existe para a cena que precisa de silencio
    /// visual — sem esta opcao, um video inteiro se move e nada respira.
    Nenhum,
}

/// De que ponto da imagem a camera parte.
///
/// Separado do movimento porque as duas escolhas sao independentes: aproximar
/// do canto superior e aproximar do centro sao enquadramentos diferentes da
/// mesma foto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Foco {
    #[default]
    Centro,
    Topo,
    Base,
    Esquerda,
    Direita,
}

/// Onde o texto pousa no quadro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Pouso {
    #[default]
    InferiorEsquerda,
    InferiorDireita,
    SuperiorEsquerda,
    Centro,
    /// Coluna estreita colada numa borda: o texto vira bloco vertical em vez de
    /// faixa horizontal. Muda a leitura do quadro inteiro.
    ColunaEsquerda,
}

/// Como a cena entra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Entrada {
    #[default]
    Fade,
    /// Sobe do rodape. Da peso ao que entra.
    Subir,
    /// Cresce do centro.
    Escala,
    /// Cortina lateral.
    Cortina,
    /// Sem transicao. Um corte seco entre duas cenas e uma escolha de ritmo,
    /// nao a ausencia de uma.
    Corte,
}

/// A direcao de UMA cena.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Direcao {
    #[serde(default)]
    pub movimento: Movimento,
    #[serde(default)]
    pub foco: Foco,
    #[serde(default)]
    pub pouso: Pouso,
    #[serde(default)]
    pub entrada: Entrada,
    /// Multiplicador do corpo do texto. Fora de 0,6–1,8 o texto some ou
    /// estoura a caixa, e nos dois casos o quadro fica inutilizavel.
    #[serde(default = "um")]
    pub escala_texto: f32,
}

fn um() -> f32 {
    1.0
}

impl Default for Direcao {
    fn default() -> Self {
        Self {
            movimento: Movimento::default(),
            foco: Foco::default(),
            pouso: Pouso::default(),
            entrada: Entrada::default(),
            escala_texto: 1.0,
        }
    }
}

/// Escalas aceitas para o texto.
const ESCALA_MIN: f32 = 0.6;
const ESCALA_MAX: f32 = 1.8;

impl Direcao {
    /// A direcao que uma cena sem direcao recebe.
    ///
    /// VARIA COM O INDICE, e isso e o ponto. Um padrao constante faria um
    /// roteiro que veio sem direcao nenhuma — o que um modelo pequeno produz
    /// com frequencia — sair com as dez cenas iguais, que e exatamente o
    /// template que este modulo existe para evitar. Alternando, o piso do
    /// sistema ja e um video que se move.
    ///
    /// Os ciclos tem comprimentos diferentes (6, 5, 4) de proposito: se os tres
    /// tivessem o mesmo, a combinacao se repetiria a cada volta e o padrao
    /// voltaria a ser visivel. Com 6, 5 e 4 a combinacao so fecha em 60 cenas —
    /// mais que o teto de 40.
    pub fn padrao_da_cena(i: usize) -> Self {
        const MOVS: [Movimento; 6] = [
            Movimento::Aproximar,
            Movimento::VarrerDireita,
            Movimento::Afastar,
            Movimento::Subir,
            Movimento::VarrerEsquerda,
            Movimento::Descer,
        ];
        const FOCOS: [Foco; 5] = [
            Foco::Centro,
            Foco::Esquerda,
            Foco::Topo,
            Foco::Direita,
            Foco::Base,
        ];
        const POUSOS: [Pouso; 4] = [
            Pouso::InferiorEsquerda,
            Pouso::ColunaEsquerda,
            Pouso::InferiorDireita,
            Pouso::SuperiorEsquerda,
        ];
        Self {
            movimento: MOVS[i % MOVS.len()],
            foco: FOCOS[i % FOCOS.len()],
            pouso: POUSOS[i % POUSOS.len()],
            entrada: Entrada::default(),
            escala_texto: 1.0,
        }
    }

    /// Aparo o que nao da para renderizar, em vez de recusar o roteiro.
    ///
    /// Uma escala de texto fora da faixa nao justifica jogar fora um trabalho
    /// de minutos: o campo tem um valor obviamente correto perto do que veio.
    /// Isto e diferente de uma imagem inexistente, que o `spec.rs` recusa —
    /// la nao ha valor perto que salve a cena.
    pub fn aparar(mut self) -> Self {
        if !self.escala_texto.is_finite() {
            self.escala_texto = 1.0;
        }
        self.escala_texto = self.escala_texto.clamp(ESCALA_MIN, ESCALA_MAX);
        self
    }
}

/// A direcao do VIDEO inteiro: o que cascateia por cima de todas as cenas.
///
/// Existe separada da direcao de cena porque e outra decisao. O diretor escolhe
/// uma vez como o filme respira, e depois enquadra cada plano dentro disso. Sem
/// esta camada, um roteiro poderia ter uma cena calma seguida de uma frenetica
/// sem nada amarrando as duas — que e a marca de um video montado por varias
/// maos, o mesmo defeito que a curva de saida unica evita.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Look {
    /// Quanta amplitude o movimento tem. 0 = quase parado, 1 = agressivo.
    ///
    /// Multiplica o deslocamento da camera e encurta as transicoes. E a
    /// alavanca que separa um institucional de um corte de rede social.
    #[serde(default = "meio")]
    pub energia: f32,
    /// Vinheta nas bordas. Assenta o quadro e ajuda o texto a vencer.
    #[serde(default)]
    pub vinheta: bool,
    /// O acento da marca aparece como barra/filete nas cenas de texto.
    #[serde(default)]
    pub filete: bool,
}

fn meio() -> f32 {
    0.5
}

impl Default for Look {
    fn default() -> Self {
        Self {
            energia: 0.5,
            vinheta: false,
            filete: false,
        }
    }
}

impl Look {
    pub fn aparar(mut self) -> Self {
        if !self.energia.is_finite() {
            self.energia = 0.5;
        }
        self.energia = self.energia.clamp(0.0, 1.0);
        self
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn um_roteiro_sem_direcao_nenhuma_ainda_alterna() {
        // E o caso que mais acontece: modelo pequeno devolve as cenas sem o
        // bloco de direcao. Se o padrao fosse constante, as dez cenas sairiam
        // identicas — o template que este modulo existe para evitar.
        let seis: Vec<Direcao> = (0..6).map(Direcao::padrao_da_cena).collect();
        let movimentos: std::collections::HashSet<_> = seis.iter().map(|d| d.movimento).collect();
        assert!(
            movimentos.len() >= 5,
            "seis cenas seguidas deviam variar o movimento, e vieram {movimentos:?}"
        );
    }

    #[test]
    fn a_combinacao_nao_se_repete_dentro_do_teto_de_cenas() {
        // Os ciclos tem 6, 5 e 4 passos: se tivessem o mesmo comprimento, a
        // combinacao fecharia a cada volta e o padrao voltaria a ser visivel.
        // Com esses tres, ela so fecha em 60 — acima do teto de 40 cenas.
        let combos: std::collections::HashSet<_> = (0..40)
            .map(Direcao::padrao_da_cena)
            .map(|d| (d.movimento, d.foco, d.pouso))
            .collect();
        assert_eq!(
            combos.len(),
            40,
            "as 40 cenas do teto deviam ter 40 direcoes distintas"
        );
    }

    #[test]
    fn escala_absurda_e_aparada_e_nao_derruba_o_roteiro() {
        // Diferente de uma imagem inexistente: aqui ha um valor obviamente
        // correto perto do que veio, e jogar fora minutos de trabalho por causa
        // de um numero seria desproporcional.
        for entrada in [0.0, -3.0, 99.0, f32::NAN, f32::INFINITY] {
            let d = Direcao {
                escala_texto: entrada,
                ..Direcao::default()
            }
            .aparar();
            assert!(
                (ESCALA_MIN..=ESCALA_MAX).contains(&d.escala_texto),
                "{entrada} virou {}",
                d.escala_texto
            );
        }
    }

    #[test]
    fn energia_fora_da_faixa_e_aparada() {
        for e in [-1.0, 7.0, f32::NAN] {
            let l = Look {
                energia: e,
                ..Look::default()
            }
            .aparar();
            assert!((0.0..=1.0).contains(&l.energia), "{e} virou {}", l.energia);
        }
    }
}
