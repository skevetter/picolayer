use anyhow::Result;
use log::info;
use octocrab::models::repos::Asset;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;

pub enum AssetExtractor {
    Archive,
    RawBinary,
}

impl AssetExtractor {
    pub async fn extract(
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

pub fn create_extractor(asset: &Asset) -> AssetExtractor {
    if is_archive(&asset.name.to_lowercase()) {
        AssetExtractor::Archive
    } else {
        AssetExtractor::RawBinary
    }
}

pub async fn extract_and_install(
    asset: &Asset,
    binary_names: &[String],
    bin_location: &str,
) -> Result<()> {
    let extractor = create_extractor(asset);
    extractor.extract(asset, binary_names, bin_location).await
}

async fn download_asset_data(asset: &Asset) -> Result<Vec<u8>> {
    let response = reqwest::get(asset.browser_download_url.clone()).await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to download asset: {}", response.status());
    }
    Ok(response.bytes().await?.to_vec())
}

fn extract_archive(archive_data: &[u8], binary_names: &[String], bin_location: &str) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    if is_tar_xz_archive(archive_data) {
        extract_tar_xz(archive_data, binary_names, bin_location, &temp_dir)
    } else if is_gzip_archive(archive_data) {
        extract_tar_gz(archive_data, binary_names, bin_location, &temp_dir)
    } else {
        anyhow::bail!("Unsupported archive format")
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
    filename.ends_with(".tar.gz")
        || filename.ends_with(".tgz")
        || filename.ends_with(".tar.xz")
        || filename.ends_with(".zip")
        || filename.ends_with(".tar.bz2")
        || filename.ends_with(".7z")
}

fn is_tar_xz_archive(data: &[u8]) -> bool {
    data.len() >= 6 && data[0] == 0xFD && &data[1..6] == b"7zXZ\x00"
}

fn is_gzip_archive(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
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

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if binary_names.iter().any(|name| name == &file_name) {
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

    archive.unpack(&extract_dir)?;
    find_and_install_binaries(&extract_dir, binary_names, bin_location)?;

    Ok(())
}

fn find_and_install_binaries(
    extract_dir: &std::path::Path,
    binary_names: &[String],
    bin_location: &str,
) -> Result<()> {
    for entry in walkdir::WalkDir::new(extract_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_str().unwrap_or("").to_string();

            if binary_names.iter().any(|name| name == &file_name) {
                let source_path = entry.path();
                let dest_path = std::path::Path::new(bin_location).join(&file_name);

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
        }
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
