//! Cofre local de segredos: chave do Gemini e credenciais das redes sociais.
//!
//! Escopo honesto do que isto protege: o arquivo do cofre e cifrado com
//! AES-256-GCM, entao um backup, um sync de nuvem ou alguem lendo o disco nao
//! ve as senhas em texto. A chave mora ao lado, com permissao 0600. Isso NAO
//! protege contra um processo rodando como voce nem contra root nesta maquina.
//!
//! Por isso o padrao e nao guardar senha: a sessao do navegador fica persistida
//! no perfil do Playwright, e a senha so e necessaria no primeiro login.

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::platform;

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Vault {
    /// Chave do Gemini. Continua sendo um campo proprio, e nao mais uma entrada
    /// no mapa abaixo, porque cofres gravados antes dos outros provedores
    /// existirem so tem este nome: mover a chave quebraria a configuracao de
    /// quem ja usava o app.
    #[serde(default)]
    pub gemini_api_key: String,
    /// slug do provedor de imagem -> chave. O Gemini le do campo acima.
    #[serde(default)]
    pub image_keys: BTreeMap<String, String>,
    /// slug da rede -> credenciais. Vazio quando o usuario opta por nao salvar.
    #[serde(default)]
    pub credentials: BTreeMap<String, Credential>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

impl Vault {
    /// A credencial do provedor escolhido, venha ela do campo antigo ou do mapa.
    pub fn chave_de(&self, p: crate::imagem::ProvedorImagem) -> String {
        if p == crate::imagem::ProvedorImagem::Gemini {
            return self.gemini_api_key.clone();
        }
        self.image_keys.get(p.slug()).cloned().unwrap_or_default()
    }

    pub fn definir_chave(&mut self, p: crate::imagem::ProvedorImagem, chave: &str) {
        let chave = chave.trim().to_string();
        if p == crate::imagem::ProvedorImagem::Gemini {
            self.gemini_api_key = chave;
            return;
        }
        if chave.is_empty() {
            self.image_keys.remove(p.slug());
        } else {
            self.image_keys.insert(p.slug().to_string(), chave);
        }
    }
}

fn vault_path() -> PathBuf {
    platform::current().data_dir().join("vault.bin")
}

fn key_path() -> PathBuf {
    platform::current().data_dir().join("vault.key")
}

/// Le a chave do disco ou cria uma nova com permissao restrita.
fn load_or_create_key() -> Result<[u8; KEY_LEN], String> {
    let path = key_path();
    if let Ok(bytes) = std::fs::read(&path) {
        if bytes.len() == KEY_LEN {
            let mut key = [0u8; KEY_LEN];
            key.copy_from_slice(&bytes);
            return Ok(key);
        }
    }
    let mut key = [0u8; KEY_LEN];
    rand::thread_rng().fill_bytes(&mut key);

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, key).map_err(|e| format!("falha ao gravar a chave do cofre: {e}"))?;
    restrict(&path)?;
    Ok(key)
}

/// 0600 em Unix. No Windows a ACL herdada do perfil do usuario ja restringe.
fn restrict(path: &std::path::Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("falha ao restringir {path:?}: {e}"))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn cipher() -> Result<Aes256Gcm, String> {
    let key = load_or_create_key()?;
    Ok(cipher_de(&key))
}

/// O cifrador a partir de uma chave crua.
///
/// Separado de `cipher()` para que o formato em disco possa ser testado sem
/// tocar no diretório de dados de quem roda os testes.
fn cipher_de(key: &[u8; KEY_LEN]) -> Aes256Gcm {
    Aes256Gcm::new(&Key::<Aes256Gcm>::from(*key))
}

