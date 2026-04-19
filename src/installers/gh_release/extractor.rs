use anyhow::Result;
use log::{info, warn};
use octocrab::models::repos::Asset;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Component, Path};

enum AssetExtractor {
    Archive,
    RawBinary,
}

impl AssetExtractor {
    async fn extract(
        &self,
        asset: &Asset,
        binary_names: &[String],
        bin_location: &str,
    ) -> Result<()> {
        match self {
            AssetExtractor::Archive => {
                info!("Downloading archive asset");
                let archive_data = download_asset_data(asset).await?;
                info!(
                    "Extracting binaries from archive: {}",
                    binary_names.join(", ")
                );
                extract_archive(&archive_data, binary_names, bin_location)
            }
            AssetExtractor::RawBinary => {
                info!("Downloading raw binary asset");
                let binary_data = download_asset_data(asset).await?;
                info!("Installing raw binary: {}", binary_names.join(", "));
                extract_raw_binary(&binary_data, binary_names, bin_location)
            }
        }
    }
}

fn create_extractor(asset: &Asset) -> AssetExtractor {
    if is_archive(&asset.name.to_lowercase()) {
        AssetExtractor::Archive
    } else {
        AssetExtractor::RawBinary
    }
}

pub(super) async fn extract_and_install(
    asset: &Asset,
    binary_names: &[String],
    bin_location: &str,
) -> Result<()> {
    let extractor = create_extractor(asset);
    extractor.extract(asset, binary_names, bin_location).await
}

const MAX_DOWNLOAD_SIZE: u64 = 500 * 1024 * 1024; // 500MB limit

async fn download_asset_data(asset: &Asset) -> Result<Vec<u8>> {
    let response = reqwest::get(asset.browser_download_url.clone()).await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to download asset: {}", response.status());
    }

    // Check content-length header before downloading the full body
    if let Some(content_length) = response.content_length() {
        if content_length > MAX_DOWNLOAD_SIZE {
            anyhow::bail!(
                "Asset too large: {} bytes (max {} bytes)",
                content_length,
                MAX_DOWNLOAD_SIZE
            );
        }
    }

    let bytes = response.bytes().await?;

    // Also check after downloading in case Content-Length was missing or inaccurate
    if bytes.len() as u64 > MAX_DOWNLOAD_SIZE {
        anyhow::bail!("Downloaded asset exceeds size limit");
    }

    Ok(bytes.to_vec())
}

fn extract_archive(archive_data: &[u8], binary_names: &[String], bin_location: &str) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    if is_tar_xz_archive(archive_data) {
        extract_tar_xz(archive_data, binary_names, bin_location, &temp_dir)
    } else if is_gzip_archive(archive_data) {
        extract_tar_gz(archive_data, binary_names, bin_location, &temp_dir)
    } else {
        anyhow::bail!("Unsupported archive format. Supported formats: tar.gz, tgz, tar.xz")
    }
}

fn extract_raw_binary(
    binary_data: &[u8],
    binary_names: &[String],
    bin_location: &str,
) -> Result<()> {
    fs::create_dir_all(bin_location)?;

    let binary_name = binary_names
        .first()
        .ok_or_else(|| anyhow::anyhow!("No binary name specified for raw binary"))?;

    let dest_path = Path::new(bin_location).join(binary_name);
    fs::write(&dest_path, binary_data)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_path, perms)?;
    }

    info!(
        "Installed raw binary: {} -> {}",
        binary_name,
        dest_path.display()
    );
    Ok(())
}

fn is_archive(filename: &str) -> bool {
    filename.ends_with(".tar.gz") || filename.ends_with(".tgz") || filename.ends_with(".tar.xz")
}

fn is_tar_xz_archive(data: &[u8]) -> bool {
    data.len() >= 6 && data[0] == 0xFD && &data[1..6] == b"7zXZ\x00"
}

fn is_gzip_archive(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}

/// Validates that a tar entry path is safe to extract into the given directory.
/// Returns `true` only if the path contains no `..` components and, once joined
/// with `extract_dir`, stays within `extract_dir`.
fn validate_tar_entry_path(entry_path: &Path, extract_dir: &Path) -> bool {
    // Reject any path that contains a parent-directory component
    for component in entry_path.components() {
        if matches!(component, Component::ParentDir) {
            return false;
        }
    }

    // Ensure the fully-resolved destination stays within extract_dir
    let dest = extract_dir.join(entry_path);
    match dest.canonicalize() {
        Ok(canonical) => canonical.starts_with(extract_dir),
        // The file may not exist yet (we haven't extracted it); fall back to
        // a lexical check on the normalized join.
        Err(_) => {
            // If we can canonicalize the extract_dir itself, do a prefix check
            // on the joined (non-canonicalized) path.
            if let Ok(canonical_base) = extract_dir.canonicalize() {
                // Build the destination without symlink resolution
                let normalized = canonical_base.join(entry_path);
                // Simple prefix check — safe because we already rejected `..`
                normalized.starts_with(&canonical_base)
            } else {
                false
            }
        }
    }
}

