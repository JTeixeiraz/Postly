//! Idioma das mensagens que o backend devolve para a tela.
//!
//! O dicionario do frontend cobre a interface, mas nao alcanca o que sobe
//! daqui: erro de campanha, aviso de memoria, recusa do Gemini. Trocar para
//! ingles deixava metade da tela em portugues, e essa metade era justamente a
//! que aparece quando algo da errado.
//!
//! A escolha vive num atomico global em vez de viajar por parametro em cada
//! assinatura: ela muda uma vez por sessao, e enfiar `idioma` em quarenta
//! funcoes so para carregar um enum seria pior de ler e de manter.

use std::sync::atomic::{AtomicBool, Ordering};

static EM_INGLES: AtomicBool = AtomicBool::new(false);

pub fn definir(idioma: &str) {
    EM_INGLES.store(idioma.eq_ignore_ascii_case("en"), Ordering::Relaxed);
}

pub fn em_ingles() -> bool {
    EM_INGLES.load(Ordering::Relaxed)
}

/// Escolhe entre as duas versoes do mesmo texto.
///
/// Uso: `msg("Sem memoria.", "Out of memory.")`. Manter as duas lado a lado no
/// ponto da chamada e o que impede uma delas de envelhecer sozinha, que e o
/// destino de todo arquivo de traducao separado.
pub fn msg(pt: &str, en: &str) -> String {
    if em_ingles() {
        en.to_string()
    } else {
        pt.to_string()
    }
}
