use anyhow::{Context, Result};
use octocrab::models::repos::Asset;
use regex::Regex;

pub trait AssetSelector {
    fn select<'a>(&self, assets: &'a [Asset]) -> Result<&'a Asset>;
}

pub struct FilterSelector {
    regex: Regex,
}

impl FilterSelector {
    pub fn new(pattern: &str) -> Result<Self> {
        let regex = Regex::new(pattern).context("Invalid filter pattern")?;
        Ok(Self { regex })
    }
}

impl AssetSelector for FilterSelector {
    fn select<'a>(&self, assets: &'a [Asset]) -> Result<&'a Asset> {
        assets
            .iter()
            .find(|a| self.regex.is_match(&a.name) && !is_signature_file(&a.name))
            .context("No asset matching filter pattern")
    }
}

pub struct PlatformSelector;

impl AssetSelector for PlatformSelector {
    fn select<'a>(&self, assets: &'a [Asset]) -> Result<&'a Asset> {
        select_by_platform(assets)
            .or_else(|| select_any_archive(assets))
            .context("No suitable asset found for this platform")
    }
}

pub fn create_selector(filter: Option<&str>) -> Result<Box<dyn AssetSelector>> {
    match filter {
        Some(pattern) => Ok(Box::new(FilterSelector::new(pattern)?)),
        None => Ok(Box::new(PlatformSelector)),
    }
}

fn select_by_platform(assets: &[Asset]) -> Option<&Asset> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let arch_regex = get_arch_regex(arch)?;
    let os_regex = get_os_regex(os)?;

    assets.iter().find(|asset| {
        let name = &asset.name;
        let has_arch = arch_regex.is_match(name);
        let has_os = os_regex.is_match(name);
        let lower = name.to_lowercase();
        let is_archive_or_binary = is_archive(&lower) || is_platform_binary(&lower);

        has_arch && has_os && is_archive_or_binary
    })
}

fn select_any_archive(assets: &[Asset]) -> Option<&Asset> {
    assets.iter().find(|a| {
        let lower = a.name.to_lowercase();
        is_archive(&lower) || is_platform_binary(&lower)
    })
}

fn get_arch_regex(arch: &str) -> Option<Regex> {
    let pattern = match arch {
        "x86_64" => r"([Aa]md64|\-x64|x64|x86[_-]64)",
        "aarch64" => r"([Aa]rm64|ARM64|[Aa]arch64)",
        "arm" => r"([Aa]rm32|ARM32|[Aa]rmv7)",
        "armv5te" => r"([Aa][Rr][Mm]v5)",
        "armv6" => r"([Aa][Rr][Mm]v6)",
        "armv7" => r"([Aa][Rr][Mm]v7)",
        "i386" => r"(i386|\-386|_386)",
        "i686" => r"(i686|\-686|_686)",
        "s390x" => r"(s390x|s390)",
        "powerpc64" => r"(\-ppc|ppc64|PPC64|_ppc)",
        _ => return None,
    };
    Regex::new(pattern).ok()
}

fn get_os_regex(os: &str) -> Option<Regex> {
    let pattern = match os {
        "linux" => r"([Ll]inux)",
        "macos" => r"([Mm]ac[Oo][Ss]|[Mm]ac\-[Oo][Ss]|\-osx\-|_osx_|[Dd]arwin|\.dmg)",
        "windows" => r"(windows|Windows|WINDOWS|win32|\-win\-|\.msi$|.msixbundle$|\.exe$)",
        "android" => r"([Aa]ndroid)",
        "ios" => r"([Ii][Oo][Ss])",
        "freebsd" => r"([Ff]ree[Bb][Ss][Dd])",
        "netbsd" => r"([Nn]et[Bb][Ss][Dd])",
        "illumos" => r"([Ii]llumos|[Oo]mni[oO][sS]|[Oo]pen[Ii]ndiana|[Tt]ribblix)",
        _ => return None,
    };
    Regex::new(pattern).ok()
}

fn is_archive(filename: &str) -> bool {
    filename.ends_with(".tar.gz")
        || filename.ends_with(".tgz")
        || filename.ends_with(".tar.xz")
        || filename.ends_with(".zip")
        || filename.ends_with(".tar.bz2")
        || filename.ends_with(".7z")
}

