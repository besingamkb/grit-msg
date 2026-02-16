use anyhow::{Context, Result, anyhow};
use dialoguer::Password;
use keyring::{Entry, Error as KeyringError};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

const SERVICE: &str = "com.github.grit-msg";
const ACCOUNT: &str = "groq_api_key";
const ENV_GROQ_API_KEY: &str = "GROQ_API_KEY";
const FALLBACK_FILE: &str = ".grit-msg-groq-key";
const LEGACY_SERVICE: &str = "com.github.git-ai-commit";
const LEGACY_FALLBACK_FILE: &str = ".git-ai-commit-groq-key";

pub fn load_or_prompt_groq_key() -> Result<String> {
    if let Ok(value) = env::var(ENV_GROQ_API_KEY) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_owned());
        }
    }

    if let Some(file_key) = load_fallback_file_key()? {
        return Ok(file_key);
    }

    let entry = Entry::new(SERVICE, ACCOUNT).context("failed to initialize keyring entry")?;

    match entry.get_password() {
        Ok(value) if !value.trim().is_empty() => {
            let key = value.trim().to_owned();
            // Keep local fallback in sync to avoid keyring backend drift.
            let _ = save_fallback_file_key(&key);
            Ok(key)
        }
        Ok(_) | Err(KeyringError::NoEntry) => {
            if let Some(migrated) = try_migrate_legacy_user_scoped_key(&entry)? {
                return Ok(migrated);
            }
            prompt_and_store_key(&entry)
        }
        Err(_) => prompt_and_store_key(&entry),
    }
}

pub fn clear_stored_groq_keys() -> Result<()> {
    let entry = Entry::new(SERVICE, ACCOUNT).context("failed to initialize keyring entry")?;
    let _ = delete_keyring_entry(&entry);
    let legacy_service_entry = Entry::new(LEGACY_SERVICE, ACCOUNT)
        .context("failed to initialize legacy-service keyring entry")?;
    let _ = delete_keyring_entry(&legacy_service_entry);

    let username = env::var("USER").unwrap_or_else(|_| "default".to_owned());
    let legacy_account = format!("{ACCOUNT}:{username}");
    let legacy_entry = Entry::new(SERVICE, &legacy_account)
        .context("failed to initialize legacy keyring entry")?;
    let _ = delete_keyring_entry(&legacy_entry);
    let legacy_service_legacy_account_entry = Entry::new(LEGACY_SERVICE, &legacy_account)
        .context("failed to initialize legacy-service legacy keyring entry")?;
    let _ = delete_keyring_entry(&legacy_service_legacy_account_entry);

    let path = fallback_key_path()?;
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("failed deleting fallback key file at {}", path.display()))?;
    }
    let legacy_path = legacy_fallback_key_path()?;
    if legacy_path.exists() {
        fs::remove_file(&legacy_path).with_context(|| {
            format!(
                "failed deleting legacy fallback key file at {}",
                legacy_path.display()
            )
        })?;
    }

    Ok(())
}

fn try_migrate_legacy_user_scoped_key(new_entry: &Entry) -> Result<Option<String>> {
    if let Some(file_key) = load_legacy_fallback_file_key()? {
        save_fallback_file_key(&file_key)?;
        let _ = new_entry.set_password(&file_key);
        return Ok(Some(file_key));
    }

    let old_service_entry = Entry::new(LEGACY_SERVICE, ACCOUNT)
        .context("failed to initialize old-service keyring entry")?;
    if let Ok(value) = old_service_entry.get_password() {
        let key = value.trim().to_owned();
        if !key.is_empty() {
            let _ = new_entry.set_password(&key);
            save_fallback_file_key(&key)?;
            return Ok(Some(key));
        }
    }

    let username = env::var("USER").unwrap_or_else(|_| "default".to_owned());
    let legacy_account = format!("{ACCOUNT}:{username}");
    let legacy_entries = [
        Entry::new(SERVICE, &legacy_account)
            .context("failed to initialize legacy keyring entry")?,
        Entry::new(LEGACY_SERVICE, &legacy_account)
            .context("failed to initialize old-service legacy keyring entry")?,
    ];

    for legacy_entry in legacy_entries {
        match legacy_entry.get_password() {
            Ok(value) if !value.trim().is_empty() => {
                let key = value.trim().to_owned();
                let _ = new_entry.set_password(&key);
                save_fallback_file_key(&key)?;
                return Ok(Some(key));
            }
            Ok(_) | Err(KeyringError::NoEntry) => {}
            Err(err) => {
                return Err(anyhow!(
                    "failed reading legacy Groq API key from keyring: {err}"
                ));
            }
        }
    }

    Ok(None)
}

fn delete_keyring_entry(entry: &Entry) {
    match entry.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => {}
        Err(_) => {}
    }
}

fn prompt_and_store_key(entry: &Entry) -> Result<String> {
    print_api_key_help();
    let key = Password::new()
        .with_prompt("Enter Groq API key")
        .allow_empty_password(false)
        .interact()
        .context("failed reading Groq API key")?;
    let key = key.trim().to_owned();
    save_fallback_file_key(&key)?;
    let _ = entry.set_password(&key);
    Ok(key)
}

fn print_api_key_help() {
    println!("Groq API key not found in keyring.");
    println!("Get a free key from: https://console.groq.com/keys");
    println!("Create/sign in, generate an API key, then paste it below.");
}

fn fallback_key_path() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME environment variable is not set")?;
    Ok(PathBuf::from(home).join(FALLBACK_FILE))
}

fn legacy_fallback_key_path() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME environment variable is not set")?;
    Ok(PathBuf::from(home).join(LEGACY_FALLBACK_FILE))
}

fn load_fallback_file_key() -> Result<Option<String>> {
    let path = fallback_key_path()?;
    load_key_from_path(&path)
}

fn load_legacy_fallback_file_key() -> Result<Option<String>> {
    let path = legacy_fallback_key_path()?;
    load_key_from_path(&path)
}

fn load_key_from_path(path: &PathBuf) -> Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed reading fallback key file at {}", path.display()))?;
    let key = raw.trim();
    if key.is_empty() {
        return Ok(None);
    }
    Ok(Some(key.to_owned()))
}

fn save_fallback_file_key(key: &str) -> Result<()> {
    let path = fallback_key_path()?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .with_context(|| format!("failed creating fallback key file at {}", path.display()))?;
    file.write_all(key.as_bytes())
        .with_context(|| format!("failed writing fallback key file at {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).with_context(|| {
            format!(
                "failed to set secure permissions on fallback key file at {}",
                path.display()
            )
        })?;
    }

    Ok(())
}
