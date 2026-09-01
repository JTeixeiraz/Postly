//! Os prompts do fluxo de video.
//!
//! Tres cargos, e nao os cinco da campanha: nao ha rede social para o Diretor
//! Geral dividir, nem legenda para o Criador escrever. O Gerente decide a
//! linha, o Motion Designer monta e o Auditor confere.

use super::assets::Projeto;
use super::spec::TipoCena;

/// O vocabulario de cenas, escrito para o modelo.
///
/// Vive aqui e nao no `spec.rs` porque e texto de prompt, que muda por razoes
/// de redacao; o `spec.rs` muda por razoes de contrato. Mas o `TipoCena` e a
/// fonte: acrescentar um tipo la e esquecer aqui daria um tipo que existe e
/// que o modelo nunca usa, entao a lista e gerada do enum.
fn catalogo_de_cenas() -> String {
    [
        TipoCena::Titulo,
        TipoCena::KenBurns,
        TipoCena::Placa,
        TipoCena::Comparacao,
        TipoCena::Declaracao,
        TipoCena::Fecho,
    ]
    .iter()
    .map(|t| {
        let (slug, o_que) = descrever(*t);
        format!(
            "- `{slug}` — {o_que} Consome {} imagem(ns).",
            t.imagens_necessarias()
        )
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn descrever(t: TipoCena) -> (&'static str, &'static str) {
    match t {
        TipoCena::Titulo => ("titulo", "Titulo sobre fundo limpo. Abre o video."),
        TipoCena::KenBurns => (
            "ken_burns",
            "Uma imagem com movimento lento de camera. E o que faz foto parada parecer video, \
             e deve ser a maioria das cenas de um video feito de fotos.",
        ),
        TipoCena::Placa => ("placa", "Imagem inteira, parada, com legenda embaixo."),
        TipoCena::Comparacao => ("comparacao", "Duas imagens lado a lado, para antes/depois."),
        TipoCena::Declaracao => (
            "declaracao",
            "So texto grande, para o momento em que a frase e o conteudo.",
        ),
        TipoCena::Fecho => ("fecho", "Fecho com chamada para acao."),
    }
}

pub const SYSTEM_GERENTE_PT: &str = "\
Voce e o Gerente de Setor num departamento de marketing. Recebeu um pedido de \
VIDEO avulso — nao e peca de campanha, nao vai para rede social nenhuma, e \
ninguem vai publicar automaticamente. A pessoa vai baixar o arquivo e usar \
como quiser.

Sua entrega e a LINHA do video, em prosa curta: para quem e, que sensacao \
deve deixar, em que ordem as ideias aparecem, e o que NAO entra. Voce nao \
escolhe cenas nem tempos — quem faz isso e o Motion Designer, que le o que \
voce escrever.

Termine com a mensagem que atravessa, entre os marcadores.";

pub const SYSTEM_GERENTE_EN: &str = "\
You are the Sector Manager in a marketing department. You received a request \
for a STANDALONE VIDEO — not a campaign piece, not bound for any social \
network, and nothing will be published automatically. The person will download \
the file and use it however they want.

Your delivery is the video's LINE, in short prose: who it is for, what feeling \
it should leave, in what order the ideas appear, and what stays out. You do \
not pick scenes or timings — the Motion Designer does that, reading what you \
write.

Finish with the message that crosses over, between the markers.";

/// O que o gerente recebe.
pub fn prompt_gerente(objetivo: &str, projeto: &Projeto, com_narracao: bool) -> String {
    let voz = if com_narracao {
        crate::idioma::msg(
            "ESTE VIDEO TEM NARRACAO. A voz carrega o argumento; o texto na tela existe \
             para NAO repetir o que ela diz.",
            "THIS VIDEO HAS NARRATION. The voice carries the argument; on-screen text \
             exists so as NOT to repeat what it says.",
        )
    } else {
        crate::idioma::msg(
            "ESTE VIDEO NAO TEM NARRACAO. O texto na tela e a unica voz, e as cenas \
             precisam durar o tempo de alguem ler.",
            "THIS VIDEO HAS NO NARRATION. On-screen text is the only voice, and scenes \
             must last long enough to read.",
        )
    };

    format!(
        "{}\n{objetivo}\n\n{voz}\n\n{}\n{}\n\n{}\n{}",
        crate::idioma::msg("OBJETIVO DO VIDEO:", "VIDEO GOAL:"),
        crate::idioma::msg("IMAGENS DISPONIVEIS:", "AVAILABLE IMAGES:"),
        lista(&projeto.imagens),
        crate::idioma::msg("AUDIO DISPONIVEL:", "AVAILABLE AUDIO:"),
        lista(&projeto.audio),
    )
}

pub const SYSTEM_MOTION_PT: &str = "\
Voce e o Motion Designer. Recebe a linha do video e o material disponivel, e \
devolve um ROTEIRO EXECUTAVEL em JSON. Nao escreva codigo, nao escreva prosa \
fora do JSON, nao invente tipo de cena.

Regras que nao se negociam:
- `dur_s` e em SEGUNDOS, nunca em quadros. Entre 0.6 e 20.
- `imagens` recebe NOMES DE ARQUIVO da lista que voce recebeu, exatamente como \
  aparecem. Nao invente nome, nao escreva caminho, nao use imagem que nao esta \
  na lista.
- Cada tipo de cena consome um numero fixo de imagens. Respeite.
- Ken Burns deve ser a maioria das cenas quando o material e foto parada: e o \
  que impede o video de parecer uma apresentacao de slides.

VOCE DIRIGE, NAO SO MONTA. Cada cena tem um bloco `direcao`, e e ele que faz \
duas cenas do mesmo tipo nao se parecerem. Um video em que todas as cenas tem a \
mesma direcao e um template, e nao um video seu.
- `movimento`: aproximar, afastar, varrer_esquerda, varrer_direita, subir, \
  descer, nenhum. NAO repita o mesmo movimento em cenas vizinhas. Use `nenhum` \
  quando a cena precisar de silencio visual — um video em que tudo se move nao \
  deixa nada respirar.
- `foco`: centro, topo, base, esquerda, direita. De onde a camera parte. \
  Aproximar do rosto no topo da foto e outro enquadramento que aproximar do \
  centro; escolha olhando o que a imagem tem.
- `pouso`: inferior_esquerda, inferior_direita, superior_esquerda, centro, \
  coluna_esquerda. Onde o texto fica. Ponha o texto onde a imagem estiver mais \
  vazia.
- `entrada`: fade, subir, escala, cortina, corte. Use `corte` quando duas cenas \
  formarem uma batida — corte seco e ritmo, nao falta de transicao.
- `escala_texto`: 0.6 a 1.8. Frase curta e forte pede numero alto; frase longa \
  pede baixo, senao estoura a caixa.

E o video inteiro tem um `look`, escolhido UMA vez:
- `energia`: 0 a 1. Institucional calmo perto de 0.2; corte de rede social \
  perto de 0.8. Isso muda quanto a camera anda e quao rapidas sao as entradas.
- `vinheta`: escurece as bordas. Ajuda texto sobre foto clara.
- `filete`: um tracinho da cor da marca antes dos titulos.

Devolva SOMENTE o JSON, neste formato:
{\"cenas\":[{\"tipo\":\"...\",\"dur_s\":0.0,\"titulo\":\"\",\"subtitulo\":\"\",\
\"imagens\":[],\"narracao\":\"\",\"direcao\":{\"movimento\":\"aproximar\",\
\"foco\":\"centro\",\"pouso\":\"inferior_esquerda\",\"entrada\":\"fade\",\
\"escala_texto\":1.0}}],\"trilha\":\"\",\"proporcao\":\"16:9\",\"racional\":\"\",\
\"look\":{\"energia\":0.5,\"vinheta\":false,\"filete\":false}}";

pub const SYSTEM_MOTION_EN: &str = "\
You are the Motion Designer. You get the video's line and the available \
material, and you return an EXECUTABLE SCRIPT as JSON. Do not write code, do \
not write prose outside the JSON, do not invent scene types.

Non-negotiable rules:
- `dur_s` is in SECONDS, never frames. Between 0.6 and 20.
- `imagens` takes FILE NAMES from the list you were given, exactly as they \
  appear. Do not invent names, do not write paths, do not use an image that is \
  not on the list.
- Each scene type consumes a fixed number of images. Respect it.
- Ken Burns should be most scenes when the material is still photos: it is what \
  keeps the video from looking like a slideshow.

YOU DIRECT, YOU DO NOT ONLY ASSEMBLE. Every scene has a `direcao` block, and it \
is what keeps two scenes of the same type from looking alike. A video where \
every scene shares one direction is a template, not your video.
- `movimento`: aproximar, afastar, varrer_esquerda, varrer_direita, subir, \
  descer, nenhum. DO NOT repeat the same movement on neighbouring scenes. Use \
  `nenhum` when a scene needs visual silence — a video where everything moves \
  lets nothing breathe.
- `foco`: centro, topo, base, esquerda, direita. Where the camera starts. \
  Pushing in on a face at the top of the photo is a different framing from \
  pushing in on the centre; choose by what the image actually holds.
- `pouso`: inferior_esquerda, inferior_direita, superior_esquerda, centro, \
  coluna_esquerda. Where the text sits. Put it where the image is emptiest.
- `entrada`: fade, subir, escala, cortina, corte. Use `corte` when two scenes \
  form a beat — a hard cut is rhythm, not a missing transition.
- `escala_texto`: 0.6 to 1.8. A short punchy line wants a high number; a long \
  one wants low, or it overflows.

And the whole video has one `look`, chosen ONCE:
- `energia`: 0 to 1. Calm corporate near 0.2; social-media cut near 0.8. It \
  drives how far the camera travels and how fast entrances are.
- `vinheta`: darkens the edges. Helps text over a bright photo.
- `filete`: a small brand-coloured rule before titles.

Return ONLY the JSON, in this shape:
{\"cenas\":[{\"tipo\":\"...\",\"dur_s\":0.0,\"titulo\":\"\",\"subtitulo\":\"\",\
\"imagens\":[],\"narracao\":\"\",\"direcao\":{\"movimento\":\"aproximar\",\
\"foco\":\"centro\",\"pouso\":\"inferior_esquerda\",\"entrada\":\"fade\",\
\"escala_texto\":1.0}}],\"trilha\":\"\",\"proporcao\":\"16:9\",\"racional\":\"\",\
\"look\":{\"energia\":0.5,\"vinheta\":false,\"filete\":false}}";

pub fn prompt_motion(
    linha: &str,
    projeto: &Projeto,
    proporcao: &str,
    correcoes: Option<&str>,
) -> String {
    let voz = if projeto.tem_narracao() {
        crate::idioma::msg(
            "HA NARRACAO GRAVADA. Distribua o texto dela pelo campo `narracao` de cada \
             cena, e faca a duracao da cena caber a fala com folga. O `titulo` na tela \
             NAO deve repetir o que a voz diz.",
            "NARRATION IS RECORDED. Spread its text across each scene's `narracao` field, \
             and make each scene long enough to hold the line with room to spare. The \
             on-screen `titulo` must NOT repeat what the voice says.",
        )
    } else {
        crate::idioma::msg(
            "NAO HA NARRACAO. Deixe `narracao` vazio em todas as cenas. O texto na tela \
             e a unica voz: de a cada cena o tempo de alguem ler em voz alta, sem pressa.",
            "THERE IS NO NARRATION. Leave `narracao` empty in every scene. On-screen text \
             is the only voice: give each scene enough time to read it aloud, unhurried.",
        )
    };

    let mut p = format!(
        "{}\n{linha}\n\n{voz}\n\n{}\n{}\n\n{}\n{}\n\n{}\n{}\n\n{} {proporcao}",
        crate::idioma::msg("A LINHA DO VIDEO:", "THE VIDEO'S LINE:"),
        crate::idioma::msg("TIPOS DE CENA DISPONIVEIS:", "AVAILABLE SCENE TYPES:"),
        catalogo_de_cenas(),
        crate::idioma::msg(
            "IMAGENS (use estes nomes, exatamente):",
            "IMAGES (use these names, exactly):"
        ),
        lista(&projeto.imagens),
        crate::idioma::msg(
            "AUDIO para a trilha (use estes nomes, ou deixe vazio):",
            "AUDIO for the track (use these names, or leave empty):"
        ),
        lista(&projeto.audio),
        crate::idioma::msg("PROPORCAO PEDIDA:", "REQUESTED ASPECT RATIO:"),
    );

    if projeto.tem_narracao() {
        p.push_str(&format!(
            "\n\n{}\n{}",
            crate::idioma::msg("ARQUIVOS DE NARRACAO:", "NARRATION FILES:"),
            lista(&projeto.narracao)
        ));
    }

    // As correcoes entram por ultimo, depois de todo o material: elas refinam
    // um roteiro que ja existe, e vir antes faria o modelo ler a critica sem
    // saber do que ela fala.
    if let Some(c) = correcoes {
        p.push_str(&format!(
            "\n\n{}\n{c}",
            crate::idioma::msg(
                "O ROTEIRO ANTERIOR FOI REPROVADO. Corrija:",
                "THE PREVIOUS SCRIPT WAS REJECTED. Fix:"
            )
        ));
    }
    p
}

pub const SYSTEM_AUDITOR_PT: &str = "\
Voce e o Auditor. Recebe a linha do video e o roteiro que o Motion Designer \
montou, e diz se ele entrega o que foi pedido.

Julgue tres coisas, e so estas tres:
1. O roteiro cumpre a linha do gerente, ou foi para outro lugar?
2. O ritmo funciona? Cena curta demais nao da tempo de ler; longa demais deixa \
   a tela parada.
3. Ha repeticao entre o que a voz diz e o que a tela escreve?

Devolva SOMENTE JSON:
{\"aprovado\":true,\"correcoes\":\"\",\"parecer\":\"\"}";

pub const SYSTEM_AUDITOR_EN: &str = "\
You are the Auditor. You get the video's line and the script the Motion \
Designer assembled, and you say whether it delivers what was asked.

Judge three things, and only these three:
1. Does the script fulfil the manager's line, or did it go somewhere else?
2. Does the pacing work? Too short and there is no time to read; too long and \
   the screen sits still.
3. Is there repetition between what the voice says and what the screen writes?

Return ONLY JSON:
{\"aprovado\":true,\"correcoes\":\"\",\"parecer\":\"\"}";

pub fn prompt_auditor(linha: &str, roteiro_json: &str, com_narracao: bool) -> String {
    format!(
        "{}\n{linha}\n\n{}\n{roteiro_json}\n\n{}",
        crate::idioma::msg("A LINHA DO VIDEO:", "THE VIDEO'S LINE:"),
        crate::idioma::msg("O ROTEIRO MONTADO:", "THE ASSEMBLED SCRIPT:"),
        if com_narracao {
            crate::idioma::msg(
                "Este video tem narracao gravada.",
                "This video has recorded narration.",
            )
        } else {
            crate::idioma::msg(
                "Este video nao tem narracao: o texto na tela e a unica voz.",
                "This video has no narration: on-screen text is the only voice.",
            )
        }
    )
}

/// O roteiro de narracao que o Motion Designer escreve quando a pessoa aceita
/// gravar a voz.
///
/// SAI EM TEXTO CORRIDO, e nao em JSON, de proposito: este texto vai ser
/// COLADO no ElevenLabs por uma pessoa. Marcador de cena, chave e colchete
/// seriam lidos em voz alta pelo sintetizador.
pub const SYSTEM_LOCUCAO_PT: &str = "\
Voce e o Motion Designer escrevendo o ROTEIRO DE LOCUCAO de um video.

O texto vai ser colado num sintetizador de voz e lido em voz alta, exatamente \
como voce escrever. Entao:
- Escreva SO o que deve ser falado. Nada de titulo de cena, marcador de tempo, \
  colchete, asterisco ou instrucao de direcao.
- Calibre em 2,6 palavras por segundo. Diga no fim quantas palavras escreveu e \
  a quantos segundos isso corresponde.
- Escreva para o ouvido: frases curtas, sem aposto longo, sem sigla que \
  precise ser soletrada.

Termine com a mensagem que atravessa, entre os marcadores.";

pub const SYSTEM_LOCUCAO_EN: &str = "\
You are the Motion Designer writing a video's VOICEOVER SCRIPT.

The text will be pasted into a speech synthesiser and read aloud exactly as you \
write it. So:
- Write ONLY what should be spoken. No scene titles, no timecodes, no brackets, \
  asterisks or stage directions.
- Calibrate at 2.6 words per second. At the end, say how many words you wrote \
  and how many seconds that is.
- Write for the ear: short sentences, no long asides, no acronym that would need \
  spelling out.

Finish with the message that crosses over, between the markers.";

pub fn prompt_locucao(objetivo: &str, linha: &str, segundos_alvo: u32) -> String {
    format!(
        "{}\n{objetivo}\n\n{}\n{linha}\n\n{}",
        crate::idioma::msg("OBJETIVO DO VIDEO:", "VIDEO GOAL:"),
        crate::idioma::msg("A LINHA DO VIDEO:", "THE VIDEO'S LINE:"),
        crate::idioma::msg(
            &format!("DURACAO ALVO: cerca de {segundos_alvo} segundos de fala."),
            &format!("TARGET LENGTH: about {segundos_alvo} seconds of speech."),
        )
    )
}

/// Nomes de arquivo em lista, ou o aviso de que nao ha nenhum.
///
/// O aviso importa: uma lista vazia sem explicacao faz um modelo pequeno
/// inventar nomes para preencher o campo.
fn lista(itens: &[super::assets::Item]) -> String {
    if itens.is_empty() {
        return crate::idioma::msg(
            "(nenhum — nao cite arquivo nenhum)",
            "(none — do not cite any file)",
        );
    }
    itens
        .iter()
        .map(|i| format!("- {}", i.nome))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn todo_tipo_de_cena_aparece_no_catalogo_do_prompt() {
        // O catalogo e gerado do enum justamente para nao dessincronizar. Se
        // alguem acrescentar um tipo e esquecer o `descrever`, o teste falha
        // aqui em vez de o modelo nunca usar o tipo novo.
        let c = catalogo_de_cenas();
        for t in [
            TipoCena::Titulo,
            TipoCena::KenBurns,
            TipoCena::Placa,
            TipoCena::Comparacao,
            TipoCena::Declaracao,
            TipoCena::Fecho,
        ] {
            let (slug, _) = descrever(t);
            assert!(
                c.contains(slug),
                "o tipo {slug} sumiu do catalogo do prompt"
            );
        }
    }

    #[test]
    fn lista_vazia_diz_para_nao_inventar() {
        // Uma lista vazia sem explicacao faz modelo pequeno preencher o campo
        // com nome inventado, e ai o roteiro so falha na validacao.
        let vazio = lista(&[]);
        assert!(vazio.contains("nenhum") || vazio.contains("none"));
    }

    #[test]
    fn todo_valor_de_direcao_aparece_no_prompt() {
        // Um valor que existe no enum e nao no prompt e um recurso que o modelo
        // nunca usa — a metade do espaco de direcao ficaria morta sem ninguem
        // perceber, e o video voltaria a parecer template.
        for v in [
            "aproximar",
            "afastar",
            "varrer_esquerda",
            "varrer_direita",
            "subir",
            "descer",
            "nenhum",
            "centro",
            "topo",
            "base",
            "esquerda",
            "direita",
            "inferior_esquerda",
            "inferior_direita",
            "superior_esquerda",
            "coluna_esquerda",
            "fade",
            "escala",
            "cortina",
            "corte",
            "energia",
            "vinheta",
            "filete",
        ] {
            assert!(SYSTEM_MOTION_PT.contains(v), "{v} nao esta no prompt PT");
            assert!(SYSTEM_MOTION_EN.contains(v), "{v} nao esta no prompt EN");
        }
    }

    #[test]
    fn o_prompt_manda_variar_a_direcao() {
        // Sem esta instrucao o modelo repete o primeiro valor em todas as cenas,
        // que e exatamente o template que a camada de direcao existe para evitar.
        assert!(SYSTEM_MOTION_PT.contains("NAO repita"));
        assert!(SYSTEM_MOTION_EN.contains("DO NOT repeat"));
    }

    #[test]
    fn o_prompt_de_locucao_proibe_marcador_de_cena() {
        // O texto vai ser colado num sintetizador. "Cena 1" seria lido em voz
        // alta no arquivo final.
        assert!(SYSTEM_LOCUCAO_PT.contains("marcador"));
        assert!(SYSTEM_LOCUCAO_EN.contains("timecodes"));
    }
}
