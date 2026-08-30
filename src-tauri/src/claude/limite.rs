//! Quando a cota do Claude Code acaba no meio de uma campanha.
//!
//! Sem isto, o turno devolveria um erro generico e a campanha morreria com uma
//! mensagem que nao diz o que fazer. Pior: a pessoa saiu da frente do
//! computador justamente porque a campanha demora, e voltaria horas depois
//! para encontrar tudo parado por um motivo que ja tinha passado.
//!
//! O CLI escreve o horario de volta na propria saida. Este modulo o extrai,
//! para que a campanha possa esperar exatamente o necessario em vez de chutar.

use chrono::{Datelike, Duration, Local, TimeZone};
use serde::Serialize;

/// A cota acabou, e quando ela volta.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Limite {
    /// Instante em que a cota volta, em segundos desde a epoca.
    ///
    /// `None` quando a mensagem nao disse o horario — acontece, e nesse caso a
    /// tela oferece so encerrar, sem prometer uma espera que nao sabe medir.
    pub volta_em: Option<i64>,
    /// O trecho que provou o limite. Vai para a tela: um aviso que so diz
    /// "acabou a cota" sem mostrar de onde tirou isso nao da para conferir.
    pub evidencia: String,
}

/// Marcas que so aparecem quando e limite de uso, e nao outro erro qualquer.
///
/// Tiradas do proprio binario do CLI (2.1.251), que carrega as variantes:
/// "Usage limit reached", "…· continuing automatically at …", e a familia
/// "You've hit your … limit". O `credit balance` entra porque a conta pre-paga
/// esgotada chega pelo mesmo caminho e tem a mesma saida para quem usa.
const MARCAS: &[&str] = &[
    "usage limit reached",
    "hit your usage limit",
    "hit your session limit",
    "hit your monthly limit",
    "hit your fast limit",
    "credit balance is too low",
    "credit balance too low",
    "rate_limit_error",
];

/// A saida acusa limite de cota?
///
/// Olha so o FIM do texto. A mensagem de limite e a ultima coisa que o CLI
/// escreve, e um briefing de campanha pode conter as palavras "limite" e
/// "cota" no meio — procurar no texto inteiro transformaria conteudo do
/// usuario em falso positivo.
pub fn detectar(texto: &str) -> Option<Limite> {
    let fim = ultimas_linhas(texto, 15);
    let minusc = fim.to_lowercase();
    if !MARCAS.iter().any(|m| minusc.contains(m)) {
        return None;
    }
    Some(Limite {
        volta_em: quando_volta(&fim),
        evidencia: fim.trim().to_string(),
    })
}

/// As ultimas `n` linhas nao vazias.
fn ultimas_linhas(texto: &str, n: usize) -> String {
    let linhas: Vec<&str> = texto.lines().filter(|l| !l.trim().is_empty()).collect();
    let corte = linhas.len().saturating_sub(n);
    linhas[corte..].join("\n")
}

/// Extrai o instante em que a cota volta.
///
/// Duas formas, nesta ordem de confianca:
///
///   1. um timestamp explicito (`resetsAt: 1767225000`) — nao tem ambiguidade
///   2. um horario de relogio (`resets 9:10pm`, `resets at 21:10`)
///
/// O horario de relogio vem no fuso que o CLI ja converteu para quem le, entao
/// e interpretado como hora local. Se ele ja passou hoje, e de amanha: uma
/// mensagem as 23h dizendo "resets 1:10am" fala do dia seguinte.
fn quando_volta(texto: &str) -> Option<i64> {
    if let Some(ts) = timestamp_explicito(texto) {
        return Some(ts);
    }
    let (h, m) = horario_de_relogio(texto)?;
    let agora = Local::now();
    let hoje = Local
        .with_ymd_and_hms(agora.year(), agora.month(), agora.day(), h, m, 0)
        .single()?;
    let alvo = if hoje > agora {
        hoje
    } else {
        hoje + Duration::days(1)
    };
    Some(alvo.timestamp())
}

/// `resetsAt: 1767225000` ou `"resetsAt":1767225000`.
fn timestamp_explicito(texto: &str) -> Option<i64> {
    let pos = texto.find("resetsAt")?;
    let resto = &texto[pos + "resetsAt".len()..];
    let digitos: String = resto
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let n: i64 = digitos.parse().ok()?;
    // O CLI guarda em segundos, mas um valor em milissegundos aqui viraria uma
    // espera de milenios. O corte e o ano 3000 em segundos.
    let seg = if n > 32_503_680_000 { n / 1000 } else { n };
    // So aceita futuro plausivel: ate 48h a frente. Um timestamp velho colado
    // de outra parte da saida faria a campanha "esperar" um instante passado.
    let agora = Local::now().timestamp();
    (seg > agora && seg < agora + 48 * 3600).then_some(seg)
}