fn extract_tar_gz(
    archive_data: &[u8],
    binary_names: &[String],
    bin_location: &str,
    temp_dir: &tempfile::TempDir,
) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let archive_path = temp_dir.path().join("download.tar.gz");
    let mut file = File::create(&archive_path)?;
    file.write_all(archive_data)?;

    let file = File::open(&archive_path)?;
    let reader = BufReader::new(file);
    let decoder = GzDecoder::new(reader);
    let mut archive = Archive::new(decoder);

    fs::create_dir_all(bin_location)?;

    let extract_dir = temp_dir.path();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

        // Skip symlinks to prevent symlink attacks
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            warn!(
                "Skipping symlink/hardlink entry in tar.gz archive: {}",
                path.display()
            );
            continue;
        }

        // Validate path to prevent path traversal
        if !validate_tar_entry_path(&path, extract_dir) {
            warn!("Skipping tar.gz entry with unsafe path: {}", path.display());
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        log::debug!(
            "Processing entry: {} (type: {:?})",
            path.display(),
            entry_type
        );
        if entry_type.is_file() && binary_names.iter().any(|name| name == &file_name) {
            log::debug!("Found matching binary: {} -> {}", file_name, path.display());
            install_binary(&mut entry, &file_name, bin_location)?;
        }
    }

    Ok(())
}

fn extract_tar_xz(
    archive_data: &[u8],
    binary_names: &[String],
    bin_location: &str,
    temp_dir: &tempfile::TempDir,
) -> Result<()> {
    use tar::Archive;
    use xz::read::XzDecoder;

    let extract_dir = temp_dir.path().join("extracted");
    fs::create_dir_all(&extract_dir)?;
    fs::create_dir_all(bin_location)?;

    let cursor = std::io::Cursor::new(archive_data);
    let xz_decoder = XzDecoder::new(cursor);
    let mut archive = Archive::new(xz_decoder);

    // Manually iterate entries instead of archive.unpack() to validate each path
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();

        // Skip symlinks to prevent symlink attacks
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            warn!(
                "Skipping symlink/hardlink entry in tar.xz archive: {}",
                path.display()
            );
            continue;
        }

        // Validate path to prevent path traversal
        if !validate_tar_entry_path(&path, &extract_dir) {
            warn!("Skipping tar.xz entry with unsafe path: {}", path.display());
            continue;
        }

        let dest = extract_dir.join(&path);
        if entry_type.is_dir() {
            fs::create_dir_all(&dest)?;
        } else if entry_type.is_file() {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(&dest)?;
        }
    }

    find_and_install_binaries(&extract_dir, binary_names, bin_location)?;

    Ok(())
}

fn find_and_install_binaries(
    extract_dir: &std::path::Path,
    binary_names: &[String],
    bin_location: &str,
) -> Result<()> {
    use std::collections::HashMap;

    // Collect all matching binaries, preferring shallower paths (root-level)
    let mut best_matches: HashMap<String, (usize, std::path::PathBuf)> = HashMap::new();

    for entry in walkdir::WalkDir::new(extract_dir).max_depth(10) {
        let entry = entry?;
        // Skip symlinks to prevent symlink-following attacks
        if entry.file_type().is_symlink() {
            warn!(
                "Skipping symlink during binary search: {}",
                entry.path().display()
            );
            continue;
        }
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_str().unwrap_or("").to_string();

            if binary_names.iter().any(|name| name == &file_name) {
                // Skip zero-byte files
                if entry.metadata()?.len() == 0 {
                    log::warn!("Skipping zero-byte file: {}", file_name);
                    continue;
                }

                let depth = entry.depth();
                let should_replace = best_matches
                    .get(&file_name)
                    .is_none_or(|(prev_depth, _)| depth < *prev_depth);

                if should_replace {
                    best_matches.insert(file_name, (depth, entry.path().to_path_buf()));
                }
            }
        }
    }

    for (file_name, (_, source_path)) in &best_matches {
        let dest_path = std::path::Path::new(bin_location).join(file_name);

        fs::copy(source_path, &dest_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&dest_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&dest_path, perms)?;
        }

        info!("Installed: {} -> {}", file_name, dest_path.display());
    }

    Ok(())
}

