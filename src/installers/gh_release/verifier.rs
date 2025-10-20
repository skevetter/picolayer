use anyhow::{Context, Result};
use log::{info, warn};
use octocrab::models::repos::Asset;
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;

pub async fn verify_with_checksum_text(asset: &Asset, checksum_text: &str) -> Result<()> {
    info!("Verifying asset with provided checksum text");

    let (algorithm, expected_hash) = parse_checksum_text(checksum_text)?;
    let asset_data = download_asset_data(asset).await?;
    let computed_hash = compute_hash(&asset_data, &algorithm)?;

    if computed_hash.eq_ignore_ascii_case(&expected_hash) {
        info!("Checksum verification passed");
        Ok(())
    } else {
        anyhow::bail!(
            "Checksum verification failed!\nExpected: {}\nComputed: {}",
            expected_hash,
            computed_hash
        );
    }
}

pub async fn verify_asset(assets: &[Asset], asset: &Asset, gpg_key: Option<&str>) -> Result<()> {
    info!("Verifying asset");

    if let Some(sig_asset) = find_signature_asset(assets, asset) {
        return verify_gpg_signature(asset, sig_asset, gpg_key).await;
    }

    let checksum_asset = find_checksum_asset(assets, asset)?;
    verify_checksum_file(asset, checksum_asset).await
}

fn parse_checksum_text(checksum_text: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = checksum_text.splitn(2, ':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid checksum text format. Expected 'algorithm:hash'");
    }

    let algorithm = parts[0].to_lowercase();
    let hash = parts[1].trim();

    // Validate algorithm and hash length
    match algorithm.as_str() {
        "sha256" if hash.len() == 64 => Ok((algorithm, hash.to_string())),
        "sha512" if hash.len() == 128 => Ok((algorithm, hash.to_string())),
        _ => anyhow::bail!(
            "Unsupported algorithm '{}' or invalid hash length",
            algorithm
        ),
    }
}

fn compute_hash(data: &[u8], algorithm: &str) -> Result<String> {
    match algorithm {
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            Ok(hex::encode(hasher.finalize()))
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            Ok(hex::encode(hasher.finalize()))
        }
        _ => anyhow::bail!("Unsupported hash algorithm: {}", algorithm),
    }
}

fn find_signature_asset<'a>(assets: &'a [Asset], asset: &Asset) -> Option<&'a Asset> {
    let exact_patterns = [format!("{}.asc", asset.name), format!("{}.sig", asset.name)];
    assets.iter().find(|a| exact_patterns.contains(&a.name))
}

fn find_checksum_asset<'a>(assets: &'a [Asset], asset: &Asset) -> Result<&'a Asset> {
    let patterns = build_checksum_patterns(&asset.name);

    assets
        .iter()
        .find(|a| patterns.iter().any(|p| a.name.eq_ignore_ascii_case(p)))
        .context("No checksum file found")
}

fn build_checksum_patterns(filename: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    let variants = get_filename_variants(filename);

    for variant in &variants {
        patterns.extend([
            format!("{}.sha256", variant),
            format!("{}.sha256sum", variant),
            format!("{}.sha512", variant),
            format!("{}.sha512sum", variant),
        ]);
    }

    // Common checksum file names (prioritized order)
    patterns.extend([
        "SHA256SUMS".to_string(),
        "sha256sums.txt".to_string(),
        "checksums.txt".to_string(),
        "CHECKSUMS".to_string(),
        "checksums.sha256".to_string(),
        "SHA512SUMS".to_string(),
        "checksums.sha512".to_string(),
    ]);

    patterns
}

async fn verify_gpg_signature(
    asset: &Asset,
    signature_asset: &Asset,
    gpg_key: Option<&str>,
) -> Result<()> {
    if let Some(key_content) = gpg_key {
        info!("Verifying GPG signature");

        let (asset_data, sig_data, public_key) = tokio::try_join!(
            download_asset_data(asset),
            download_asset_data(signature_asset),
            load_public_key(key_content)
        )?;

        use pgp::composed::{Deserializable, DetachedSignature};
        use std::io::Cursor;

        let signature = if sig_data.starts_with(b"-----BEGIN PGP SIGNATURE-----") {
            let sig_str = String::from_utf8(sig_data)?;
            let (sig, _) = DetachedSignature::from_string(&sig_str)?;
            sig
        } else {
            DetachedSignature::from_bytes(Cursor::new(&sig_data[..]))?
        };

        signature.verify(&public_key, &asset_data[..])?;
        info!("GPG signature verification passed!");
        Ok(())
    } else {
        warn!("Found signature file but no GPG key provided");
        info!("Use --gpg-key option to enable GPG verification");
        Ok(())
    }
}

