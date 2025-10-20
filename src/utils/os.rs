use anyhow::Result;
use std::fs;

#[derive(Debug, PartialEq)]
pub enum LinuxDistro {
    Ubuntu,
    Debian,
    Alpine,
    Other,
}

/// Detect the Linux distribution
pub fn detect_distro() -> Result<LinuxDistro> {
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        let mut kv = std::collections::HashMap::new();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_ascii_uppercase();
                let mut val = line[pos + 1..]
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'')
                    .to_string();
                if key == "ID_LIKE" {
                    val = val.replace(&[',', ';'][..], " ");
                }
                kv.insert(key, val);
            }
        }

        let id = kv.get("ID").map(|s| s.as_str()).unwrap_or_default();
        let id_like = kv.get("ID_LIKE").map(|s| s.as_str()).unwrap_or_default();

        let matches_any = |target: &str| {
            if id.eq_ignore_ascii_case(target) {
                return true;
            }
            id_like
                .split_whitespace()
                .any(|token| token.eq_ignore_ascii_case(target))
        };

        if matches_any("ubuntu") {
            return Ok(LinuxDistro::Ubuntu);
        }
        if matches_any("alpine") {
            return Ok(LinuxDistro::Alpine);
        }
        if matches_any("debian") {
            return Ok(LinuxDistro::Debian);
        }
    }

    if fs::metadata("/etc/alpine-release").is_ok() {
        return Ok(LinuxDistro::Alpine);
    }
    if fs::metadata("/etc/debian_version").is_ok() {
        return Ok(LinuxDistro::Debian);
    }
    if let Ok(contents) = fs::read_to_string("/etc/lsb-release") {
        for line in contents.lines() {
            let line = line.trim();
            if let Some(pos) = line.find('=') {
                let key = &line[..pos];
                let val = line[pos + 1..]
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\'');
                if key == "DISTRIB_ID" && val.eq_ignore_ascii_case("ubuntu") {
                    return Ok(LinuxDistro::Ubuntu);
                }
            }
        }
    }

    Ok(LinuxDistro::Other)
}

/// Check if the system is Ubuntu
pub fn is_ubuntu() -> bool {
    matches!(detect_distro(), Ok(LinuxDistro::Ubuntu))
}

/// Check if the system is Debian-like
pub fn is_debian_like() -> bool {
    matches!(
        detect_distro(),
        Ok(LinuxDistro::Ubuntu) | Ok(LinuxDistro::Debian)
    )
}

/// Check if the system is Alpine
pub fn is_alpine() -> bool {
    matches!(detect_distro(), Ok(LinuxDistro::Alpine))
}

/// Check if the system is Debian
pub fn is_debian() -> bool {
    matches!(detect_distro(), Ok(LinuxDistro::Debian))
}