/// `resets 9:10pm`, `resets at 21:10`, `continuing automatically at 3am`.
fn horario_de_relogio(texto: &str) -> Option<(u32, u32)> {
    let t = texto.to_lowercase();
    // Ancora no verbo: sem isso, qualquer hora escrita no texto (um horario de
    // publicacao no briefing, por exemplo) seria lida como hora de reset.
    let ancora = [
        "resets at ",
        "resets ",
        "automatically at ",
        "available at ",
    ]
    .iter()
    .find_map(|a| t.find(a).map(|i| i + a.len()))?;
    let bruto: String = t[ancora..].chars().take(20).collect();
    let bruto = bruto.trim_start().to_string();

    let horas: String = bruto.chars().take_while(|c| c.is_ascii_digit()).collect();
    if horas.is_empty() || horas.len() > 2 {
        return None;
    }
    let mut h: u32 = horas.parse().ok()?;

    let apos = &bruto[horas.len()..];
    let minutos: u32 = if let Some(sem_dois) = apos.strip_prefix(':') {
        let m: String = sem_dois
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if m.len() != 2 {
            return None;
        }
        m.parse().ok()?
    } else {
        0
    };

    // `pm` soma 12, exceto ao meio-dia; `am` zera a meia-noite.
    let sufixo = apos.trim_start_matches(|c: char| c.is_ascii_digit() || c == ':');
    let sufixo = sufixo.trim_start();
    if sufixo.starts_with("pm") {
        if h > 12 {
            return None;
        }
        if h != 12 {
            h += 12;
        }
    } else if sufixo.starts_with("am") {
        if h > 12 {
            return None;
        }
        if h == 12 {
            h = 0;
        }
    }

    (h < 24 && minutos < 60).then_some((h, minutos))
}

// ─────────────────────────────────────────────────────────── a pausa e a espera

use tauri::{AppHandle, Emitter};

/// Avisa que a cota acabou e espera a decisao de quem usa.
///
/// Devolve `true` quando a campanha deve continuar — o que so acontece depois
/// de a cota realmente voltar, porque a espera acontece aqui dentro.
///
/// Encerrar e o caminho padrao em toda saida que nao seja um "esperar"
/// explicito: janela fechada, tempo esgotado, canal perdido. Uma campanha que
/// se prende sozinha esperando por ninguem e pior que uma que para e diz por
/// que — os turnos ja rodados ficam gravados de qualquer jeito.
pub async fn pausar_e_esperar(app: &AppHandle, state: &crate::state::AppState, l: &Limite) -> bool {
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    {
        let Ok(mut vaga) = state.resposta_limite.lock() else {
            return false;
        };
        *vaga = Some(tx);
    }

    // A notificacao do sistema e o que faz este aviso valer: a pessoa saiu da
    // frente do computador justamente porque a campanha demora.
    if let Some(step) = crate::platform::current().notify_step(
        &crate::idioma::msg("Postly: a cota acabou", "Postly: quota ran out"),
        &crate::idioma::msg(
            "O Claude Code atingiu o limite. Volte ao app para decidir.",
            "Claude Code hit its limit. Return to the app to decide.",
        ),
    ) {
        std::thread::spawn(move || {
            let _ = step.run();
        });
    }

    let _ = app.emit("postly://limite", l);

    // Sem teto de espera: aqui a pessoa pode estar longe do computador por
    // horas, que e exatamente o cenario. O que impede o travamento eterno e a
    // janela — fechando o app, a campanha morre com ele.
    let Ok(Ok(esperar)) = rx.await.map(Ok::<bool, ()>) else {
        if let Ok(mut vaga) = state.resposta_limite.lock() {
            *vaga = None;
        }
        return false;
    };
    if !esperar {
        return false;
    }

    let Some(volta) = l.volta_em else {
        // Escolheu esperar, mas o CLI nao disse ate quando. Prometer uma
        // espera cega seria pior que admitir que nao da: a tela nem oferece
        // esta opcao nesse caso, entao chegar aqui e defeito.
        return false;
    };

    let faltam = (volta - chrono::Local::now().timestamp()).max(0) as u64;
    // Um minuto a mais: o horario do CLI e arredondado, e voltar cedo demais
    // gasta um turno para receber o mesmo erro.
    let dormir = faltam + 60;
    let _ = app.emit(
        "postly://limite-esperando",
        serde_json::json!({ "volta_em": volta, "segundos": dormir }),
    );

    // A vaga e reaberta durante o sono. Sem isso, o botao de encerrar que a
    // tela mostra enquanto a campanha espera nao teria para onde mandar o
    // clique — e um botao que nao faz nada e pior que a ausencia dele.
    let (tx_cancel, rx_cancel) = tokio::sync::oneshot::channel::<bool>();
    {
        let Ok(mut vaga) = state.resposta_limite.lock() else {
            return false;
        };
        *vaga = Some(tx_cancel);
    }

    let seguiu = tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_secs(dormir)) => true,
        _ = rx_cancel => false,
    };
    if let Ok(mut vaga) = state.resposta_limite.lock() {
        *vaga = None;
    }
    if !seguiu {
        let _ = app.emit("postly://limite-fim", ());
        return false;
    }

    if let Some(step) = crate::platform::current().notify_step(
        &crate::idioma::msg("Postly: a campanha voltou", "Postly: the campaign resumed"),
        &crate::idioma::msg(
            "A cota do Claude Code voltou e a campanha seguiu sozinha.",
            "The Claude Code quota is back and the campaign resumed on its own.",
        ),
    ) {
        std::thread::spawn(move || {
            let _ = step.run();
        });
    }
    let _ = app.emit("postly://limite-fim", ());
    true
}

