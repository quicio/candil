//! Comprobación de la última release, selección de binario y descarga
//! verificada. No reemplaza ni ejecuta nada: solo deja el archivo listo.

use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const RELEASES_URL: &str = "https://api.github.com/repos/quicio/candil/releases/latest";
const TIMEOUT_SECS: u64 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: String,
    pub tag: String,
    pub page_url: String,
    pub notes: String,
    pub published_at: String,
    pub asset: Option<Asset>,
    pub sums_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    published_at: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Versión compilada, sin prefijo.
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Assets aceptables para esta plataforma, en orden de preferencia.
pub fn platform_asset_names() -> &'static [&'static str] {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        &["candil-macos-arm64-app.zip", "candil-macos-arm64"]
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        &["candil-linux-x86_64"]
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        &["candil-windows-x86_64.exe"]
    } else {
        &[]
    }
}

/// Primer asset disponible de la plataforma actual.
pub fn select_platform_asset(assets: &[Asset]) -> Option<Asset> {
    platform_asset_names()
        .iter()
        .find_map(|name| select_asset_for(assets, name))
}

fn version_parts(version: &str) -> [u64; 3] {
    let mut parts = [0u64; 3];
    let clean = version.trim().trim_start_matches(['v', 'V']);
    for (index, part) in clean.split('.').take(3).enumerate() {
        let digits: String = part.chars().take_while(|ch| ch.is_ascii_digit()).collect();
        parts[index] = digits.parse().unwrap_or(0);
    }
    parts
}

/// Compara dos versiones numéricas (`v0.5.0` vs `0.6.0`).
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    version_parts(a).cmp(&version_parts(b))
}

pub fn is_newer(remote: &str, current: &str) -> bool {
    compare_versions(remote, current) == Ordering::Greater
}

pub fn select_asset_for(assets: &[Asset], name: &str) -> Option<Asset> {
    if name.is_empty() {
        return None;
    }
    assets.iter().find(|asset| asset.name == name).cloned()
}

/// Parsea la respuesta de `releases/latest`. Devuelve `None` si no hay una
/// versión estable más nueva o si la release no aplica.
pub fn parse_release(json: &str, current: &str) -> Result<Option<ReleaseInfo>, String> {
    let release: GhRelease =
        serde_json::from_str(json).map_err(|e| format!("Respuesta de GitHub inválida: {e}"))?;
    if release.draft || release.prerelease {
        return Ok(None);
    }
    let tag = release.tag_name.trim().to_string();
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    if version.is_empty() || !is_newer(&version, current) {
        return Ok(None);
    }
    let assets: Vec<Asset> = release
        .assets
        .into_iter()
        .map(|asset| Asset {
            name: asset.name,
            url: asset.browser_download_url,
            size: asset.size,
        })
        .collect();
    let asset = select_platform_asset(&assets);
    let sums_url = assets
        .iter()
        .find(|asset| asset.name == "SHA256SUMS.txt")
        .map(|asset| asset.url.clone());
    Ok(Some(ReleaseInfo {
        version,
        tag,
        page_url: release.html_url,
        notes: release.body.trim().to_string(),
        published_at: release.published_at,
        asset,
        sums_url,
    }))
}

/// Consulta GitHub. Es una operación de red: la UI la llama solo desde un hilo
/// y a pedido del usuario.
pub fn check_latest() -> Result<Option<ReleaseInfo>, String> {
    let bytes = http_get(RELEASES_URL)?;
    let text = String::from_utf8_lossy(&bytes);
    parse_release(&text, current_version())
}

