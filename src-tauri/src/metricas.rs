//! Desempenho das pecas publicadas, e o que a proxima campanha faz com ele.
//!
//! Um gerador de conteudo sem retorno de desempenho e um gerador de opiniao:
//! ele repete o que o modelo acha bonito, nao o que funcionou. Este modulo
//! fecha o laco. Cada peca publicada vira um registro; os registros viram uma
//! leitura; e a leitura entra no prompt do cargo que decide a proxima campanha.
//!
//! A REGRA QUE MANDA AQUI, e ela e do produto, nao da estatistica:
//!
//!   o padrao e DIVERGIR. Repetir o que funcionou parece seguro e e a forma
//!   mais rapida de estagnar: o algoritmo das redes pune a repeticao e o
//!   publico se acostuma. Entao a leitura sai como uma ordem de melhorar, com
//!   o que funcionou servindo de piso a superar, nao de molde a copiar.
//!
//!   A unica excecao e o acerto extraordinario. Quando uma peca bate um
//!   multiplo grande da mediana propria daquela rede, aquilo deixou de ser
//!   ruido e virou veio: ai vale continuar na mesma linha enquanto ela render.
//!
//! O corte entre os dois casos e `LIMIAR_VIRAL`, e ele so vale com historico
//! suficiente. Com tres publicacoes, a "melhor" e sorte; chamar isso de viral
//! ensinaria a proxima campanha a copiar um acidente.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::platform;

/// Quantas vezes acima da mediana uma peca precisa render para o sistema
/// aceitar seguir na mesma linha em vez de exigir algo novo.
const LIMIAR_VIRAL: f64 = 3.0;

/// Historico minimo para uma mediana significar alguma coisa.
const MINIMO_PARA_COMPARAR: usize = 4;

/// De onde vieram os numeros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origem {
    /// A pessoa digitou olhando o painel da rede. Sempre funciona.
    Manual,
    /// Lido da pagina pelo navegador. Rapido, e quebra quando a rede muda.
    Raspagem,
}

/// O desempenho de uma peca publicada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registro {
    pub id: String,
    /// Campanha que gerou a peca, quando ela nasceu aqui dentro.
    #[serde(default)]
    pub run_id: String,
    pub rede: String,
    /// Data da publicacao, `AAAA-MM-DD`.
    pub publicado_em: String,
    #[serde(default)]
    pub url: String,
    /// O que a peca era, em uma linha. E o que a leitura vai correlacionar.
    pub conceito: String,
    #[serde(default)]
    pub impressoes: u64,
    #[serde(default)]
    pub curtidas: u64,
    #[serde(default)]
    pub comentarios: u64,
    #[serde(default)]
    pub compartilhamentos: u64,
    #[serde(default)]
    pub salvamentos: u64,
    #[serde(default)]
    pub cliques: u64,
    pub origem: Origem,
    pub coletado_em: String,
}

impl Registro {
    /// Interacao ponderada pelo custo de cada gesto.
    ///
    /// Curtir e quase gratuito; comentar exige escrever; compartilhar exige
    /// colocar a propria reputacao junto. Somar tudo com peso 1 faz o numero
    /// ser dominado pela curtida, que e justamente o sinal mais fraco.
    pub fn interacoes(&self) -> f64 {
        self.curtidas as f64
            + 2.0 * self.comentarios as f64
            + 3.0 * self.compartilhamentos as f64
            + 2.0 * self.salvamentos as f64
            + 1.5 * self.cliques as f64
    }

    /// Engajamento por mil impressoes.
    ///
    /// `None` quando a rede nao deu alcance. Alcance quase sempre mora no
    /// painel profissional, que a raspagem nao alcanca: por isso o sistema
    /// tambem sabe ranquear sem ele, ver `Base`.
    pub fn taxa(&self) -> Option<f64> {
        (self.impressoes > 0).then(|| self.interacoes() / self.impressoes as f64 * 1000.0)
    }
}

fn caminho() -> PathBuf {
    platform::current().data_dir().join("metricas.json")
}

