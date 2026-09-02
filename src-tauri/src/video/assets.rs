//! Os assets de um video: onde ficam, e o que a pasta de cada um significa.
//!
//! A estrutura repete a licao da galeria de produto — o diretorio nao e
//! organizacao por gosto, e a REGRA DO PRODUTO VISIVEL NO DISCO:
//!
//! ```text
//! videos/lancamento-outono/
//! ├── .nome        o nome digitado, com acento e maiuscula
//! ├── imagens/     entram na tela do video
//! ├── clipes/      VIDEO BRUTO da pessoa, para o modelo cortar e montar
//! ├── audio/       trilha e efeitos
//! ├── narracao/    A VOZ. E so olhar aqui para saber se ha narracao.
//! └── saida/       os .mp4 renderizados
//! ```
//!
//! POR QUE A NARRACAO TEM PASTA PROPRIA. O motion designer precisa saber se o
//! video tem voz antes de escrever a primeira cena, porque a resposta muda
//! tudo: com narracao, a duracao de cada cena e a duracao da fala mais um
//! respiro, e o texto na tela existe para nao repetir o que a voz ja disse;
//! sem narracao, o texto na tela E a narracao e as cenas precisam durar o
//! tempo de leitura.
//!
//! Deixar o modelo adivinhar isso por nome de arquivo seria adivinhacao com
//! aparencia de deteccao: `audio-final-2.mp3` pode ser trilha ou locucao. Uma
//! pasta responde sem ambiguidade, e quem arrasta o arquivo para dentro dela
//! ja declarou o que ele e.
//!
//! As subpastas nascem juntas e nao sob demanda, pela mesma razao da galeria:
//! uma pasta de narracao que so aparece depois de alguem procurar por ela nao
//! ensina que o lugar existe.
//!
//! A DURACAO DOS AUDIOS NAO E MEDIDA AQUI. Medir exige decodificar o arquivo,
//! e trazer uma dependencia de audio para o Rust so para isso seria peso
//! permanente por conveniencia de um numero. O sidecar ja precisa abrir os
//! arquivos para renderizar — a duracao sai de la, medida, e volta junto do
//! resultado.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// As subpastas, e o que cada uma aceita.
///
/// A lista de extensoes e por pasta e nao global: um `.mp3` solto em
/// `imagens/` nunca vai virar quadro, e aceitar ali so adiaria a falha para o
/// render, que e onde ela custa mais caro para diagnosticar.
const PASTAS: &[(&str, &[&str])] = &[
    ("imagens", &["png", "jpg", "jpeg", "webp"]),
    ("clipes", &["mp4", "mov", "webm", "mkv", "m4v"]),
    ("audio", &["mp3", "wav", "m4a", "ogg"]),
    ("narracao", &["mp3", "wav", "m4a", "ogg"]),
];

/// Onde o render grava. Fora do `PASTAS` de proposito: e saida, nao entrada, e
/// nada do que a pessoa sobe vai para la.
const SAIDA: &str = "saida";

pub fn raiz() -> PathBuf {
    crate::platform::current().data_dir().join("videos")
}

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub nome: String,
    pub caminho: String,
    pub bytes: u64,
}

/// Um projeto de video, com tudo que ele tem em disco.
#[derive(Debug, Clone, Serialize)]
pub struct Projeto {
    /// Nome de diretorio: e a identidade em disco.
    pub slug: String,
    /// O que a pessoa digitou. Pode ter acento, espaco e maiuscula.
    pub nome: String,
    pub caminho: String,
    pub imagens: Vec<Item>,
    /// Video bruto que a pessoa gravou. O modelo corta e monta.
    pub clipes: Vec<Item>,
    pub audio: Vec<Item>,
    /// A voz. A existencia desta lista e o que responde `tem_narracao`.
    pub narracao: Vec<Item>,
    /// Os .mp4 ja renderizados, do mais novo para o mais velho.
    pub saidas: Vec<Item>,
    pub bytes: u64,
}

impl Projeto {
    /// Ha narracao neste projeto?
    ///
    /// E um metodo e nao um campo calculado na tela porque tres lugares fazem
    /// a mesma pergunta — o prompt do gerente, a pausa que pergunta a pessoa e
    /// o render — e a resposta precisa ser a mesma nos tres. Um deles
    /// discordando dos outros produziria a pior falha possivel: um video
    /// renderizado com cenas medidas para uma voz que nao existe.
    pub fn tem_narracao(&self) -> bool {
        !self.narracao.is_empty()
    }

