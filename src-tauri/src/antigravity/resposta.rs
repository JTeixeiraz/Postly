//! O que o `agy` respondeu: ler o envelope e traduzir a falha.
//!
//! Separado do `mod.rs` porque e outro momento. La e montar o turno e rodar;
//! aqui e entender o que voltou — e o que voltou tem tres formas diferentes
//! (sucesso, erro declarado, cancelamento silencioso) que so se parecem por
//! fora.

use serde::Deserialize;

/// O envelope do `--output-format json`, medido.
///
/// ```json
/// {"conversation_id":"…","status":"SUCCESS","response":"OK\n",
///  "duration_seconds":2.33,"num_turns":1,
///  "usage":{"input_tokens":13929,"output_tokens":1,"total_tokens":13930}}
/// ```
#[derive(Debug, Deserialize)]
pub(super) struct RespostaCli {
    #[serde(default)]
    pub(super) status: String,
    #[serde(default)]
    pub(super) response: Option<String>,
    /// String, e nao objeto — diferente do Gemini CLI, que trazia `{message,code}`.
    #[serde(default)]
    pub(super) error: Option<String>,
    #[serde(default)]
    pub(super) usage: Option<Uso>,
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
pub(super) struct Uso {
    #[serde(default)]
    pub(super) input_tokens: u64,
    #[serde(default)]
    pub(super) output_tokens: u64,
}

/// Traduz um status que nao e sucesso em algo acionavel.
pub(super) fn explicar(r: &RespostaCli, stderr: &str) -> String {
    let detalhe = r
        .error
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| motivo(stderr));

    match r.status.as_str() {
        // Medido: e o que acontece quando uma ferramenta pede permissao e nao
        // ha ninguem para responder. Sem esta explicacao, o turno morreria com
        // uma resposta vazia e nenhum motivo.
        // A EXPLICACAO DO PROPRIO CLI VENCE A NOSSA, quando ela existe. Medido:
        // ele escreve no stderr algo bem mais util que qualquer texto generico —
        //
        //   a tool required the "read_file" permission that headless mode cannot
        //   prompt for, so it was auto-denied
        //
        // Nossa frase dizia so "pediu uma permissao", sem dizer QUAL. Um relato
        // de defeito com o nome da ferramenta vale muito mais que um sem.
        "CANCELED" | "INTERRUPTED" => {
            let nossa = crate::idioma::msg(
                "O turno foi cancelado porque o agente pediu uma permissao que nao ha como \
                 responder num processo sem terminal. Isto e um defeito do Postly, nao seu — \
                 relate o caso.",
                "The turn was cancelled because the agent asked for a permission that cannot be \
                 answered in a headless process. That is a Postly defect, not yours — please \
                 report it.",
            );
            if detalhe.trim().is_empty() {
                nossa
            } else {
                format!("{nossa} ({detalhe})")
            }
        }
        "INVALID" => format!(
            "{} {detalhe}",
            crate::idioma::msg(
                "O Antigravity recusou a entrada deste turno.",
                "Antigravity rejected this turn's input.",
            )
        ),
        _ => detalhe,
    }
}

/// A sessao do CLI precisa ser refeita a mao?
pub(super) fn precisa_de_login(texto: &str) -> bool {
    const MARCAS: &[&str] = &[
        "not logged in",
        "please sign in",
        "authentication cancelled",
        "opening authentication page",
        "unauthenticated",
    ];
    let t = texto.to_lowercase();
    MARCAS.iter().any(|m| t.contains(m))
}

/// Le o envelope JSON de uma das saidas.
///
/// A busca recomeca na primeira chave quando o texto inteiro nao e JSON: o CLI
/// pode escrever aviso em texto puro antes do envelope, e um `from_str` no
/// texto todo faria o turno virar "resposta vazia" — o pior tipo de erro, o que
/// aponta para o lugar errado.
pub(super) fn achar_envelope(texto: &str) -> Option<RespostaCli> {
    let t = texto.trim();
    if t.is_empty() {
        return None;
    }
    serde_json::from_str(t)
        .ok()
        .or_else(|| serde_json::from_str(&t[t.find('{')?..]).ok())
}

