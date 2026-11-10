//! Almacenamiento de la API key de IA en el llavero del sistema.
//!
//! La clave nunca se escribe en `config.json`, `consultas.json`, logs ni URLs.
//! `OPENCODE_API_KEY` tiene prioridad sobre el llavero para CI y scripts.

use keyring::Entry;

const SERVICE: &str = "com.quicio.candil";
const LEGACY_ACCOUNT: &str = "ai-api-key";
const ACCOUNT_PREFIX: &str = "ai-api-key/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Env,
    Keyring,
    None,
}

#[derive(Debug, Clone)]
pub struct KeyState {
    pub key: Option<String>,
    pub source: Source,
    pub error: Option<String>,
}

fn env_applies(provider: &str) -> bool {
    provider == "opencode"
}

fn env_key(provider: &str) -> Option<String> {
    // La variable histórica solo aplica a OpenCode Go. Usarla para otro
    // proveedor mandaría la clave equivocada.
    if !env_applies(provider) {
        return None;
    }
    std::env::var("OPENCODE_API_KEY")
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

/// Cuenta del llavero para un proveedor. El valor incluye solo el id, nunca la
/// clave.
pub fn account_for(provider: &str) -> String {
    format!("{ACCOUNT_PREFIX}{provider}")
}

fn entry_for(provider: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, &account_for(provider))
        .map_err(|error| format!("No pude abrir el llavero: {error}"))
}

fn legacy_entry() -> Result<Entry, String> {
    Entry::new(SERVICE, LEGACY_ACCOUNT)
        .map_err(|error| format!("No pude abrir el llavero: {error}"))
}

/// Resuelve la clave efectiva del proveedor: entorno (solo OpenCode Go),
/// después su entrada del llavero. Si no hay entrada propia, migra la clave
/// histórica compartida a la primera cuenta que la necesite.
pub fn load(provider: &str) -> KeyState {
    if let Some(key) = env_key(provider) {
        return KeyState {
            key: Some(key),
            source: Source::Env,
            error: None,
        };
    }

    // Capturas y CI corren con un HOME falso, sin llavero de login.
    if std::env::var_os("CANDIL_NO_KEYRING").is_some() {
        return KeyState {
            key: None,
            source: Source::None,
            error: None,
        };
    }

    let entry = match entry_for(provider) {
        Ok(entry) => entry,
        Err(error) => {
            return KeyState {
                key: None,
                source: Source::None,
                error: Some(error),
            };
        }
    };

    match entry.get_password() {
        Ok(key) if !key.trim().is_empty() => KeyState {
            key: Some(key),
            source: Source::Keyring,
            error: None,
        },
        Ok(_) => KeyState {
            key: None,
            source: Source::None,
            error: None,
        },
        Err(keyring::Error::NoEntry) => migrate_legacy(provider),
        Err(error) => KeyState {
            key: None,
            source: Source::None,
            error: Some(format!("No pude leer la clave del llavero: {error}")),
        },
    }
}

/// Mueve la clave histórica `ai-api-key` a la cuenta del proveedor actual. Si
/// falla la escritura, igual devuelve la clave para esta sesión; si falla el
/// borrado, la entrada vieja queda como respaldo.
fn migrate_legacy(provider: &str) -> KeyState {
    let legacy = match legacy_entry() {
        Ok(entry) => entry,
        Err(_) => {
            return KeyState {
                key: None,
                source: Source::None,
                error: None,
            }
        }
    };
    match legacy.get_password() {
        Ok(key) if !key.trim().is_empty() => {
            let key = key.trim().to_string();
            if let Ok(entry) = entry_for(provider) {
                let _ = entry.set_password(&key);
            }
            let _ = legacy.delete_credential();
            KeyState {
                key: Some(key),
                source: Source::Keyring,
                error: None,
            }
        }
        Ok(_) | Err(keyring::Error::NoEntry) => KeyState {
            key: None,
            source: Source::None,
            error: None,
        },
        Err(error) => KeyState {
            key: None,
            source: Source::None,
            error: Some(format!("No pude leer la clave del llavero: {error}")),
        },
    }
}

pub fn save(provider: &str, key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("La API key está vacía.".into());
    }
    entry_for(provider)?
        .set_password(key)
        .map_err(|error| format!("No pude guardar la clave en el llavero: {error}"))
}

pub fn delete(provider: &str) -> Result<(), String> {
    match entry_for(provider)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("No pude borrar la clave del llavero: {error}")),
    }
}

/// Texto de estado para la UI, sin exponer la clave.
pub fn status(state: &KeyState) -> String {
    if let Some(error) = &state.error {
        return error.clone();
    }
    match state.source {
        Source::Env => "OPENCODE_API_KEY detectada en el entorno (tiene prioridad)".into(),
        Source::Keyring => "Clave guardada en el llavero del sistema".into(),
        Source::None => "Sin API key configurada".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuentas_por_proveedor_son_estables_y_distintas() {
        assert_eq!(account_for("openai"), "ai-api-key/openai");
        assert_eq!(account_for("opencode"), "ai-api-key/opencode");
        assert_ne!(account_for("openai"), account_for("anthropic"));
        assert_ne!(account_for("openai"), LEGACY_ACCOUNT);
    }

    #[test]
    fn variable_de_entorno_solo_aplica_a_opencode() {
        assert!(env_applies("opencode"));
        assert!(!env_applies("openai"));
        assert!(!env_applies("anthropic"));
        assert!(!env_applies("ollama"));
    }
}