    /// Este video ja tem voz, venha ela de onde vier?
    ///
    /// A PERGUNTA SOBRE NARRACAO NAO PODE IGNORAR OS CLIPES. Quando a pessoa
    /// sobe um video em que ela fala, a voz do video E a narracao — parar o
    /// trabalho para perguntar "voce quer narracao?" seria perguntar sobre algo
    /// que ja esta ali, e aceitar produziria duas vozes por cima uma da outra.
    ///
    /// Clipe sem audio (b-roll) nao conta: ele nao traz voz nenhuma.
    pub fn tem_voz(&self, clipes: &[super::analise::Clipe]) -> bool {
        self.tem_narracao() || clipes.iter().any(|c| c.tem_audio && !c.com_som.is_empty())
    }

    pub fn caminho_de(&self, sub: &str) -> PathBuf {
        PathBuf::from(&self.caminho).join(sub)
    }
}

/// Cria o projeto, com as quatro subpastas dentro.
pub fn criar(nome: &str) -> Result<Projeto, String> {
    // Reusa o slug da galeria em vez de escrever outro: ele ja tem os tres
    // testes que impedem um "../" digitado no campo de escrever fora da pasta,
    // e uma segunda implementacao teria que ganhar os mesmos testes para valer
    // o mesmo — ou seria a que esquece um deles.
    let slug = crate::galeria::slugificar(nome);
    if slug.is_empty() {
        return Err(crate::idioma::msg(
            "De um nome ao video.",
            "Give the video a name.",
        ));
    }
    let dir = raiz().join(&slug);
    if dir.exists() {
        return Err(crate::idioma::msg(
            "Ja existe um video com esse nome.",
            "A video with that name already exists.",
        ));
    }
    for (sub, _) in PASTAS {
        std::fs::create_dir_all(dir.join(sub)).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(dir.join(SAIDA)).map_err(|e| e.to_string())?;
    let _ = std::fs::write(dir.join(".nome"), nome.trim());
    ler(&slug).ok_or_else(|| "o projeto sumiu logo depois de criado".to_string())
}

pub fn listar() -> Vec<Projeto> {
    let Ok(dir) = std::fs::read_dir(raiz()) else {
        return Vec::new();
    };
    let mut projetos: Vec<Projeto> = dir
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| ler(&e.file_name().to_string_lossy()))
        .collect();
    projetos.sort_by_key(|p| p.nome.to_lowercase());
    projetos
}

pub fn ler(slug: &str) -> Option<Projeto> {
    let slug = crate::galeria::slugificar(slug);
    let dir = raiz().join(&slug);
    if !dir.is_dir() {
        return None;
    }
    let nome = std::fs::read_to_string(dir.join(".nome"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| slug.clone());

    let imagens = arquivos_de(&dir.join("imagens"), extensoes("imagens"));
    let clipes = arquivos_de(&dir.join("clipes"), extensoes("clipes"));
    let audio = arquivos_de(&dir.join("audio"), extensoes("audio"));
    let narracao = arquivos_de(&dir.join("narracao"), extensoes("narracao"));
    let mut saidas = arquivos_de(&dir.join(SAIDA), &["mp4"]);
    // Do mais novo para o mais velho: quem acabou de renderizar quer o de cima.
    saidas.reverse();

    let bytes = imagens
        .iter()
        .chain(&clipes)
        .chain(&audio)
        .chain(&narracao)
        .chain(&saidas)
        .map(|i| i.bytes)
        .sum();

    Some(Projeto {
        slug,
        nome,
        caminho: dir.to_string_lossy().to_string(),
        imagens,
        clipes,
        audio,
        narracao,
        saidas,
        bytes,
    })
}

fn extensoes(sub: &str) -> &'static [&'static str] {
    PASTAS
        .iter()
        .find(|(nome, _)| *nome == sub)
        .map(|(_, exts)| *exts)
        .unwrap_or(&[])
}