async fn load_public_key(key_content: &str) -> Result<pgp::composed::SignedPublicKey> {
    use pgp::composed::{Deserializable, SignedPublicKey};

    let key_data = if key_content.starts_with("http://") || key_content.starts_with("https://") {
        info!("Downloading GPG public key from URL");
        reqwest::get(key_content).await?.text().await?
    } else if std::path::Path::new(key_content).exists() {
        tokio::fs::read_to_string(key_content).await?
    } else {
        key_content.to_string()
    };

    let (public_key, _) = SignedPublicKey::from_string(&key_data)?;
    Ok(public_key)
}

async fn verify_checksum_file(asset: &Asset, checksum_asset: &Asset) -> Result<()> {
    info!("Verifying checksum from file: {}", checksum_asset.name);

    let (asset_data, checksum_content) = tokio::try_join!(
        download_asset_data(asset),
        download_asset_text(checksum_asset)
    )?;

    let checksums = parse_checksum_file(&checksum_content)?;
    let asset_variants = get_filename_variants(&asset.name);

    for variant in &asset_variants {
        if let Some((algorithm, expected_hash)) = checksums.get(variant) {
            let computed_hash = compute_hash(&asset_data, algorithm)?;

            if computed_hash.eq_ignore_ascii_case(expected_hash) {
                info!("Checksum verification passed ({})", algorithm);
                return Ok(());
            } else {
                anyhow::bail!(
                    "Checksum verification failed!\nFile: {}\nAlgorithm: {}\nExpected: {}\nComputed: {}",
                    variant,
                    algorithm,
                    expected_hash,
                    computed_hash
                );
            }
        }
    }

    anyhow::bail!("No matching checksum found for asset: {}", asset.name)
}

fn parse_checksum_file(content: &str) -> Result<HashMap<String, (String, String)>> {
    let mut checksums = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((hash, filename)) = parse_checksum_line_format(line) {
            let algorithm = detect_algorithm_from_hash(&hash);
            checksums.insert(filename, (algorithm, hash));
        }
    }

    if checksums.is_empty() {
        anyhow::bail!("No valid checksums found in file");
    }

    Ok(checksums)
}

fn parse_checksum_line_format(line: &str) -> Option<(String, String)> {
    if let Some((filename, hash)) = line.split_once(':') {
        let filename = filename.trim();
        let hash = hash.trim();
        if !hash.is_empty() && !filename.is_empty() {
            return Some((hash.to_string(), filename.to_string()));
        }
    }

    if let Some((hash, rest)) = line.split_once(char::is_whitespace) {
        let filename = rest.trim_start_matches('*').trim();
        if !hash.is_empty() && !filename.is_empty() {
            return Some((hash.to_string(), filename.to_string()));
        }
    }

    None
}

fn detect_algorithm_from_hash(hash: &str) -> String {
    match hash.len() {
        64 => "sha256".to_string(),
        128 => "sha512".to_string(),
        _ => "sha256".to_string(), // Default fallback
    }
}

async fn download_asset_data(asset: &Asset) -> Result<Vec<u8>> {
    let response = reqwest::get(asset.browser_download_url.clone()).await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to download asset: {}", response.status());
    }
    Ok(response.bytes().await?.to_vec())
}

async fn download_asset_text(asset: &Asset) -> Result<String> {
    let response = reqwest::get(asset.browser_download_url.clone()).await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to download asset: {}", response.status());
    }
    Ok(response.text().await?)
}