fn install_binary(
    entry: &mut tar::Entry<impl std::io::Read>,
    file_name: &str,
    bin_location: &str,
) -> Result<()> {
    let dest_path = Path::new(bin_location).join(file_name);
    entry.unpack(&dest_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest_path, perms)?;
    }

    info!("Installed: {} -> {}", file_name, dest_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_asset(name: &str) -> Asset {
        serde_json::from_value(serde_json::json!({
            "id": 1,
            "node_id": "n",
            "name": name,
            "content_type": "application/octet-stream",
            "size": 100,
            "download_count": 0,
            "state": "uploaded",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "browser_download_url": format!("https://example.com/{}", name),
            "url": "https://example.com/test"
        }))
        .expect("failed to construct mock Asset")
    }

    // ── is_archive ──────────────────────────────────────────────────────

    #[test]
    fn is_archive_recognises_supported_extensions() {
        assert!(is_archive("tool.tar.gz"));
        assert!(is_archive("tool.tgz"));
        assert!(is_archive("tool.tar.xz"));
    }

    #[test]
    fn is_archive_rejects_unsupported() {
        assert!(!is_archive("tool.zip"));
        assert!(!is_archive("tool.tar.bz2"));
        assert!(!is_archive("tool.7z"));
        assert!(!is_archive("tool.exe"));
        assert!(!is_archive("README.md"));
    }

    // ── is_tar_xz_archive ──────────────────────────────────────────────

    #[test]
    fn is_tar_xz_archive_valid_magic_bytes() {
        let valid: Vec<u8> = vec![0xFD, b'7', b'z', b'X', b'Z', 0x00, 0x01, 0x02];
        assert!(is_tar_xz_archive(&valid));
    }

    #[test]
    fn is_tar_xz_archive_invalid_data() {
        assert!(!is_tar_xz_archive(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]));
        assert!(!is_tar_xz_archive(&[0xFD])); // too short
        assert!(!is_tar_xz_archive(&[])); // empty
    }

    // ── is_gzip_archive ────────────────────────────────────────────────

    #[test]
    fn is_gzip_archive_valid_magic_bytes() {
        let valid: Vec<u8> = vec![0x1f, 0x8b, 0x08, 0x00];
        assert!(is_gzip_archive(&valid));
    }

    #[test]
    fn is_gzip_archive_invalid_data() {
        assert!(!is_gzip_archive(&[0x00, 0x00]));
        assert!(!is_gzip_archive(&[0x1f])); // too short
        assert!(!is_gzip_archive(&[])); // empty
    }

    // ── extract_raw_binary ─────────────────────────────────────────────

    #[test]
    fn extract_raw_binary_writes_file_with_correct_permissions() {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        let bin_dir = tmp.path().join("bin");

        let data = b"#!/bin/sh\necho hello\n";
        let names = vec!["my-tool".to_string()];

        extract_raw_binary(data, &names, bin_dir.to_str().unwrap())
            .expect("extract_raw_binary failed");

        let dest = bin_dir.join("my-tool");
        assert!(dest.exists(), "binary file should exist");

        let contents = fs::read(&dest).expect("failed to read binary");
        assert_eq!(contents, data);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&dest).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o755, "binary should be executable (0755)");
        }
    }

    // ── create_extractor ───────────────────────────────────────────────

    #[test]
    fn create_extractor_returns_archive_for_archives() {
        let asset = mock_asset("tool-v1.2.3-linux-amd64.tar.gz");
        assert!(
            matches!(create_extractor(&asset), AssetExtractor::Archive),
            "tar.gz asset should produce Archive extractor"
        );
    }

    #[test]
    fn create_extractor_returns_raw_binary_for_non_archives() {
        let asset = mock_asset("tool-v1.2.3-linux-amd64");
        assert!(
            matches!(create_extractor(&asset), AssetExtractor::RawBinary),
            "plain binary asset should produce RawBinary extractor"
        );

        let asset = mock_asset("tool.exe");
        assert!(
            matches!(create_extractor(&asset), AssetExtractor::RawBinary),
            "exe asset should produce RawBinary extractor"
        );
    }

    // ── find_and_install_binaries ──────────────────────────────────────

    #[test]
    fn find_and_install_prefers_shallow_binary() {
        let temp = tempfile::tempdir().unwrap();
        let extract_dir = temp.path().join("extract");
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();

        // Create binary at two depths
        let shallow = extract_dir.join("mytool");
        let deep = extract_dir.join("sub/dir/mytool");
        fs::create_dir_all(deep.parent().unwrap()).unwrap();
        fs::write(&shallow, b"shallow-version").unwrap();
        fs::write(&deep, b"deep-version").unwrap();

        find_and_install_binaries(
            &extract_dir,
            &["mytool".to_string()],
            bin_dir.to_str().unwrap(),
        )
        .unwrap();

        let installed = fs::read(bin_dir.join("mytool")).unwrap();
        assert_eq!(installed, b"shallow-version");
    }
}