/// Descarga el asset y su checksum, verifica SHA-256 y guarda el archivo en
/// `dir` con un temporal + rename. No lo ejecuta ni lo instala.
pub fn download_release(release: &ReleaseInfo, dir: &Path) -> Result<PathBuf, String> {
    let asset = release
        .asset
        .as_ref()
        .ok_or_else(|| "No hay binario para esta plataforma.".to_string())?;
    let sums_url = release
        .sums_url
        .as_ref()
        .ok_or_else(|| "El release no publica SHA256SUMS.txt.".to_string())?;
    let bytes = http_get(&asset.url)?;
    let sums = http_get(sums_url)?;
    let sums = String::from_utf8_lossy(&sums);
    let expected = expected_sha(&sums, &asset.name)
        .ok_or_else(|| "No encontré el checksum del binario en SHA256SUMS.txt.".to_string())?;
    verify_bytes(&bytes, &expected)?;

    fs::create_dir_all(dir).map_err(|e| format!("No pude crear {}: {e}", dir.display()))?;
    let file_name = format!("candil-{}-{}", release.version, asset.name);
    let path = dir.join(file_name);
    let temp = dir.join(format!(
        ".candil-update-{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    if let Err(error) = fs::write(&temp, &bytes) {
        let _ = fs::remove_file(&temp);
        return Err(format!("No pude guardar la descarga: {error}"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(error) = fs::set_permissions(&temp, fs::Permissions::from_mode(0o755)) {
            let _ = fs::remove_file(&temp);
            return Err(format!("No pude hacer ejecutable el archivo: {error}"));
        }
    }
    if path.exists() {
        let _ = fs::remove_file(&path);
    }
    if let Err(error) = fs::rename(&temp, &path) {
        let _ = fs::remove_file(&temp);
        return Err(format!("No pude guardar la actualización: {error}"));
    }
    Ok(path)
}

pub fn default_download_dir() -> PathBuf {
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|home| home.join("Downloads")))
        .unwrap_or_else(|| crate::store::data_dir().join("updates"))
}

fn http_get(url: &str) -> Result<Vec<u8>, String> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(TIMEOUT_SECS)))
        .build();
    let agent: ureq::Agent = config.into();
    let mut response = agent
        .get(url)
        .header("User-Agent", &format!("candil/{}", current_version()))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(http_error)?;
    let bytes = response
        .body_mut()
        .read_to_vec()
        .map_err(|e| format!("No pude leer la respuesta: {e}"))?;
    Ok(bytes)
}

fn http_error(error: ureq::Error) -> String {
    match error {
        ureq::Error::StatusCode(403) => {
            "GitHub limitó las consultas. Probá más tarde o abrí la página del release.".into()
        }
        ureq::Error::StatusCode(404) => "No encontré releases publicados.".into(),
        ureq::Error::Timeout(_) => "La consulta tardó demasiado. Probá de nuevo.".into(),
        other => format!("No pude conectar con GitHub: {other}"),
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

pub fn verify_bytes(bytes: &[u8], expected: &str) -> Result<(), String> {
    let actual = sha256_hex(bytes);
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err("El checksum no coincide; no guardé el archivo.".into())
    }
}

/// Busca el hash del asset en `SHA256SUMS.txt` (`<hash>  <nombre>`).
pub fn expected_sha(sums: &str, asset_name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts
            .next()?
            .trim_start_matches('*')
            .trim_start_matches("./");
        (name == asset_name && hash.len() == 64).then(|| hash.to_ascii_lowercase())
    })
}

fn next_minor(version: &str) -> String {
    let mut parts = version.split('.');
    let major = parts.next().unwrap_or("0");
    let minor = parts
        .next()
        .and_then(|part| part.parse::<u64>().ok())
        .unwrap_or(0)
        + 1;
    format!("{major}.{minor}.0")
}