fn is_signature_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".asc")
        || lower.ends_with(".sig")
        || lower.ends_with(".sha256")
        || lower.ends_with(".sha256sum")
        || lower.ends_with(".sha512")
        || lower.ends_with(".sha512sum")
        || lower.ends_with(".md5")
        || lower.ends_with(".md5sum")
}

/// Checks whether the filename looks like a platform-specific binary.
/// Expects `lower` to already be lowercased.
fn is_platform_binary(lower: &str) -> bool {
    let has_platform_info = lower.contains("linux")
        || lower.contains("darwin")
        || lower.contains("windows")
        || lower.contains("x86_64")
        || lower.contains("amd64")
        || lower.contains("arm64")
        || lower.contains("aarch64");
    let is_not_archive = !is_archive(lower);
    let is_not_signature = !lower.ends_with(".sig") && !lower.ends_with(".asc");

    has_platform_info && is_not_archive && is_not_signature
}

#[cfg(test)]
mod tests {
    use super::*;
    use octocrab::models::repos::Asset;
    use serde_json;

    /// Helper: build a mock Asset with the given name via JSON deserialization.
    fn mock_asset(name: &str) -> Asset {
        let json = serde_json::json!({
            "id": 1,
            "node_id": "RA_test",
            "name": name,
            "state": "uploaded",
            "content_type": "application/octet-stream",
            "size": 1024,
            "download_count": 0,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "browser_download_url": format!("https://example.com/{name}"),
            "url": format!("https://api.example.com/{name}")
        });
        serde_json::from_value(json).expect("failed to deserialize mock Asset")
    }

    // ── FilterSelector ──────────────────────────────────────────────────

    #[test]
    fn filter_selector_matches_correct_asset() {
        let assets = vec![
            mock_asset("tool-v1.0-linux-amd64.tar.gz"),
            mock_asset("tool-v1.0-darwin-arm64.tar.gz"),
            mock_asset("tool-v1.0-windows-amd64.zip"),
        ];

        let selector = FilterSelector::new("darwin").unwrap();
        let selected = selector.select(&assets).unwrap();
        assert_eq!(selected.name, "tool-v1.0-darwin-arm64.tar.gz");
    }

    #[test]
    fn filter_selector_skips_signature_files() {
        let assets = vec![
            mock_asset("tool-v1.0-linux-amd64.tar.gz.asc"),
            mock_asset("tool-v1.0-linux-amd64.tar.gz.sig"),
            mock_asset("tool-v1.0-linux-amd64.tar.gz.sha256"),
            mock_asset("tool-v1.0-linux-amd64.tar.gz"),
        ];

        let selector = FilterSelector::new("linux").unwrap();
        let selected = selector.select(&assets).unwrap();
        assert_eq!(selected.name, "tool-v1.0-linux-amd64.tar.gz");
    }