/// O formato em disco: `[nonce de 12 bytes][ciphertext]`.
///
/// Estas duas funções são o contrato que precisa sobreviver a qualquer troca
/// de versão da biblioteca. Um cofre gravado por uma versão anterior tem de
/// continuar legível — quem perder isto perde a chave da API e as credenciais
/// de quem já usava o aplicativo.
fn selar(cipher: &Aes256Gcm, nonce: &[u8; NONCE_LEN], plain: &[u8]) -> Result<Vec<u8>, String> {
    let ciphertext = cipher
        .encrypt(&Nonce::from(*nonce), plain)
        .map_err(|e| format!("falha ao cifrar o cofre: {e}"))?;
    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(nonce);
    blob.extend_from_slice(&ciphertext);
    Ok(blob)
}

fn abrir(cipher: &Aes256Gcm, blob: &[u8]) -> Option<Vec<u8>> {
    if blob.len() <= NONCE_LEN {
        return None;
    }
    let (nonce, ciphertext) = blob.split_at(NONCE_LEN);
    let nonce: [u8; NONCE_LEN] = nonce.try_into().ok()?;
    cipher.decrypt(&Nonce::from(nonce), ciphertext).ok()
}

pub fn load() -> Vault {
    let Ok(blob) = std::fs::read(vault_path()) else {
        return Vault::default();
    };
    let Ok(cipher) = cipher() else {
        return Vault::default();
    };
    let Some(plain) = abrir(&cipher, &blob) else {
        return Vault::default();
    };
    serde_json::from_slice(&plain).unwrap_or_default()
}

pub fn save(vault: &Vault) -> Result<(), String> {
    let cipher = cipher()?;
    let plain = serde_json::to_vec(vault).map_err(|e| e.to_string())?;

    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce);
    let blob = selar(&cipher, &nonce, &plain)?;

    let path = vault_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, blob).map_err(|e| format!("falha ao gravar o cofre: {e}"))?;
    restrict(&path)
}

/// O que o frontend pode ver: nunca o segredo em si.
#[derive(Debug, Clone, Serialize)]
pub struct VaultSummary {
    pub has_gemini_key: bool,
    pub gemini_key_hint: String,
    pub saved_networks: Vec<String>,
    pub path: String,
}

pub fn summary() -> VaultSummary {
    let vault = load();
    let key = &vault.gemini_api_key;
    let hint = if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    } else if key.is_empty() {
        String::new()
    } else {
        "****".to_string()
    };
    VaultSummary {
        has_gemini_key: !key.is_empty(),
        gemini_key_hint: hint,
        saved_networks: vault.credentials.keys().cloned().collect(),
        path: vault_path().to_string_lossy().to_string(),
    }
}

/// Um cofre real, gravado por aes-gcm 0.10.3 com a chave e o nonce fixos do
/// módulo de testes. Não regenere sem necessidade: o valor dele é justamente
/// ter nascido de uma versão anterior da biblioteca.
#[cfg(test)]
const VETOR_0_10_3: &[u8] = &[
    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0x56, 0x9e, 0x26, 0x55,
    0x93, 0x6b, 0x74, 0xd1, 0x2c, 0xea, 0x49, 0xae, 0x16, 0x8f, 0x9c, 0xab, 0x93, 0x04, 0xa8, 0x60,
    0xd7, 0x5f, 0x3b, 0x8a, 0xa7, 0x06, 0xb9, 0xf0, 0x60, 0xdd, 0xc0, 0x8b, 0x89, 0xf4, 0xe0, 0xca,
    0xf1, 0x0d, 0xa5, 0x4a, 0x38, 0xfe, 0xf9, 0x8f, 0x6d, 0x1f, 0x0e, 0x3b, 0x21, 0x0d, 0x28, 0xbc,
    0xfe, 0xfe, 0x86, 0x6b,
];

#[cfg(test)]
mod testes {
    use super::*;

    /// Uma chave fixa, para que o vetor abaixo seja reproduzível.
    const CHAVE: [u8; KEY_LEN] = [
        0x9f, 0x2c, 0x41, 0x08, 0xd7, 0x63, 0xb5, 0x1a, 0xee, 0x37, 0x90, 0x4c, 0x25, 0x8b, 0x6d,
        0x13, 0x7a, 0xf0, 0x59, 0xc2, 0x84, 0x31, 0xab, 0x6e, 0x0d, 0x97, 0x52, 0xe8, 0x3f, 0xc6,
        0x71, 0x44,
    ];
    const NONCE: [u8; NONCE_LEN] = [
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc,
    ];