#[cfg(test)]
mod testes {
    use super::*;

    fn hm(ts: i64) -> (u32, u32) {
        use chrono::Timelike;
        let d = Local.timestamp_opt(ts, 0).single().unwrap();
        (d.hour(), d.minute())
    }

    #[test]
    fn reconhece_as_mensagens_reais_do_cli() {
        // Colhidas do binario 2.1.251 e da saida do proprio CLI.
        for m in [
            "You've hit your session limit · resets 9:10pm (America/Sao_Paulo)",
            "Usage limit reached · continuing automatically at 3am",
            "Claude AI usage limit reached",
            "You've hit your monthly limit — raise it below, or it resets next month.",
            "credit balance is too low",
        ] {
            assert!(detectar(m).is_some(), "nao detectou: {m}");
        }
    }

    #[test]
    fn nao_confunde_com_outro_erro_nem_com_o_briefing() {
        for m in [
            "falha ao iniciar o Claude Code: No such file or directory",
            "O Claude Code passou do tempo limite deste turno.",
            // O texto de uma campanha pode falar de limite e de horario sem
            // que isso tenha qualquer relacao com cota.
            "Publicar as 9:10pm. O limite de caracteres do X e 280.",
            "",
        ] {
            assert!(detectar(m).is_none(), "falso positivo: {m}");
        }
    }

    #[test]
    fn le_o_horario_em_12h_e_em_24h() {
        for (texto, esperado) in [
            ("Usage limit reached · resets 9:10pm", (21, 10)),
            ("usage limit reached · resets at 21:10", (21, 10)),
            (
                "usage limit reached · continuing automatically at 3am",
                (3, 0),
            ),
            ("usage limit reached · resets 12pm", (12, 0)),
            ("usage limit reached · resets 12am", (0, 0)),
            (
                "usage limit reached · resets 7:05am (America/Sao_Paulo)",
                (7, 5),
            ),
        ] {
            let l = detectar(texto).expect(texto);
            let ts = l.volta_em.unwrap_or_else(|| panic!("sem horario: {texto}"));
            assert_eq!(hm(ts), esperado, "texto: {texto}");
        }
    }

    #[test]
    fn horario_que_ja_passou_e_de_amanha() {
        use chrono::Timelike;
        let agora = Local::now();
        // Uma hora atras, no relogio de 24h.
        let passado = (agora.hour() + 23) % 24;
        let texto = format!(
            "usage limit reached · resets at {passado:02}:{:02}",
            agora.minute()
        );
        let ts = detectar(&texto).unwrap().volta_em.unwrap();
        let faltam = ts - agora.timestamp();
        assert!(
            faltam > 22 * 3600 && faltam < 24 * 3600,
            "esperava ~23h de espera, deu {}h",
            faltam / 3600
        );
    }

    #[test]
    fn sem_horario_a_espera_nao_e_prometida() {
        let l = detectar("Claude AI usage limit reached").unwrap();
        assert_eq!(l.volta_em, None);
    }

    #[test]
    fn prefere_o_timestamp_ao_relogio_quando_os_dois_aparecem() {
        let daqui_a_uma_hora = Local::now().timestamp() + 3600;
        let texto =
            format!("usage limit reached · resets 9:10pm\n{{\"resetsAt\":{daqui_a_uma_hora}}}");
        assert_eq!(detectar(&texto).unwrap().volta_em, Some(daqui_a_uma_hora));
    }

    #[test]
    fn timestamp_no_passado_ou_absurdo_e_ignorado() {
        // Um valor velho colado de outra parte da saida faria a campanha
        // "esperar" um instante que ja passou, e ela seguiria na hora.
        for ts in [1_600_000_000_i64, Local::now().timestamp() + 90 * 3600] {
            let texto = format!("usage limit reached\n{{\"resetsAt\":{ts}}}");
            assert_eq!(detectar(&texto).unwrap().volta_em, None, "ts: {ts}");
        }
    }

    #[test]
    fn le_so_o_fim_da_saida() {
        // O limite chega na ultima linha; o comeco e a resposta do modelo.
        let mut texto = String::new();
        for i in 0..80 {
            texto.push_str(&format!("linha {i} da peca\n"));
        }
        texto.push_str("Usage limit reached · resets 9:10pm\n");
        let l = detectar(&texto).unwrap();
        assert!(l.evidencia.lines().count() <= 15);
        assert!(l.evidencia.contains("Usage limit reached"));
    }
}