pub fn load() -> Vec<Registro> {
    std::fs::read_to_string(caminho())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save(regs: &[Registro]) -> Result<(), String> {
    let p = caminho();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let texto = serde_json::to_string_pretty(regs).map_err(|e| e.to_string())?;
    // Grava ao lado e renomeia: um desligamento no meio da escrita nao deixa
    // um arquivo pela metade, que aqui significaria perder todo o historico.
    let tmp = p.with_extension("json.tmp");
    std::fs::write(&tmp, texto).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
}

// ------------------------------------------------------------------ leitura

/// Em que escala as pecas de uma rede estao sendo comparadas.
///
/// Existe porque impressao e um dado privilegiado: ela mora no painel
/// profissional da rede, e a raspagem quase nunca a alcanca. Sem esta
/// distincao, todo registro coletado automaticamente ficaria de fora do
/// ranking e a auditoria so funcionaria para quem digitasse tudo a mao.
///
/// A regra: taxa e melhor, entao ela vence quando ha registros suficientes com
/// alcance. Quando nao ha, o sistema compara volume bruto e AVISA, porque
/// volume tambem mede o tamanho da audiencia daquele dia, nao so a peca.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Base {
    /// Interacao ponderada por mil impressoes. A leitura confiavel.
    Taxa,
    /// Interacao ponderada bruta. Aproximacao, quando nao ha alcance.
    Volume,
}

