//! Quando a cota do Gemini CLI acaba no meio de um trabalho.
//!
//! Reusa o tipo `Limite` e a espera do Claude Code: a decisao que a pessoa
//! toma e a mesma (encerrar agora ou esperar a cota voltar), e duplicar o
//! modal e o canal so porque o provedor mudou daria duas telas que precisam
//! ser corrigidas juntas para sempre.
//!
//! O QUE MUDA E O `volta_em`, E ELE E SEMPRE `None` AQUI. O Claude Code
//! escreve o horario de volta na propria saida (`resets 9:10pm`), e por isso
//! la da para esperar exatamente o necessario. O Gemini CLI diz apenas que a
//! cota diaria daquele modelo acabou, sem dizer quando volta. Chutar
//! "meia-noite do Pacifico" seria prometer um horario que nao foi medido — e a
//! regra que ja existe cobre esse caso: sem horario, a tela oferece so
//! encerrar, em vez de prometer uma espera que nao sabe medir.

pub use crate::claude::limite::Limite;

/// Marcas que so aparecem quando e limite de uso, e nao outro erro qualquer.
///
/// Tiradas do proprio binario 0.50.0: a mensagem de usuario
/// (`You have exhausted your daily quota on this model.`), os codigos que a
/// API devolve por baixo (`RESOURCE_EXHAUSTED`, `QUOTA_EXCEEDED`) e o
/// `429`, que e como o esgotamento chega quando o CLI repassa o status cru.
const MARCAS: &[&str] = &[
    "exhausted your daily quota",
    "quota exceeded",
    "resource_exhausted",
    "quota_exceeded",
    "rate limit exceeded",
    "too many requests",
];

/// A saida acusa limite de cota?
///
/// Olha so o FIM do texto, pela mesma razao do detector do Claude Code: um
/// briefing de campanha pode conter as palavras "cota" e "limite" no meio, e
/// procurar no texto inteiro transformaria conteudo de quem usa em falso
/// positivo.
pub fn detectar(texto: &str) -> Option<Limite> {
    let fim = ultimas_linhas(texto, 15);
    let minusc = fim.to_lowercase();
    if !MARCAS.iter().any(|m| minusc.contains(m)) {
        return None;
    }
    Some(Limite {
        volta_em: None,
        evidencia: fim.trim().to_string(),
    })
}

fn ultimas_linhas(texto: &str, n: usize) -> String {
    let linhas: Vec<&str> = texto.lines().filter(|l| !l.trim().is_empty()).collect();
    let corte = linhas.len().saturating_sub(n);
    linhas[corte..].join("\n")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn reconhece_a_mensagem_real_do_cli() {
        // A frase esta no binario 0.50.0, em `QuotaError`.
        let saida = "{\"error\":{\"message\":\"You have exhausted your daily quota on this model.\",\"code\":1}}";
        let l = detectar(saida).expect("devia reconhecer a cota esgotada");
        assert!(l.evidencia.contains("daily quota"));
    }

    #[test]
    fn reconhece_o_codigo_cru_da_api() {
        let saida = "erro: 429 RESOURCE_EXHAUSTED";
        assert!(detectar(saida).is_some());
    }

    #[test]
    fn nao_promete_um_horario_que_o_cli_nao_disse() {
        // A tela decide se oferece "esperar" olhando este campo. Um horario
        // inventado aqui faria o app dormir ate um instante que ninguem mediu.
        let l = detectar("You have exhausted your daily quota on this model.").unwrap();
        assert_eq!(l.volta_em, None);
    }

    #[test]
    fn conteudo_do_usuario_no_meio_nao_dispara() {
        // O briefing de uma campanha sobre um produto de assinatura fala de
        // cota e limite o tempo todo. Um falso positivo aqui pararia a
        // campanha para perguntar sobre uma cota que nao acabou.
        let briefing = "Objetivo: vender o plano com cota exceeded ilimitada.\n\
                        A peca deve citar o limite de 429 assinantes.\n\
                        Publicar as 21h.";
        let mut texto = briefing.to_string();
        // Vinte linhas de resposta depois, para o trecho sair da janela final.
        for i in 0..20 {
            texto.push_str(&format!("\nlinha de resposta {i}"));
        }
        assert!(detectar(&texto).is_none());
    }
}