    #[test]
    fn filter_selector_error_on_no_match() {
        let assets = vec![
            mock_asset("tool-v1.0-linux-amd64.tar.gz"),
            mock_asset("tool-v1.0-darwin-arm64.tar.gz"),
        ];

        let selector = FilterSelector::new("freebsd").unwrap();
        let result = selector.select(&assets);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No asset matching filter pattern")
        );
    }

    #[test]
    fn filter_selector_error_when_only_signature_files_match() {
        let assets = vec![
            mock_asset("tool-v1.0-linux-amd64.tar.gz.asc"),
            mock_asset("tool-v1.0-linux-amd64.tar.gz.sha256"),
        ];

        let selector = FilterSelector::new("linux").unwrap();
        let result = selector.select(&assets);
        assert!(result.is_err());
    }

    // ── PlatformSelector ────────────────────────────────────────────────

    #[test]
    fn platform_selector_selects_archive_for_current_platform() {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        // Build a name that matches the current platform
        let (arch_token, os_token) = match (arch, os) {
            ("x86_64", "linux") => ("amd64", "linux"),
            ("x86_64", "macos") => ("amd64", "darwin"),
            ("aarch64", "linux") => ("arm64", "linux"),
            ("aarch64", "macos") => ("arm64", "darwin"),
            _ => {
                // Skip on unsupported platforms rather than fail
                eprintln!("Skipping platform_selector test on {os}/{arch}");
                return;
            }
        };

        let expected_name = format!("tool-v1.0-{os_token}-{arch_token}.tar.gz");
        let assets = vec![
            mock_asset("tool-v1.0-checksums.sha256"),
            mock_asset("tool-v1.0-source.tar.gz"), // no platform info, fallback only
            mock_asset(&expected_name),
        ];

        let selector = PlatformSelector;
        let selected = selector.select(&assets).unwrap();
        assert_eq!(selected.name, expected_name);
    }

    #[test]
    fn platform_selector_falls_back_to_any_archive() {
        // Provide assets with no platform-specific names at all
        let assets = vec![mock_asset("tool-v1.0.tar.gz"), mock_asset("README.md")];

        let selector = PlatformSelector;
        let selected = selector.select(&assets).unwrap();
        assert_eq!(selected.name, "tool-v1.0.tar.gz");
    }

    #[test]
    fn platform_selector_error_when_no_suitable_asset() {
        let assets = vec![mock_asset("README.md"), mock_asset("LICENSE")];

        let selector = PlatformSelector;
        let result = selector.select(&assets);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No suitable asset found")
        );
    }

    // ── is_signature_file ───────────────────────────────────────────────

    #[test]
    fn is_signature_file_identifies_known_extensions() {
        assert!(is_signature_file("tool.tar.gz.asc"));
        assert!(is_signature_file("tool.tar.gz.sig"));
        assert!(is_signature_file("tool.tar.gz.sha256"));
        assert!(is_signature_file("tool.tar.gz.sha256sum"));
        assert!(is_signature_file("tool.tar.gz.sha512"));
        assert!(is_signature_file("tool.tar.gz.sha512sum"));
        assert!(is_signature_file("tool.tar.gz.md5"));
        assert!(is_signature_file("tool.tar.gz.md5sum"));
    }

    #[test]
    fn is_signature_file_case_insensitive() {
        assert!(is_signature_file("tool.tar.gz.ASC"));
        assert!(is_signature_file("tool.tar.gz.SHA256"));
        assert!(is_signature_file("tool.tar.gz.SIG"));
    }

    #[test]
    fn is_signature_file_rejects_non_signature() {
        assert!(!is_signature_file("tool.tar.gz"));
        assert!(!is_signature_file("tool.zip"));
        assert!(!is_signature_file("tool-linux-amd64"));
        assert!(!is_signature_file("README.md"));
    }

    // ── is_archive ──────────────────────────────────────────────────────

    #[test]
    fn is_archive_identifies_known_extensions() {
        assert!(is_archive("tool.tar.gz"));
        assert!(is_archive("tool.tgz"));
        assert!(is_archive("tool.tar.xz"));
        assert!(is_archive("tool.zip"));
        assert!(is_archive("tool.tar.bz2"));
        assert!(is_archive("tool.7z"));
    }

    #[test]
    fn is_archive_rejects_non_archive() {
        assert!(!is_archive("tool.exe"));
        assert!(!is_archive("tool.deb"));
        assert!(!is_archive("tool.rpm"));
        assert!(!is_archive("readme.md"));
        assert!(!is_archive("tool.asc"));
    }

    // ── create_selector ─────────────────────────────────────────────────

    #[test]
    fn create_selector_returns_filter_when_pattern_given() {
        let selector = create_selector(Some("linux")).unwrap();
        // Verify it behaves as a FilterSelector by testing with assets
        let assets = vec![
            mock_asset("tool-linux-amd64.tar.gz"),
            mock_asset("tool-darwin-arm64.tar.gz"),
        ];
        let selected = selector.select(&assets).unwrap();
        assert_eq!(selected.name, "tool-linux-amd64.tar.gz");
    }

    #[test]
    fn create_selector_returns_platform_when_no_pattern() {
        let selector = create_selector(None).unwrap();
        // Verify it behaves as a PlatformSelector: falls back to any archive
        let assets = vec![mock_asset("tool.tar.gz")];
        let selected = selector.select(&assets).unwrap();
        assert_eq!(selected.name, "tool.tar.gz");
    }

    #[test]
    fn create_selector_returns_error_for_invalid_regex() {
        let result = create_selector(Some("[invalid"));
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(
            err_msg.contains("Invalid filter pattern"),
            "unexpected error: {err_msg}"
        );
    }
}