/// O que o sistema conclui do historico de uma rede.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Veredito {
    /// Historico curto demais para comparar. A proxima campanha corre solta.
    SemBase,
    /// O caso normal: ha um piso a superar, e repetir e proibido.
    Divergir,
    /// Uma peca destoou tanto que a linha dela virou ativo. Seguir enquanto rende.
    Seguir,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeituraDaRede {
    pub rede: String,
    pub publicacoes: usize,
    pub veredito: Veredito,
    /// Em que escala este ranking foi calculado.
    pub base: Base,
    /// Registros descartados por nao caberem na base escolhida.
    pub fora_da_base: usize,
    /// Mediana da taxa, a referencia contra a qual tudo e medido.
    pub mediana: f64,
    /// Quantas vezes a melhor peca bateu a mediana.
    pub multiplo_da_melhor: f64,
    pub melhor_conceito: String,
    pub pior_conceito: String,
    /// Peca a peca, da melhor para a pior, para a tela desenhar.
    pub ranking: Vec<ItemRanking>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemRanking {
    pub id: String,
    pub conceito: String,
    pub publicado_em: String,
    pub taxa: f64,
    /// Razao contra a mediana. 1,0 e exatamente a mediana.
    pub multiplo: f64,
    pub interacoes: f64,
    pub impressoes: u64,
}

fn mediana(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// Le o historico de uma rede e decide o que a proxima campanha deve fazer.
pub fn ler_rede(regs: &[Registro], rede: &str) -> LeituraDaRede {
    let da_rede: Vec<&Registro> = regs.iter().filter(|r| r.rede == rede).collect();

    // Escolhe a base antes de ranquear. Nunca se misturam as duas: taxa esta em
    // interacao por mil impressoes e volume em interacao absoluta, e um ranking
    // que soma as duas ordena por escala, nao por desempenho.
    let com_alcance = da_rede.iter().filter(|r| r.impressoes > 0).count();
    let base = if com_alcance >= MINIMO_PARA_COMPARAR || com_alcance * 2 >= da_rede.len() {
        Base::Taxa
    } else {
        Base::Volume
    };

    let mut com_taxa: Vec<(&Registro, f64)> = da_rede
        .iter()
        .filter_map(|r| match base {
            Base::Taxa => r.taxa().map(|t| (*r, t)),
            Base::Volume => Some((*r, r.interacoes())),
        })
        .collect();
    let fora_da_base = da_rede.len() - com_taxa.len();

    com_taxa.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let med = mediana(com_taxa.iter().map(|(_, t)| *t).collect());
    let n = com_taxa.len();

    let multiplo = |t: f64| if med > 0.0 { t / med } else { 0.0 };
    let melhor = com_taxa
        .first()
        .map(|(r, t)| (r.conceito.clone(), multiplo(*t)));
    let pior = com_taxa.last().map(|(r, _)| r.conceito.clone());

    let (melhor_conceito, multiplo_da_melhor) = melhor.unwrap_or_default();

    let veredito = if n < MINIMO_PARA_COMPARAR {
        Veredito::SemBase
    } else if multiplo_da_melhor >= LIMIAR_VIRAL {
        Veredito::Seguir
    } else {
        Veredito::Divergir
    };

    LeituraDaRede {
        rede: rede.to_string(),
        publicacoes: n,
        veredito,
        base,
        fora_da_base,
        mediana: med,
        multiplo_da_melhor,
        melhor_conceito,
        pior_conceito: pior.unwrap_or_default(),
        ranking: com_taxa
            .iter()
            .map(|(r, t)| ItemRanking {
                id: r.id.clone(),
                conceito: r.conceito.clone(),
                publicado_em: r.publicado_em.clone(),
                taxa: *t,
                multiplo: multiplo(*t),
                interacoes: r.interacoes(),
                impressoes: r.impressoes,
            })
            .collect(),
    }
}

/// Todas as redes que tem historico.
pub fn ler_tudo(regs: &[Registro]) -> Vec<LeituraDaRede> {
    let mut redes: Vec<String> = regs.iter().map(|r| r.rede.clone()).collect();
    redes.sort();
    redes.dedup();
    redes.iter().map(|r| ler_rede(regs, r)).collect()
}

// ------------------------------------------------- o que vai para o prompt

/// O bloco que entra no system do cargo que decide a proxima campanha.
///
/// E aqui que a regra do produto vira instrucao: o texto diz explicitamente
/// que repetir esta proibido e qual e o numero a bater, porque um modelo
/// obedece muito melhor a uma meta nomeada ("supere 12,4 por mil") do que a
/// um pedido vago ("faca melhor que antes").
pub fn bloco_de_desempenho(rede: &str) -> String {
    let leitura = ler_rede(&load(), rede);

    // A unidade acompanha a base. Dizer "por mil impressoes" quando o ranking
    // saiu de volume bruto entregaria ao modelo uma meta numa escala que ele
    // nao tem como bater, e ele obedeceria assim mesmo.
    let unidade = match leitura.base {
        Base::Taxa => "interacao ponderada por mil impressoes",
        Base::Volume => "interacao ponderada bruta (esta conta nao expoe alcance)",
    };

    match leitura.veredito {
        Veredito::SemBase => String::new(),

        Veredito::Divergir => {
            let piores: Vec<&ItemRanking> = leitura.ranking.iter().rev().take(2).collect();
            format!(
                "DESEMPENHO REAL DESTA CONTA. Estes numeros vieram das {n} publicacoes \
                 anteriores em {rede}, medidos em {unidade}. A mediana da conta e \
                 {med:.1}.\n\n\
                 O que mais rendeu: \"{melhor}\" ({mult:.1}x a mediana).\n\
                 O que menos rendeu: {piores}.\n\n\
                 ORDEM: a peca desta campanha precisa SUPERAR {med:.1}. Nao repita o \
                 conceito que mais rendeu: ele ja foi usado, o publico ja viu e o \
                 algoritmo penaliza repeticao. Use o que funcionou apenas para extrair \
                 o PRINCIPIO por tras (o tipo de gancho, o registro, o formato do \
                 argumento) e aplique esse principio a um conceito novo. Diga em uma \
                 frase qual principio voce extraiu e como esta peca o leva adiante.",
                n = leitura.publicacoes,
                rede = rede,
                unidade = unidade,
                med = leitura.mediana,
                melhor = leitura.melhor_conceito,
                mult = leitura.multiplo_da_melhor,
                piores = piores
                    .iter()
                    .map(|p| format!("\"{}\" ({:.1}x)", p.conceito, p.multiplo))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }

        Veredito::Seguir => format!(
            "DESEMPENHO REAL DESTA CONTA. Em {n} publicacoes anteriores em {rede}, uma \
             peca destoou de todas: \"{melhor}\" rendeu {mult:.1}x a mediana da conta \
             ({med:.1}, em {unidade}).\n\n\
             Um multiplo desse tamanho nao e variacao: e um veio. ORDEM: continue nessa \
             mesma linha enquanto ela render. Mantenha o que fez aquela peca funcionar \
             (o angulo, o registro, o formato) e mude o assunto e os exemplos. Nao \
             transforme isso em copia: repetir a mesma peca queima o veio em duas \
             publicacoes. Diga em uma frase o que voce esta mantendo e o que esta \
             trocando.",
            n = leitura.publicacoes,
            rede = rede,
            melhor = leitura.melhor_conceito,
            mult = leitura.multiplo_da_melhor,
            med = leitura.mediana,
            unidade = unidade,
        ),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn reg(rede: &str, conceito: &str, imp: u64, likes: u64) -> Registro {
        Registro {
            id: format!("{conceito}-{likes}"),
            run_id: String::new(),
            rede: rede.into(),
            publicado_em: "2026-08-01".into(),
            url: String::new(),
            conceito: conceito.into(),
            impressoes: imp,
            curtidas: likes,
            comentarios: 0,
            compartilhamentos: 0,
            salvamentos: 0,
            cliques: 0,
            origem: Origem::Manual,
            coletado_em: "2026-08-29".into(),
        }
    }

    #[test]
    fn historico_curto_nao_vira_veredito() {
        let regs = vec![reg("instagram", "a", 1000, 500)];
        let l = ler_rede(&regs, "instagram");
        assert_eq!(l.veredito, Veredito::SemBase);
        assert!(
            bloco_de_desempenho("instagram").is_empty() || l.publicacoes < MINIMO_PARA_COMPARAR
        );
    }

    #[test]
    fn o_padrao_e_divergir_mesmo_com_um_vencedor_claro() {
        // O melhor rende o dobro da mediana. E bom, mas nao e veio.
        let regs = vec![
            reg("instagram", "a", 1000, 100),
            reg("instagram", "b", 1000, 50),
            reg("instagram", "c", 1000, 40),
            reg("instagram", "d", 1000, 30),
        ];
        let l = ler_rede(&regs, "instagram");
        assert_eq!(l.veredito, Veredito::Divergir);
        assert_eq!(l.melhor_conceito, "a");
    }

    #[test]
    fn so_um_multiplo_grande_autoriza_seguir_na_mesma_linha() {
        let regs = vec![
            reg("instagram", "viral", 1000, 900),
            reg("instagram", "b", 1000, 50),
            reg("instagram", "c", 1000, 40),
            reg("instagram", "d", 1000, 30),
        ];
        let l = ler_rede(&regs, "instagram");
        assert_eq!(l.veredito, Veredito::Seguir);
        assert!(l.multiplo_da_melhor >= LIMIAR_VIRAL);
    }

    #[test]
    fn com_alcance_na_maioria_o_ranking_usa_taxa_e_descarta_o_resto() {
        // Metade ou mais com impressao: taxa vence, e quem nao tem fica fora.
        let regs = vec![
            reg("x", "com alcance", 1000, 10),
            reg("x", "sem alcance", 0, 900),
        ];
        let l = ler_rede(&regs, "x");
        assert_eq!(l.base, Base::Taxa);
        assert_eq!(l.publicacoes, 1);
        assert_eq!(l.fora_da_base, 1);
        assert_eq!(l.melhor_conceito, "com alcance");
    }

    #[test]
    fn sem_alcance_o_ranking_cai_para_volume_em_vez_de_ficar_vazio() {
        // Raspagem quase nunca alcanca impressao. Se o ranking exigisse taxa,
        // todo dado coletado automaticamente seria jogado fora.
        let regs = vec![
            reg("x", "a", 0, 300),
            reg("x", "b", 0, 200),
            reg("x", "c", 0, 100),
            reg("x", "d", 0, 50),
        ];
        let l = ler_rede(&regs, "x");
        assert_eq!(l.base, Base::Volume);
        assert_eq!(l.publicacoes, 4);
        assert_eq!(l.fora_da_base, 0);
        assert_eq!(l.melhor_conceito, "a");
    }

    #[test]
    fn compartilhar_pesa_mais_que_curtir() {
        let mut a = reg("x", "a", 1000, 10);
        a.compartilhamentos = 0;
        let mut b = reg("x", "b", 1000, 0);
        b.compartilhamentos = 10;
        assert!(b.interacoes() > a.interacoes());
    }
}
