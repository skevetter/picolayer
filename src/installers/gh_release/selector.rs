use anyhow::{Context, Result};
use octocrab::models::repos::Asset;
use regex::Regex;

pub trait AssetSelector {
    fn select<'a>(&self, assets: &'a [Asset]) -> Result<&'a Asset>;
}

pub struct FilterSelector {
    pattern: String,
}

impl FilterSelector {
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }
}

impl AssetSelector for FilterSelector {
    fn select<'a>(&self, assets: &'a [Asset]) -> Result<&'a Asset> {
        let regex = Regex::new(&self.pattern).context("Invalid filter pattern")?;
        assets
            .iter()
            .find(|a| regex.is_match(&a.name))
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

pub fn create_selector(filter: Option<&str>) -> Box<dyn AssetSelector> {
    match filter {
        Some(pattern) => Box::new(FilterSelector::new(pattern.to_string())),
        None => Box::new(PlatformSelector),
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
        let is_archive_or_binary = is_archive(&name.to_lowercase()) || is_platform_binary(name);

        has_arch && has_os && is_archive_or_binary
    })
}

fn select_any_archive(assets: &[Asset]) -> Option<&Asset> {
    assets
        .iter()
        .find(|a| is_archive(&a.name.to_lowercase()) || is_platform_binary(&a.name))
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

fn is_platform_binary(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    let has_platform_info = lower.contains("linux")
        || lower.contains("darwin")
        || lower.contains("windows")
        || lower.contains("x86_64")
        || lower.contains("amd64")
        || lower.contains("arm64")
        || lower.contains("aarch64");
    let is_not_archive = !is_archive(&lower);
    let is_not_signature = !lower.ends_with(".sig") && !lower.ends_with(".asc");

    has_platform_info && is_not_archive && is_not_signature
}
