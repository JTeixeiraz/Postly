//! Onde vivem os arquivos que o app EXECUTA, e não os que ele mostra.
//!
//! O `sidecar/` (Playwright e Remotion) e o `motion/` (a biblioteca de cenas)
//! não são código Rust nem frontend: são arquivos que precisam existir em disco
//! na máquina de quem usa, com um `node_modules` ao lado, para o Node poder
//! rodá-los.
//!
//! ISTO EXISTE POR CAUSA DE UM DEFEITO REAL. Um usuário no Windows abriu o app,
//! viu o erro de navegador e **nada foi instalado automaticamente**. A causa: o
//! `sidecar/` nunca entrava no instalador. Em desenvolvimento a pasta está ao
//! lado do `src-tauri`, e por isso tudo funcionava na máquina de quem escreveu
//! — o pior tipo de bug, o que só existe para os outros.
//!
//! POR QUE COPIAR EM VEZ DE USAR O RECURSO DIRETO. O provisionamento roda
//! `npm ci`, que escreve um `node_modules` de centenas de MB. No Windows o app
//! instala em `C:\Program Files\...`, onde um usuário comum **não tem permissão
//! de escrita** — e no macOS o pacote é assinado, então escrever dentro dele
//! quebraria a assinatura. Os recursos empacotados são somente leitura por
//! natureza; a cópia gravável mora no diretório de dados da pessoa.
//!
//! ```text
//! recurso (só leitura)                 cópia gravável
//! <app>/resources/sidecar/  ─────────▶ <dados>/runtime/sidecar/
//! <app>/resources/motion/   ─────────▶ <dados>/runtime/motion/
//!                                      <dados>/runtime/.versao
//! ```

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// O diretório de recursos do pacote, definido uma vez na abertura.
///
/// Vem de `app.path().resource_dir()`, que só existe com um `AppHandle` na mão.
/// Guardar aqui evita passar o handle por `navegador`, `browser` e `video` só
/// para carregar um caminho constante — e é o mesmo padrão que o `platform` e o
/// `claude` já usam.
static RECURSOS: OnceLock<PathBuf> = OnceLock::new();

pub fn definir_diretorio_de_recursos(dir: PathBuf) {
    let _ = RECURSOS.set(dir);
}

/// A raiz de execução em DESENVOLVIMENTO, quando ela existe.
///
/// `CARGO_MANIFEST_DIR` é gravado em tempo de compilação: na máquina de quem
/// usa ele aponta para um caminho que não existe, então o teste do arquivo é o
/// que separa os dois mundos. Não basta olhar o caminho — ele precisa estar lá.
fn raiz_de_desenvolvimento() -> Option<PathBuf> {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    repo.join("sidecar/package.json")
        .exists()
        .then(|| repo.canonicalize().unwrap_or(repo))
}

/// A cópia gravável, no diretório de dados.
fn raiz_gravavel() -> PathBuf {
    crate::platform::current().data_dir().join("runtime")
}

/// Onde `sidecar/` e `motion/` estão AGORA, para quem for executá-los.
///
/// Em desenvolvimento é o próprio repositório — semear uma cópia ali só
/// duplicaria o que já está versionado, e faria uma edição no sidecar não
/// aparecer até alguém lembrar de semear de novo.
pub fn raiz() -> PathBuf {
    raiz_de_desenvolvimento().unwrap_or_else(raiz_gravavel)
}

/// Copia os recursos empacotados para a cópia gravável, quando preciso.
///
/// Roda na abertura, antes de qualquer sonda. É barata quando não há o que
/// fazer: uma leitura de arquivo de versão.
///
/// A SEMEADURA REFAZ QUANDO A VERSÃO MUDA, e isso não é zelo: numa atualização
/// o `remotion-agent.mjs` pode ganhar uma flag que o Rust passou a mandar, e
/// uma cópia velha ao lado de um binário novo daria um erro que não aponta para
/// lugar nenhum. O `node_modules` NÃO é apagado junto — ele é o que custa
/// minutos de download, e o `npm ci` seguinte reconcilia o que mudou.
pub fn semear() -> Result<(), String> {
    // A POLÍTICA FICA AQUI E A MECÂNICA EM `semear_em`, de propósito. Esta
    // função só sabe responder "devo semear?", e a resposta é não em
    // desenvolvimento — onde o repositório já é a raiz. Se as duas coisas
    // morassem juntas, o caminho que só existe num app instalado seria
    // impossível de testar na máquina de quem escreve, que é exatamente como o
    // defeito do sidecar sobreviveu até chegar num usuário.
    if raiz_de_desenvolvimento().is_some() {
        return Ok(());
    }
    let Some(recursos) = RECURSOS.get() else {
        return Ok(());
    };
    semear_em(recursos, &raiz_gravavel(), env!("CARGO_PKG_VERSION"))
}