fn get_filename_variants(filename: &str) -> Vec<String> {
    let compression_extensions = [
        ".tar.gz",
        ".tgz",
        ".tar.xz",
        ".txz",
        ".tar.bz2",
        ".tbz2",
        ".tar.Z",
        ".tar.lz",
        ".tar.lzma",
        ".zip",
        ".gz",
        ".xz",
        ".bz2",
        ".Z",
        ".lz",
        ".lzma",
    ];

    let mut variants = vec![filename.to_string()];
    let mut base_name = filename;

    for ext in &compression_extensions {
        if filename.ends_with(ext) {
            base_name = filename.strip_suffix(ext).unwrap();
            break;
        }
    }

    if base_name != filename {
        variants.push(base_name.to_string());
    }

    variants
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test hash for "hello" - SHA-256
    const TEST_HELLO_SHA256: &str =
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

    #[test]
    fn test_parse_checksum_text_valid() {
        let checksum_text = format!("sha256:{}", TEST_HELLO_SHA256);
        let result = parse_checksum_text(&checksum_text).unwrap();
        assert_eq!("sha256", result.0);
        assert_eq!(TEST_HELLO_SHA256, result.1);
    }

    #[test]
    fn test_parse_checksum_text_invalid_format() {
        let result = parse_checksum_text("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_checksum_text_invalid_length() {
        let result = parse_checksum_text("sha256:short");
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_hash_sha256() {
        let data = b"hello world";
        let result = compute_hash(data, "sha256").unwrap();
        assert_eq!(TEST_HELLO_SHA256, result);
    }

    #[test]
    fn test_compute_hash_unsupported() {
        let data = b"hello world";
        let result = compute_hash(data, "unsupported_algorithm");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_algorithm_from_hash() {
        assert_eq!(
            detect_algorithm_from_hash("a".repeat(64).as_str()),
            "sha256"
        );
        assert_eq!(
            detect_algorithm_from_hash("a".repeat(128).as_str()),
            "sha512"
        );
        assert_eq!(detect_algorithm_from_hash("short"), "sha256");
    }

    #[test]
    fn test_parse_checksum_line_format_space_separated() {
        let result = parse_checksum_line_format("abc123  filename.tar.gz").unwrap();
        assert_eq!(result.0, "abc123");
        assert_eq!(result.1, "filename.tar.gz");
    }

    #[test]
    fn test_parse_checksum_line_format_with_asterisk() {
        let result = parse_checksum_line_format("abc123 *filename.tar.gz").unwrap();
        assert_eq!(result.0, "abc123");
        assert_eq!(result.1, "filename.tar.gz");
    }

    #[test]
    fn test_parse_checksum_line_format_colon_separated() {
        let result = parse_checksum_line_format("filename.tar.gz: abc123").unwrap();
        assert_eq!(result.0, "abc123"); // hash
        assert_eq!(result.1, "filename.tar.gz"); // filename
    }

    #[test]
    fn test_parse_checksum_file() {
        let content = format!(
            "{}  file1.tar.gz\n{} *file2.zip\n# comment\nfile3.tar.xz: {}",
            TEST_HELLO_SHA256, TEST_HELLO_SHA256, TEST_HELLO_SHA256
        );
        let result = parse_checksum_file(&content).unwrap();

        assert_eq!(3, result.len());
        assert_eq!(
            Some(&("sha256".to_string(), TEST_HELLO_SHA256.to_string())),
            result.get("file1.tar.gz")
        );
        assert_eq!(
            Some(&("sha256".to_string(), TEST_HELLO_SHA256.to_string())),
            result.get("file2.zip")
        );
        assert_eq!(
            Some(&("sha256".to_string(), TEST_HELLO_SHA256.to_string())),
            result.get("file3.tar.xz")
        );
    }

    #[test]
    fn test_get_filename_variants() {
        let variants = get_filename_variants("app.tar.gz");
        assert!(variants.contains(&"app.tar.gz".to_string()));
        assert!(variants.contains(&"app".to_string()));
    }

    #[test]
    fn test_build_checksum_patterns() {
        let patterns = build_checksum_patterns("app.tar.gz");
        assert!(patterns.contains(&"app.tar.gz.sha256".to_string()));
        assert!(patterns.contains(&"app.sha256".to_string()));
        assert!(patterns.contains(&"SHA256SUMS".to_string()));
    }
}