fn arquivos_de(dir: &Path, exts: &[&str]) -> Vec<Item> {
    let Ok(entradas) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut itens: Vec<Item> = entradas
        .flatten()
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| exts.contains(&x.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .map(|e| Item {
            nome: e.file_name().to_string_lossy().to_string(),
            caminho: e.path().to_string_lossy().to_string(),
            bytes: e.metadata().map(|m| m.len()).unwrap_or(0),
        })
        .collect();
    itens.sort_by(|a, b| a.nome.cmp(&b.nome));
    itens
}

/// Grava um arquivo enviado pela tela dentro de uma das subpastas.
///
/// Recebe base64 pela mesma razao da galeria: sem o plugin de dialogo do Tauri
/// o navegador nao entrega o caminho real do arquivo, e trazer o plugin so
/// para isto seria peso permanente por conveniencia de uma tela.
pub fn adicionar(slug: &str, sub: &str, nome: &str, base64_dados: &str) -> Result<Item, String> {
    use base64::Engine;

    let exts = extensoes(sub);
    if exts.is_empty() {
        return Err(format!("pasta desconhecida: {sub}"));
    }

    let slug = crate::galeria::slugificar(slug);
    let destino_dir = raiz().join(&slug).join(sub);
    if !destino_dir.is_dir() {
        return Err(crate::idioma::msg(
            "Projeto de video nao encontrado.",
            "Video project not found.",
        ));
    }

    let ext = Path::new(nome)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .filter(|e| exts.contains(&e.as_str()))
        .ok_or_else(|| {
            // A mensagem cita as extensoes DAQUELA pasta, e nao uma lista
            // geral: quem largou um mp3 em `imagens/` precisa saber que o
            // problema e a pasta, nao o arquivo.
            crate::idioma::msg(
                &format!("Esta pasta aceita: {}.", exts.join(", ")),
                &format!("This folder accepts: {}.", exts.join(", ")),
            )
        })?;

    // O navegador manda `data:audio/mpeg;base64,AAAA...`; so a cauda interessa.
    let cru = base64_dados
        .split_once(',')
        .map(|(_, resto)| resto)
        .unwrap_or(base64_dados);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(cru.trim())
        .map_err(|e| format!("arquivo invalido: {e}"))?;

    // Teto maior que o das imagens da galeria (6 MB) porque audio de locucao de
    // um minuto ja passa disso. Continua havendo teto: o arquivo atravessa a
    // ponte IPC como texto base64, e um WAV de 200 MB viraria 270 MB de string
    // na memoria da janela.
    const MAX: usize = 60 * 1024 * 1024;
    if bytes.len() > MAX {
        return Err(crate::idioma::msg(
            &format!(
                "O limite e 60 MB por arquivo, e este tem {:.1} MB.",
                bytes.len() as f64 / (1024.0 * 1024.0)
            ),
            &format!(
                "The limit is 60 MB per file, and this one is {:.1} MB.",
                bytes.len() as f64 / (1024.0 * 1024.0)
            ),
        ));
    }

    let base = Path::new(nome)
        .file_stem()
        .map(|s| crate::galeria::slugificar(&s.to_string_lossy()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "arquivo".into());
    // Nome livre: dois arquivos de mesmo nome nao podem se sobrescrever em
    // silencio. Numa pasta de narracao isso seria pior que na galeria — a
    // segunda tomada apagaria a primeira sem ninguem ver.
    let mut destino = destino_dir.join(format!("{base}.{ext}"));
    let mut n = 2;
    while destino.exists() {
        destino = destino_dir.join(format!("{base}-{n}.{ext}"));
        n += 1;
    }
    std::fs::write(&destino, &bytes).map_err(|e| format!("nao consegui gravar: {e}"))?;

    Ok(Item {
        nome: destino
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        caminho: destino.to_string_lossy().to_string(),
        bytes: bytes.len() as u64,
    })
}

/// Copia um arquivo que já está no disco da pessoa para dentro do projeto.
///
/// POR CAMINHO, E NÃO POR BASE64, e a diferença não é gosto. Imagem e áudio
/// chegam pela ponte IPC como texto base64 porque são pequenos; um vídeo de
/// meio giga viraria 660 MB de string na memória da janela e outro tanto no
/// Rust, e o app morreria antes de gravar o primeiro byte.
///
/// O caminho real vem do evento de arrastar-e-soltar do Tauri, que entrega
/// `paths` de verdade — sem plugin novo, que é o que fez a galeria escolher
/// base64 no começo.
pub fn adicionar_por_caminho(slug: &str, sub: &str, origem: &str) -> Result<Item, String> {
    let exts = extensoes(sub);
    if exts.is_empty() {
        return Err(format!("pasta desconhecida: {sub}"));
    }

    let origem = Path::new(origem);
    let ext = origem
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .filter(|e| exts.contains(&e.as_str()))
        .ok_or_else(|| {
            crate::idioma::msg(
                &format!("Esta pasta aceita: {}.", exts.join(", ")),
                &format!("This folder accepts: {}.", exts.join(", ")),
            )
        })?;

    let slug = crate::galeria::slugificar(slug);
    let destino_dir = raiz().join(&slug).join(sub);
    if !destino_dir.is_dir() {
        return Err(crate::idioma::msg(
            "Projeto de video nao encontrado.",
            "Video project not found.",
        ));
    }

    let base = origem
        .file_stem()
        .map(|s| crate::galeria::slugificar(&s.to_string_lossy()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "clipe".into());
    let mut destino = destino_dir.join(format!("{base}.{ext}"));
    let mut n = 2;
    while destino.exists() {
        destino = destino_dir.join(format!("{base}-{n}.{ext}"));
        n += 1;
    }

    // Copia, e nao move: o arquivo e da pessoa e continua onde ela deixou.
    // Mover o video original de alguem para dentro de uma pasta do app seria
    // decidir pelo disco dela.
    let bytes =
        std::fs::copy(origem, &destino).map_err(|e| format!("{}: {e}", origem.display()))?;

    Ok(Item {
        nome: destino
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        caminho: destino.to_string_lossy().to_string(),
        bytes,
    })
}

/// Apaga um arquivo de um projeto.
///
/// Confere que o caminho esta DENTRO da raiz dos videos antes de apagar: um
/// caminho vindo da tela nao e confiavel, e um `..` no meio apagaria arquivo
/// qualquer do disco. Mesma guarda do `galeria::remover_item`, e pelo mesmo
/// motivo.
pub fn remover_item(caminho: &str) -> Result<(), String> {
    let alvo = std::fs::canonicalize(caminho).map_err(|e| e.to_string())?;
    let dentro = std::fs::canonicalize(raiz()).map_err(|e| e.to_string())?;
    if !alvo.starts_with(&dentro) {
        return Err(crate::idioma::msg(
            "Esse arquivo nao esta num projeto de video.",
            "That file is not in a video project.",
        ));
    }
    std::fs::remove_file(alvo).map_err(|e| e.to_string())
}

pub fn remover_projeto(slug: &str) -> Result<(), String> {
    let slug = crate::galeria::slugificar(slug);
    if slug.is_empty() {
        return Err("nome invalido".into());
    }
    let dir = raiz().join(&slug);
    if dir.is_dir() {
        std::fs::remove_dir_all(dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn clipe_falado_conta_como_voz_e_clipe_mudo_nao() {
        // Sem isto, quem sobe um video em que fala seria interrompido com
        // "voce quer narracao?" — e aceitar poria uma segunda voz por cima da
        // dela. Ja um b-roll mudo nao traz voz nenhuma e a pergunta continua
        // fazendo sentido.
        use crate::video::analise::{Clipe, Trecho};
        let base = Projeto {
            slug: "x".into(),
            nome: "x".into(),
            caminho: "/tmp/x".into(),
            imagens: vec![],
            clipes: vec![],
            audio: vec![],
            narracao: vec![],
            saidas: vec![],
            bytes: 0,
        };
        let clipe = |audio: bool, som: bool| Clipe {
            nome: "t.mp4".into(),
            duracao_s: 5.0,
            largura: 1920,
            altura: 1080,
            fps: 30.0,
            tem_audio: audio,
            com_som: if som {
                vec![Trecho {
                    de_s: 0.0,
                    ate_s: 4.0,
                }]
            } else {
                vec![]
            },
            pausas: 0,
            erro: None,
        };

        assert!(base.tem_voz(&[clipe(true, true)]), "clipe falado e voz");
        assert!(
            !base.tem_voz(&[clipe(false, false)]),
            "b-roll mudo nao e voz"
        );
        assert!(!base.tem_voz(&[clipe(true, false)]), "faixa muda nao e voz");
        assert!(!base.tem_voz(&[]), "sem nada nao ha voz");
    }

    #[test]
    fn cada_pasta_aceita_so_o_que_ela_sabe_usar() {
        // Um mp3 em `imagens/` nunca vira quadro. Aceitar ali so adiaria a
        // falha para o render, que e onde ela custa mais caro para diagnosticar.
        assert!(extensoes("imagens").contains(&"png"));
        assert!(!extensoes("imagens").contains(&"mp3"));
        assert!(extensoes("narracao").contains(&"mp3"));
        assert!(!extensoes("narracao").contains(&"png"));
    }

    #[test]
    fn a_saida_nao_aceita_arquivo_de_ninguem() {
        // `saida/` e do render. Se ela aparecesse no `PASTAS`, a tela ofereceria
        // um botao de subir arquivo para a pasta que o render limpa.
        assert!(extensoes(SAIDA).is_empty());
    }

    #[test]
    fn a_narracao_e_declarada_pela_pasta_e_nao_adivinhada() {
        // O nome do arquivo nao decide nada: `audio-final-2.mp3` pode ser
        // trilha ou locucao, e adivinhar seria adivinhacao com aparencia de
        // deteccao. So a pasta responde.
        let mut p = Projeto {
            slug: "x".into(),
            nome: "x".into(),
            caminho: "/tmp/x".into(),
            imagens: vec![],
            clipes: vec![],
            audio: vec![Item {
                nome: "narracao-final.mp3".into(),
                caminho: "/tmp/x/audio/narracao-final.mp3".into(),
                bytes: 1,
            }],
            narracao: vec![],
            saidas: vec![],
            bytes: 1,
        };
        assert!(
            !p.tem_narracao(),
            "um arquivo chamado 'narracao' na pasta de trilha nao e narracao"
        );

        p.narracao.push(Item {
            nome: "vo-01.mp3".into(),
            caminho: "/tmp/x/narracao/vo-01.mp3".into(),
            bytes: 1,
        });
        assert!(p.tem_narracao());
    }
}
