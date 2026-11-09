//! Catálogo de proveedores de IA y protocolos HTTP.
//!
//! Los proveedores OpenAI-compatible comparten request/response; Anthropic y
//! Gemini tienen adaptadores propios. La selección se guarda en
//! `config.json`; la clave nunca se guarda ahí.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// `POST {base}/chat/completions` con `Authorization: Bearer`.
    OpenAi,
    /// `POST {base}/messages` con `x-api-key` y `anthropic-version`.
    Anthropic,
    /// `POST {base}/models/{model}:generateContent` con `x-goog-api-key`.
    Gemini,
}

#[derive(Debug, Clone, Copy)]
pub struct Provider {
    pub id: &'static str,
    pub label: &'static str,
    pub protocol: Protocol,
    pub base_url: &'static str,
    pub default_model: &'static str,
    pub source_url: &'static str,
    pub needs_key: bool,
}

pub const ALL: &[Provider] = &[
    Provider {
        id: "opencode",
        label: "OpenCode Go",
        protocol: Protocol::OpenAi,
        base_url: "https://opencode.ai/inference/go/openai/v1",
        default_model: "deepseek-v4.1-flash",
        source_url: "https://opencode.ai/console/go",
        needs_key: true,
    },
    Provider {
        id: "openai",
        label: "OpenAI",
        protocol: Protocol::OpenAi,
        base_url: "https://api.openai.com/v1",
        default_model: "gpt-4o-mini",
        source_url: "https://platform.openai.com/",
        needs_key: true,
    },
    Provider {
        id: "anthropic",
        label: "Anthropic",
        protocol: Protocol::Anthropic,
        base_url: "https://api.anthropic.com/v1",
        default_model: "claude-3-5-sonnet-latest",
        source_url: "https://console.anthropic.com/",
        needs_key: true,
    },
    Provider {
        id: "gemini",
        label: "Google Gemini",
        protocol: Protocol::Gemini,
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        default_model: "gemini-2.0-flash",
        source_url: "https://aistudio.google.com/",
        needs_key: true,
    },
    Provider {
        id: "openrouter",
        label: "OpenRouter",
        protocol: Protocol::OpenAi,
        base_url: "https://openrouter.ai/api/v1",
        default_model: "openai/gpt-4o-mini",
        source_url: "https://openrouter.ai/",
        needs_key: true,
    },
    Provider {
        id: "groq",
        label: "Groq",
        protocol: Protocol::OpenAi,
        base_url: "https://api.groq.com/openai/v1",
        default_model: "llama-3.3-70b-versatile",
        source_url: "https://console.groq.com/",
        needs_key: true,
    },
    Provider {
        id: "mistral",
        label: "Mistral",
        protocol: Protocol::OpenAi,
        base_url: "https://api.mistral.ai/v1",
        default_model: "mistral-small-latest",
        source_url: "https://console.mistral.ai/",
        needs_key: true,
    },
    Provider {
        id: "deepseek",
        label: "DeepSeek",
        protocol: Protocol::OpenAi,
        base_url: "https://api.deepseek.com/v1",
        default_model: "deepseek-chat",
        source_url: "https://platform.deepseek.com/",
        needs_key: true,
    },
    Provider {
        id: "ollama",
        label: "Ollama (local)",
        protocol: Protocol::OpenAi,
        base_url: "http://localhost:11434/v1",
        default_model: "llama3.2",
        source_url: "https://ollama.com/",
        needs_key: false,
    },
];

/// Devuelve el proveedor con ese id; si no existe, cae en OpenCode Go.
pub fn provider(id: &str) -> &'static Provider {
    ALL.iter()
        .find(|provider| provider.id == id)
        .unwrap_or(&ALL[0])
}

pub fn is_valid(id: &str) -> bool {
    ALL.iter().any(|provider| provider.id == id)
}

pub fn requires_key(id: &str) -> bool {
    provider(id).needs_key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_unicos_y_protocolos() {
        let mut ids: Vec<&str> = ALL.iter().map(|provider| provider.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ALL.len());
        assert_eq!(provider("openai").label, "OpenAI");
        assert_eq!(provider("anthropic").protocol, Protocol::Anthropic);
        assert_eq!(provider("gemini").protocol, Protocol::Gemini);
        assert_eq!(provider("no-existe").id, "opencode");
        assert!(!requires_key("ollama"));
        assert!(requires_key("openai"));
    }
}