/// O motivo, tirado do stderr, sem o ruido que vem antes dele.
pub(super) fn motivo(stderr: &str) -> String {
    stderr
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with("at ") && !l.starts_with('[') && !l.starts_with('{'))
        .find(|l| {
            let b = l.to_lowercase();
            b.contains("error") || b.contains("erro") || b.contains("failed")
        })
        .or_else(|| {
            stderr
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty() && !l.starts_with("at ") && !l.starts_with('['))
        })
        .unwrap_or("")
        .chars()
        .take(240)
        .collect()
}

#[cfg(test)]
mod testes_envelope {
    use super::*;

    /// Capturado do `agy` 1.1.23 real, turno bem-sucedido.
    const SUCESSO: &str = r#"{"conversation_id":"744921bc","status":"SUCCESS","response":"OK\n","duration_seconds":2.33,"num_turns":1,"usage":{"input_tokens":13929,"output_tokens":1,"thinking_tokens":0,"cache_read_tokens":0,"total_tokens":13930}}"#;

    /// Capturado real: `-p ""` com o texto no stdin.
    const ERRO: &str = r#"{"conversation_id":"","status":"ERROR","response":"","error":"Error: empty prompt. Usage: agy --print \"your prompt here\"","duration_seconds":0.00003,"num_turns":0,"usage":{"input_tokens":0,"output_tokens":0}}"#;

    /// Capturado real: ferramenta pediu permissao num processo sem terminal.
    /// EXIT FOI 0 — e este e o ponto do teste.
    const CANCELADO: &str = r#"{"conversation_id":"613c8c91","status":"CANCELED","response":"","duration_seconds":1.38,"num_turns":1,"usage":{"input_tokens":13933,"output_tokens":39}}"#;

    #[test]
    fn le_o_envelope_de_sucesso_real() {
        let e = achar_envelope(SUCESSO).expect("devia ler");
        assert_eq!(e.status, "SUCCESS");
        assert_eq!(e.response.as_deref(), Some("OK\n"));
        assert_eq!(e.usage.unwrap().input_tokens, 13929);
    }

    #[test]
    fn o_cancelado_nao_passa_por_sucesso() {
        // O CLI devolveu isto com CODIGO DE SAIDA 0. Se o provedor confiasse no
        // codigo, o orquestrador receberia uma peca vazia como se estivesse
        // tudo bem — e a campanha seguiria montando em cima do nada.
        let e = achar_envelope(CANCELADO).expect("devia ler");
        assert_ne!(e.status, "SUCCESS");
        let m = explicar(&e, "");
        assert!(
            m.contains("permiss") || m.contains("permission"),
            "a explicacao nao diz o que aconteceu: {m}"
        );
    }

    #[test]
    fn o_cancelado_carrega_a_explicacao_do_cli_quando_ha_uma() {
        // O stderr real do `agy` diz QUAL ferramenta pediu permissao. Um relato
        // de defeito com o nome da ferramenta vale muito mais que um sem.
        let e = achar_envelope(CANCELADO).unwrap();
        let stderr = "jetski: no output produced — a tool required the \"read_file\" permission \
                      that headless mode cannot prompt for, so it was auto-denied.";
        let m = explicar(&e, stderr);
        assert!(m.contains("read_file"), "a causa concreta sumiu: {m}");
    }

    #[test]
    fn o_erro_usa_a_mensagem_do_proprio_cli() {
        let e = achar_envelope(ERRO).expect("devia ler");
        assert!(explicar(&e, "").contains("empty prompt"));
    }

    #[test]
    fn le_o_envelope_mesmo_com_aviso_em_texto_antes() {
        let saida = format!("aviso solto do CLI\n{SUCESSO}");
        assert_eq!(
            achar_envelope(&saida)
                .expect("devia achar depois do aviso")
                .status,
            "SUCCESS"
        );
    }

    #[test]
    fn o_ruido_do_stderr_nao_vira_o_motivo() {
        // Mesma licao do provedor anterior: banner e diagnostico vem antes do
        // erro de verdade, e pegar a primeira linha aponta para o lugar errado.
        let stderr = "[STARTUP] alguma medicao\nError: a causa de verdade\n    at foo (x.go:1)";
        let m = motivo(stderr);
        assert!(!m.contains("[STARTUP]"), "{m}");
        assert!(m.starts_with("Error: a causa"), "{m}");
    }
}
