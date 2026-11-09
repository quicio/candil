//! Almacenamiento de la API key de IA en el llavero del sistema.
//!
//! La clave nunca se escribe en `config.json`, `consultas.json`, logs ni URLs.
//! `OPENCODE_API_KEY` tiene prioridad sobre el llavero para CI y scripts.

use keyring::Entry;

const SERVICE: &str = "com.quicio.candil";
const ACCOUNT: &str = "ai-api-key";

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

fn env_key() -> Option<String> {
    std::env::var("OPENCODE_API_KEY")
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn entry() -> Result<Entry, String> {
    Entry::new(SERVICE, ACCOUNT).map_err(|error| format!("No pude abrir el llavero: {error}"))
}

/// Resuelve la clave efectiva: entorno primero, después llavero.
pub fn load() -> KeyState {
    if let Some(key) = env_key() {
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

    let entry = match entry() {
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
        Err(keyring::Error::NoEntry) => KeyState {
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

pub fn save(key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("La API key está vacía.".into());
    }
    entry()?
        .set_password(key)
        .map_err(|error| format!("No pude guardar la clave en el llavero: {error}"))
}

pub fn delete() -> Result<(), String> {
    match entry()?.delete_credential() {
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