/// Copia os recursos para o destino, se a versão gravada não bater.
///
/// As pastas que o app precisa executar. `sidecar/` roda o Playwright e o
/// Remotion; `motion/` é a biblioteca de cenas que o render empacota.
const PASTAS: &[&str] = &["sidecar", "motion"];

fn semear_em(recursos: &Path, destino: &Path, versao: &str) -> Result<(), String> {
    let carimbo = destino.join(".versao");
    if std::fs::read_to_string(&carimbo).is_ok_and(|v| v.trim() == versao) {
        return Ok(());
    }

    for pasta in PASTAS {
        let de = recursos.join(pasta);
        if !de.is_dir() {
            return Err(format!(
                "recurso ausente no pacote: {}. O instalador foi gerado sem `bundle.resources` \
                 — sem essa pasta o navegador nao instala e o video nao renderiza.",
                de.display()
            ));
        }
        copiar_arvore(&de, &destino.join(pasta))?;
    }

    std::fs::create_dir_all(destino).map_err(|e| e.to_string())?;
    std::fs::write(&carimbo, versao).map_err(|e| e.to_string())
}

/// Cópia recursiva, sobrescrevendo o que já existe.
///
/// Não apaga o destino antes: `node_modules` mora dentro de `sidecar/` e
/// apagá-lo transformaria toda atualização do app numa reinstalação de
/// centenas de MB.
fn copiar_arvore(de: &Path, para: &Path) -> Result<(), String> {
    std::fs::create_dir_all(para).map_err(|e| format!("{}: {e}", para.display()))?;
    for entrada in std::fs::read_dir(de).map_err(|e| format!("{}: {e}", de.display()))? {
        let entrada = entrada.map_err(|e| e.to_string())?;
        let destino = para.join(entrada.file_name());
        if entrada.path().is_dir() {
            copiar_arvore(&entrada.path(), &destino)?;
        } else {
            std::fs::copy(entrada.path(), &destino)
                .map_err(|e| format!("{}: {e}", destino.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn em_desenvolvimento_a_raiz_e_o_repositorio() {
        // Quando este teste roda, o repositório está em disco — então a raiz
        // tem que ser ele, e não a cópia no diretório de dados. Semear em
        // desenvolvimento faria uma edição no sidecar não aparecer até alguém
        // lembrar de semear de novo.
        let r = raiz();
        assert!(
            r.join("sidecar/package.json").exists(),
            "a raiz {} nao tem o sidecar",
            r.display()
        );
        assert!(r.join("motion/src/index.ts").exists());
    }

    /// Um "pacote" com a forma que o instalador produz.
    fn pacote_falso(base: &Path) -> PathBuf {
        let recursos = base.join("resources");
        std::fs::create_dir_all(recursos.join("sidecar")).unwrap();
        std::fs::create_dir_all(recursos.join("motion/src")).unwrap();
        std::fs::write(recursos.join("sidecar/package.json"), "{}").unwrap();
        std::fs::write(recursos.join("sidecar/remotion-agent.mjs"), "// v1").unwrap();
        std::fs::write(recursos.join("motion/src/index.ts"), "// v1").unwrap();
        recursos
    }

    #[test]
    fn a_semeadura_leva_as_duas_pastas_e_carimba_a_versao() {
        // ESTE E O CAMINHO QUE SO EXISTE NUM APP INSTALADO, e por isso ele
        // sobreviveu ate chegar num usuario: na maquina de quem escreve, o
        // repositorio ja esta em disco e nada disto roda.
        let base = std::env::temp_dir().join("postly-teste-semear");
        let _ = std::fs::remove_dir_all(&base);
        let recursos = pacote_falso(&base);
        let destino = base.join("runtime");

        semear_em(&recursos, &destino, "1.0.0").unwrap();

        assert!(destino.join("sidecar/remotion-agent.mjs").exists());
        assert!(destino.join("motion/src/index.ts").exists());
        assert_eq!(
            std::fs::read_to_string(destino.join(".versao")).unwrap(),
            "1.0.0"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_versao_nova_refaz_a_copia_e_preserva_o_node_modules() {
        // Numa atualizacao, o `remotion-agent.mjs` pode ganhar uma flag que o
        // Rust passou a mandar; uma copia velha ao lado de um binario novo daria
        // um erro que nao aponta para lugar nenhum. Mas apagar o `node_modules`
        // junto transformaria toda atualizacao numa reinstalacao de 294 MB.
        let base = std::env::temp_dir().join("postly-teste-semear-v2");
        let _ = std::fs::remove_dir_all(&base);
        let recursos = pacote_falso(&base);
        let destino = base.join("runtime");

        semear_em(&recursos, &destino, "1.0.0").unwrap();
        std::fs::create_dir_all(destino.join("sidecar/node_modules/playwright")).unwrap();
        std::fs::write(
            destino.join("sidecar/node_modules/playwright/i.js"),
            "pesado",
        )
        .unwrap();

        std::fs::write(recursos.join("sidecar/remotion-agent.mjs"), "// v2").unwrap();
        semear_em(&recursos, &destino, "1.1.0").unwrap();

        assert_eq!(
            std::fs::read_to_string(destino.join("sidecar/remotion-agent.mjs")).unwrap(),
            "// v2",
            "a copia velha sobreviveu a atualizacao"
        );
        assert!(
            destino
                .join("sidecar/node_modules/playwright/i.js")
                .exists(),
            "o node_modules foi apagado na atualizacao"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_mesma_versao_nao_recopia() {
        // A semeadura roda em toda abertura. Se ela recopiasse sempre, cada
        // abertura pagaria uma copia de arquivos por nada.
        let base = std::env::temp_dir().join("postly-teste-semear-idem");
        let _ = std::fs::remove_dir_all(&base);
        let recursos = pacote_falso(&base);
        let destino = base.join("runtime");

        semear_em(&recursos, &destino, "1.0.0").unwrap();
        std::fs::write(destino.join("sidecar/remotion-agent.mjs"), "editado a mao").unwrap();
        semear_em(&recursos, &destino, "1.0.0").unwrap();

        assert_eq!(
            std::fs::read_to_string(destino.join("sidecar/remotion-agent.mjs")).unwrap(),
            "editado a mao",
            "recopiou com a mesma versao"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn pacote_sem_sidecar_falha_dizendo_o_que_falta() {
        // O DEFEITO RELATADO, virado teste. Um instalador gerado sem
        // `bundle.resources` deixava a pessoa com um erro de navegador que
        // nunca saia da tela, porque nao havia o que instalar. Agora a falha
        // diz o nome da pasta e a causa.
        let base = std::env::temp_dir().join("postly-teste-semear-vazio");
        let _ = std::fs::remove_dir_all(&base);
        let recursos = base.join("resources");
        std::fs::create_dir_all(&recursos).unwrap();

        let e = semear_em(&recursos, &base.join("runtime"), "1.0.0").unwrap_err();
        assert!(e.contains("sidecar"), "{e}");
        assert!(e.contains("bundle.resources"), "{e}");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_arvore_e_copiada_inteira_e_sem_apagar_o_destino() {
        let base = std::env::temp_dir().join("postly-teste-recursos");
        let _ = std::fs::remove_dir_all(&base);
        let de = base.join("de");
        let para = base.join("para");
        std::fs::create_dir_all(de.join("src")).unwrap();
        std::fs::write(de.join("package.json"), "{}").unwrap();
        std::fs::write(de.join("src/a.ts"), "a").unwrap();

        // Um `node_modules` que já existia no destino: apagar isto a cada
        // atualizacao custaria centenas de MB de download.
        std::fs::create_dir_all(para.join("node_modules/x")).unwrap();
        std::fs::write(para.join("node_modules/x/i.js"), "pesado").unwrap();

        copiar_arvore(&de, &para).unwrap();

        assert!(para.join("package.json").exists());
        assert!(para.join("src/a.ts").exists());
        assert!(
            para.join("node_modules/x/i.js").exists(),
            "o node_modules foi apagado na copia"
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
