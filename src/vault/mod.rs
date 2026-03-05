mod crypto;
mod store;

pub use store::SecretEntry;

use std::fs;
use std::path::PathBuf;

use age::x25519::{Identity, Recipient};
use anyhow::{Context, Result};
use rusqlite::Connection;

/// The main vault handle. Holds an open DB connection and loaded keys.
pub struct Vault {
    conn: Connection,
    identity: Identity,
    recipient: Recipient,
}

impl Vault {
    /// Open or initialize the vault at ~/.kagienv/.
    pub fn open() -> Result<Self> {
        let vault_dir = Self::vault_dir()?;
        let keys_dir = vault_dir.join("keys");
        let identity_path = keys_dir.join("identity.txt");
        let db_path = vault_dir.join("vault.db");

        if !vault_dir.exists() {
            fs::create_dir_all(&vault_dir).context("Failed to create ~/.kagienv/")?;
            Self::set_dir_permissions(&vault_dir)?;
        }
        if !keys_dir.exists() {
            fs::create_dir_all(&keys_dir).context("Failed to create ~/.kagienv/keys/")?;
            Self::set_dir_permissions(&keys_dir)?;
        }

        let identity = if identity_path.exists() {
            crypto::load_identity(&identity_path)?
        } else {
            eprintln!("Initializing kagienv vault at {}...", vault_dir.display());
            let id = crypto::generate_identity(&identity_path)?;
            Self::set_file_permissions(&identity_path)?;
            eprintln!("Generated new age keypair.");
            id
        };

        let recipient = identity.to_public();
        let conn = store::open_db(&db_path)?;

        Ok(Vault {
            conn,
            identity,
            recipient,
        })
    }

    /// Add (or update) a secret in the vault.
    pub fn add(&self, name: &str, value: &str) -> Result<()> {
        let encrypted = crypto::encrypt(value, &self.recipient)?;
        store::upsert_secret(&self.conn, name, &encrypted)
    }

    /// List all secrets (names + timestamps, no values).
    pub fn list(&self) -> Result<Vec<SecretEntry>> {
        store::list_secrets(&self.conn)
    }

    /// Retrieve and decrypt a single secret value.
    pub fn get(&self, name: &str) -> Result<String> {
        let encrypted = store::get_secret(&self.conn, name)?;
        crypto::decrypt(&encrypted, &self.identity)
    }

    /// Retrieve and decrypt all secrets.
    pub fn get_all(&self) -> Result<Vec<(String, String)>> {
        let encrypted_entries = store::get_all_secrets(&self.conn)?;
        let mut result = Vec::with_capacity(encrypted_entries.len());
        for (name, encrypted) in encrypted_entries {
            let value = crypto::decrypt(&encrypted, &self.identity)?;
            result.push((name, value));
        }
        Ok(result)
    }

    /// Delete a secret from the vault.
    pub fn delete(&self, name: &str) -> Result<()> {
        store::delete_secret(&self.conn, name)
    }

    fn vault_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".kagienv"))
    }

    #[cfg(unix)]
    fn set_dir_permissions(path: &PathBuf) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("Failed to set permissions on {}", path.display()))
    }

    #[cfg(not(unix))]
    fn set_dir_permissions(_path: &PathBuf) -> Result<()> {
        Ok(())
    }

    #[cfg(unix)]
    fn set_file_permissions(path: &PathBuf) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed to set permissions on {}", path.display()))
    }

    #[cfg(not(unix))]
    fn set_file_permissions(_path: &PathBuf) -> Result<()> {
        Ok(())
    }
}
