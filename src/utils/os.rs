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

/// Check if the system is macOS
pub fn is_macos() -> bool {
    std::env::consts::OS == "macos"
}

/// Check if the system is Linux
pub fn is_linux() -> bool {
    std::env::consts::OS == "linux"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_distro_returns_valid_variant() {
        // Should return some valid distro, never panic
        let result = detect_distro();
        assert!(result.is_ok());
    }

    #[test]
    fn is_macos_matches_platform() {
        if cfg!(target_os = "macos") {
            assert!(is_macos());
        } else {
            assert!(!is_macos());
        }
    }

    #[test]
    fn is_linux_matches_platform() {
        if cfg!(target_os = "linux") {
            assert!(is_linux());
        } else {
            assert!(!is_linux());
        }
    }

    #[test]
    fn linux_distro_enum_debug() {
        // Ensure Debug derive works
        let distro = LinuxDistro::Ubuntu;
        assert_eq!(format!("{:?}", distro), "Ubuntu");
    }

    #[test]
    fn linux_distro_enum_eq() {
        // Ensure PartialEq works
        assert_eq!(LinuxDistro::Ubuntu, LinuxDistro::Ubuntu);
        assert_ne!(LinuxDistro::Ubuntu, LinuxDistro::Debian);
        assert_ne!(LinuxDistro::Alpine, LinuxDistro::Other);
    }

    #[test]
    fn debian_like_includes_ubuntu_and_debian() {
        // On this system, verify consistency between functions
        if is_ubuntu() {
            assert!(is_debian_like());
        }
        if is_debian() {
            assert!(is_debian_like());
        }
        if is_alpine() {
            assert!(!is_debian_like());
        }
    }
}
