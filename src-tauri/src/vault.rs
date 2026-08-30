//! Cofre local de segredos: chave do Gemini e credenciais das redes sociais.
//!
//! Escopo honesto do que isto protege: o arquivo do cofre e cifrado com
//! AES-256-GCM, entao um backup, um sync de nuvem ou alguem lendo o disco nao
//! ve as senhas em texto. A chave mora ao lado, com permissao 0600. Isso NAO
//! protege contra um processo rodando como voce nem contra root nesta maquina.
//!
//! Por isso o padrao e nao guardar senha: a sessao do navegador fica persistida
//! no perfil do Playwright, e a senha so e necessaria no primeiro login.

use aes_gcm::aead::{generic_array::GenericArray, Aead};
use aes_gcm::{Aes256Gcm, KeyInit};
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
    Ok(Aes256Gcm::new(GenericArray::from_slice(&key)))
}

pub fn load() -> Vault {
    let Ok(blob) = std::fs::read(vault_path()) else {
        return Vault::default();
    };
    if blob.len() <= NONCE_LEN {
        return Vault::default();
    }
    let Ok(cipher) = cipher() else {
        return Vault::default();
    };
    let (nonce, ciphertext) = blob.split_at(NONCE_LEN);
    let Ok(plain) = cipher.decrypt(GenericArray::from_slice(nonce), ciphertext) else {
        return Vault::default();
    };
    serde_json::from_slice(&plain).unwrap_or_default()
}

pub fn save(vault: &Vault) -> Result<(), String> {
    let cipher = cipher()?;
    let plain = serde_json::to_vec(vault).map_err(|e| e.to_string())?;

    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(GenericArray::from_slice(&nonce), plain.as_ref())
        .map_err(|e| format!("falha ao cifrar o cofre: {e}"))?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(&nonce);
    blob.extend_from_slice(&ciphertext);

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
