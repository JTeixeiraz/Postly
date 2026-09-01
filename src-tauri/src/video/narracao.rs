//! A pergunta sobre narracao: quando ela acontece, e o que a pessoa recebe.
//!
//! O PEDIDO E ESPECIFICO E A ORDEM IMPORTA. O modelo nao adivinha se ha voz —
//! ele OLHA a pasta `narracao/`. Se ha arquivo la, ha narracao e ninguem
//! pergunta nada. Se nao ha, a campanha PARA e pergunta, uma vez, antes de o
//! Motion Designer montar a primeira cena.
//!
//! Por que exatamente ali, e nao antes nem depois:
//!
//!   - ANTES DA LINHA DO GERENTE: a pergunta chegaria sem contexto. "Voce quer
//!     narracao?" e uma pergunta melhor de responder depois de ler para onde o
//!     video vai.
//!   - DEPOIS DO ROTEIRO: tarde demais. As cenas ja teriam sido medidas para
//!     texto na tela, e acrescentar voz depois exigiria remontar tudo — o
//!     mesmo erro que animar uma peca antes de ela ser aprovada.
//!
//! E POR QUE A CAMPANHA PARA DE VERDADE. O Rust dorme num canal esperando a
//! resposta, como no turno de movimento. A notificacao do sistema sai junto,
//! pela mesma razao medida la: quem espera minutos por um turno nao fica
//! olhando a janela, e um modal que passa despercebido segura tudo ate o tempo
//! estourar.

use tauri::{AppHandle, Emitter};

use crate::platform;

/// Onde a pessoa gera a voz.
///
/// Um link so, e o do ElevenLabs, porque foi o que o pedido nomeou. Uma lista
/// de servicos aqui viraria uma decisao a mais no meio de um fluxo que ja
/// parou para perguntar uma coisa.
pub const ELEVENLABS: &str = "https://elevenlabs.io/text-to-speech";

/// O que a tela recebe quando o video para para perguntar sobre a voz.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PedidoNarracao {
    /// A linha que o gerente decidiu. A pergunta sem isso seria abstrata.
    pub linha: String,
    /// Onde os arquivos de voz devem ser largados depois de gerados.
    ///
    /// Vai para a tela como caminho absoluto de propósito: a pessoa vai sair
    /// do app, gerar o audio noutro site e voltar com um arquivo na mao. "Pasta
    /// de narracao" nao diz onde ela fica.
    pub pasta: String,
    pub elevenlabs: String,
}

/// A resposta da pessoa.
///
/// Tres estados e nao dois, porque "nao quero voz" e "quero, me da o roteiro"
/// levam o video para lugares diferentes — e fechar a janela nao pode ser lido
/// como nenhum dos dois.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RespostaNarracao {
    /// Segue sem voz: o texto na tela e a unica narracao.
    SemVoz,
    /// Escreve o roteiro de locucao e entrega junto do link do ElevenLabs.
    QueroRoteiro,
}

/// Pergunta, e espera.
///
/// Devolve `SemVoz` em todo caminho que nao seja um pedido explicito de
/// roteiro: recusa, janela fechada, tempo esgotado. Seguir sem voz e o unico
/// padrao seguro — um video sem narracao e um video; um video montado para uma
/// voz que nunca chegou e uma sequencia de cenas com buraco de silencio.
pub async fn perguntar(
    app: &AppHandle,
    state: &crate::state::AppState,
    linha: &str,
    pasta: &std::path::Path,
) -> RespostaNarracao {
    let (tx, rx) = tokio::sync::oneshot::channel::<RespostaNarracao>();
    {
        let Ok(mut vaga) = state.resposta_narracao.lock() else {
            return RespostaNarracao::SemVoz;
        };
        *vaga = Some(tx);
    }

    if let Some(step) = platform::current().notify_step(
        &crate::idioma::msg("Postly: decisao pendente", "Postly: decision pending"),
        &crate::idioma::msg(
            "O video pode ter narracao. Volte ao app para decidir.",
            "The video can have narration. Return to the app to decide.",
        ),
    ) {
        // Notificar nunca pode derrubar o trabalho nem faze-lo esperar.
        std::thread::spawn(move || {
            let _ = step.run();
        });
    }

    let _ = app.emit(
        "postly://narracao",
        PedidoNarracao {
            linha: linha.to_string(),
            pasta: pasta.to_string_lossy().to_string(),
            elevenlabs: ELEVENLABS.to_string(),
        },
    );

    // Teto de espera, pela mesma razao do turno de movimento: sem ele uma
    // janela fechada deixaria o video parado para sempre segurando a pasta da
    // execucao.
    match tokio::time::timeout(std::time::Duration::from_secs(600), rx).await {
        Ok(Ok(r)) => r,
        _ => {
            if let Ok(mut vaga) = state.resposta_narracao.lock() {
                *vaga = None;
            }
            RespostaNarracao::SemVoz
        }
    }
}

/// O que a tela mostra depois de o cargo escrever o roteiro de locucao.
///
/// Os tres campos andam juntos porque a pessoa precisa dos tres na mesma tela:
/// o texto para copiar, o site para colar, e a pasta para largar o arquivo que
/// voltar. Faltando qualquer um, ela sai do app e nao sabe voltar.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoteiroDeLocucao {
    pub texto: String,
    pub elevenlabs: String,
    pub pasta: String,
    /// Palavras contadas aqui, e nao pelo modelo.
    ///
    /// O prompt PEDE que ele diga a contagem, e ele costuma dizer — errado. A
    /// tela mostra a contagem real: um numero medido vale mais que um numero
    /// afirmado, e este e barato de medir.
    pub palavras: usize,
    /// Segundos estimados a 2,6 palavras por segundo, a mesma calibragem que o
    /// video de apresentacao do proprio Postly usou.
    pub segundos_estimados: f32,
}

/// Palavras por segundo de locucao.
///
/// Medido no video de apresentacao do Postly: 81 palavras couberam nas janelas
/// de cena calibradas a esta taxa, com folga minima de 0,36s por cena.
const PALAVRAS_POR_SEGUNDO: f32 = 2.6;

impl RoteiroDeLocucao {
    pub fn novo(texto: String, pasta: &std::path::Path) -> Self {
        let palavras = texto.split_whitespace().count();
        Self {
            segundos_estimados: palavras as f32 / PALAVRAS_POR_SEGUNDO,
            palavras,
            texto,
            elevenlabs: ELEVENLABS.to_string(),
            pasta: pasta.to_string_lossy().to_string(),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::path::Path;

    #[test]
    fn a_contagem_de_palavras_e_medida_e_nao_perguntada() {
        // O prompt pede ao modelo que diga quantas palavras escreveu, e ele
        // costuma errar. A tela mostra o numero real.
        let r = RoteiroDeLocucao::novo("uma duas tres quatro cinco".into(), Path::new("/tmp"));
        assert_eq!(r.palavras, 5);
    }

    #[test]
    fn a_estimativa_usa_a_taxa_calibrada_do_video_do_proprio_postly() {
        // 130 palavras a 2,6 p/s = 50s. Se alguem mexer na constante sem
        // motivo, este teste diz que o numero tinha origem.
        let texto = vec!["palavra"; 130].join(" ");
        let r = RoteiroDeLocucao::novo(texto, Path::new("/tmp"));
        assert!((r.segundos_estimados - 50.0).abs() < 0.01);
    }

    #[test]
    fn quebra_de_linha_nao_conta_como_palavra() {
        let r = RoteiroDeLocucao::novo("uma\n\n  duas \n tres ".into(), Path::new("/tmp"));
        assert_eq!(r.palavras, 3);
    }
}