    #[test]
    fn o_que_foi_selado_volta_igual() {
        let c = cipher_de(&CHAVE);
        let segredo = br#"{"gemini_key":"AIzaEXEMPLO","contas":{}}"#;
        let blob = selar(&c, &NONCE, segredo).expect("selar");
        assert_eq!(abrir(&c, &blob).as_deref(), Some(&segredo[..]));
    }

    #[test]
    fn o_nonce_vai_na_frente_e_o_texto_nao_aparece_cru() {
        let c = cipher_de(&CHAVE);
        let segredo = b"chave-secreta-do-gemini";
        let blob = selar(&c, &NONCE, segredo).expect("selar");
        assert_eq!(&blob[..NONCE_LEN], &NONCE, "o nonce abre o arquivo");
        // O ponto do cofre: quem ler o arquivo no disco não acha o segredo.
        assert!(
            !blob.windows(segredo.len()).any(|j| j == segredo),
            "o texto claro vazou para o arquivo"
        );
    }

    #[test]
    fn chave_errada_nao_abre() {
        let blob = selar(&cipher_de(&CHAVE), &NONCE, b"segredo").expect("selar");
        let mut outra = CHAVE;
        outra[0] ^= 0xff;
        assert_eq!(abrir(&cipher_de(&outra), &blob), None);
    }

    #[test]
    fn byte_trocado_invalida_o_cofre() {
        // AES-GCM é autenticado: adulterar o arquivo tem de falhar a abertura,
        // não devolver lixo. É isto que separa cifrar de proteger.
        let c = cipher_de(&CHAVE);
        let mut blob = selar(&c, &NONCE, b"segredo").expect("selar");
        let ultimo = blob.len() - 1;
        blob[ultimo] ^= 0x01;
        assert_eq!(abrir(&c, &blob), None);
    }

    #[test]
    fn arquivo_curto_demais_nao_derruba() {
        let c = cipher_de(&CHAVE);
        for n in 0..=NONCE_LEN {
            assert_eq!(abrir(&c, &vec![0u8; n]), None, "tamanho {n}");
        }
    }

    /// O teste que atravessa versões da biblioteca.
    ///
    /// O vetor abaixo foi gravado com aes-gcm 0.10.3 usando a chave e o nonce
    /// fixos acima. Se uma atualização mudar o formato em disco, este teste
    /// falha — e falhar aqui é infinitamente melhor do que descobrir pelo
    /// usuário que perdeu a chave da API e as credenciais das redes.
    /// Roda com `cargo test -- --ignored gerar_vetor --nocapture` para
    /// produzir o literal de `VETOR_0_10_3` quando ele precisar ser refeito.
    #[test]
    #[ignore]
    fn gerar_vetor() {
        let blob = selar(
            &cipher_de(&CHAVE),
            &NONCE,
            br#"{"gemini_key":"AIzaEXEMPLO","contas":{}}"#,
        )
        .expect("selar");
        let corpo: Vec<String> = blob.iter().map(|b| format!("0x{b:02x}")).collect();
        println!(
            "&[\n    {},\n];",
            corpo
                .chunks(12)
                .map(|l| l.join(", "))
                .collect::<Vec<_>>()
                .join(",\n    ")
        );
    }

    #[test]
    fn cofre_gravado_por_versao_anterior_continua_legivel() {
        let blob = VETOR_0_10_3;
        let aberto = abrir(&cipher_de(&CHAVE), blob).expect("cofre antigo ilegível");
        assert_eq!(
            String::from_utf8_lossy(&aberto),
            r#"{"gemini_key":"AIzaEXEMPLO","contas":{}}"#
        );
    }
}