/// Release simulada para `--update-demo`; no toca la red.
pub fn demo_release() -> ReleaseInfo {
    let version = next_minor(current_version());
    let tag = format!("v{version}");
    ReleaseInfo {
        version,
        page_url: format!("https://github.com/quicio/candil/releases/tag/{tag}"),
        tag,
        notes: "Vista previa de la actualización: comprobación manual, descarga \
                verificada con SHA-256 y ningún reemplazo automático."
            .into(),
        published_at: "2026-11-10T00:00:00Z".into(),
        asset: Some(Asset {
            name: platform_asset_names()
                .first()
                .copied()
                .unwrap_or("candil-macos-arm64")
                .into(),
            url: "https://example.com/candil".into(),
            size: 7_250_976,
        }),
        sums_url: Some("https://example.com/SHA256SUMS.txt".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "tag_name": "v0.6.0",
        "html_url": "https://github.com/quicio/candil/releases/tag/v0.6.0",
        "body": "Notas de la versión.",
        "published_at": "2026-11-10T00:00:00Z",
        "draft": false,
        "prerelease": false,
        "assets": [
            {"name":"candil-macos-arm64-app.zip","browser_download_url":"https://example.com/mac-app","size":7240000},
            {"name":"candil-macos-arm64","browser_download_url":"https://example.com/mac","size":7234304},
            {"name":"candil-linux-x86_64","browser_download_url":"https://example.com/linux","size":11806608},
            {"name":"candil-windows-x86_64.exe","browser_download_url":"https://example.com/win","size":10061312},
            {"name":"SHA256SUMS.txt","browser_download_url":"https://example.com/sums","size":263}
        ]
    }"#;

    #[test]
    fn demo_usa_la_siguiente_minor() {
        assert_eq!(next_minor("0.7.0"), "0.8.0");
        assert_eq!(next_minor("1.2.3"), "1.3.0");
    }

    #[test]
    fn compara_versiones_numericas() {
        assert_eq!(compare_versions("0.5.0", "0.5.0"), Ordering::Equal);
        assert_eq!(compare_versions("v0.5.1", "0.5.0"), Ordering::Greater);
        assert_eq!(compare_versions("0.4.9", "0.5.0"), Ordering::Less);
        assert_eq!(compare_versions("0.10.0", "0.9.9"), Ordering::Greater);
        assert_eq!(compare_versions("0.5", "0.5.0"), Ordering::Equal);
        assert!(is_newer("v0.6.0", "0.5.0"));
        assert!(!is_newer("0.5.0", "0.5.0"));
    }

    #[test]
    fn parsea_release_y_conserva_metadatos() {
        let release = parse_release(SAMPLE, "0.5.0").unwrap().expect("nueva");
        assert_eq!(release.version, "0.6.0");
        assert_eq!(release.tag, "v0.6.0");
        assert_eq!(release.notes, "Notas de la versión.");
        assert_eq!(
            release.sums_url.as_deref(),
            Some("https://example.com/sums")
        );
        let expected = platform_asset_names().first().copied();
        assert_eq!(
            release.asset.as_ref().map(|asset| asset.name.as_str()),
            expected,
            "se elige el primer asset disponible de la plataforma"
        );
    }

    #[test]
    fn ignora_versiones_iguales_o_anteriores() {
        assert!(parse_release(SAMPLE, "0.6.0").unwrap().is_none());
        assert!(parse_release(SAMPLE, "1.0.0").unwrap().is_none());
    }

    #[test]
    fn ignora_prerelease_y_draft() {
        let prerelease = SAMPLE.replace("\"prerelease\": false", "\"prerelease\": true");
        assert!(parse_release(&prerelease, "0.5.0").unwrap().is_none());
        let draft = SAMPLE.replace("\"draft\": false", "\"draft\": true");
        assert!(parse_release(&draft, "0.5.0").unwrap().is_none());
    }

    #[test]
    fn selecciona_el_asset_por_nombre_exacto() {
        let assets = vec![
            Asset {
                name: "candil-macos-arm64".into(),
                url: "https://example.com/mac".into(),
                size: 1,
            },
            Asset {
                name: "candil-linux-x86_64".into(),
                url: "https://example.com/linux".into(),
                size: 2,
            },
        ];
        assert_eq!(
            select_asset_for(&assets, "candil-linux-x86_64").map(|asset| asset.size),
            Some(2)
        );
        assert!(select_asset_for(&assets, "candil-macos-x86_64").is_none());
        assert!(select_asset_for(&assets, "").is_none());
    }

    #[test]
    fn verifica_sha256_y_parsea_sums() {
        let empty = sha256_hex(b"");
        assert_eq!(
            empty,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert!(verify_bytes(b"", &empty).is_ok());
        assert!(verify_bytes(b"hola", &empty).is_err());

        let sums = format!("{empty}  candil-macos-arm64\n{empty} *candil-linux-x86_64\n");
        assert_eq!(
            expected_sha(&sums, "candil-macos-arm64").as_deref(),
            Some(empty.as_str())
        );
        assert_eq!(
            expected_sha(&sums, "candil-linux-x86_64").as_deref(),
            Some(empty.as_str())
        );
        assert!(expected_sha(&sums, "otro").is_none());
    }

    #[test]
    fn descarga_verifica_y_guarda_en_disco() {
        use std::io::{Read as _, Write as _};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let addr = listener.local_addr().expect("addr");
        let payload = b"candil-test-binary".to_vec();
        let hash = sha256_hex(&payload);
        let sums = format!("{hash}  candil-test-asset\n");
        let payload_server = payload.clone();
        std::thread::spawn(move || {
            for index in 0..2 {
                let (mut stream, _) = listener.accept().expect("conexión");
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                let body: &[u8] = if index == 0 {
                    &payload_server
                } else {
                    sums.as_bytes()
                };
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(body);
            }
        });

        let release = ReleaseInfo {
            version: "9.9.9".into(),
            tag: "v9.9.9".into(),
            page_url: "https://example.com".into(),
            notes: String::new(),
            published_at: String::new(),
            asset: Some(Asset {
                name: "candil-test-asset".into(),
                url: format!("http://{addr}/asset"),
                size: payload.len() as u64,
            }),
            sums_url: Some(format!("http://{addr}/sums")),
        };
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("candil-update-test-{}-{stamp}", std::process::id()));
        let path = download_release(&release, &dir).expect("descarga");
        assert_eq!(std::fs::read(&path).unwrap(), payload);
        assert!(path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("9.9.9"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o755
            );
        }
        let _ = std::fs::remove_dir_all(dir);
    }
}
