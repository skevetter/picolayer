use anyhow::Result;
use log::info;
use octocrab::models::repos::Release;

use crate::cli::RetryConfig;
use crate::utils::retry::retry_async;

pub(super) async fn fetch_release(
    owner: &str,
    repo: &str,
    version: &str,
    include_prerelease: bool,
    retry_config: &RetryConfig,
) -> Result<Release> {
    let octocrab = if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        octocrab::Octocrab::builder()
            .personal_token(token)
            .build()?
    } else {
        (*octocrab::instance()).clone()
    };

    if version == "latest" {
        if include_prerelease {
            retry_async(
                retry_config,
                "GitHub API - fetch latest release",
                || async { Ok(octocrab.repos(owner, repo).releases().get_latest().await?) },
            )
            .await
        } else {
            let releases =
                retry_async(retry_config, "GitHub API - fetch releases list", || async {
                    Ok(octocrab.repos(owner, repo).releases().list().send().await?)
                })
                .await?;

            let stable_release = releases
                .items
                .into_iter()
                .find(|r| !r.prerelease)
                .ok_or_else(|| anyhow::anyhow!("No stable releases found"))?;

            info!(
                "Skipping prereleases, using stable release: {}",
                stable_release.tag_name
            );
            Ok(stable_release)
        }
    } else {
        retry_async(
            retry_config,
            "GitHub API - fetch release by tag",
            || async {
                Ok(octocrab
                    .repos(owner, repo)
                    .releases()
                    .get_by_tag(version)
                    .await?)
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use octocrab::models::repos::Release;

    fn create_mock_release(tag_name: &str, prerelease: bool) -> Release {
        serde_json::from_value(serde_json::json!({
            "id": 1,
            "node_id": "node123",
            "tag_name": tag_name,
            "target_commitish": "main",
            "name": tag_name,
            "draft": false,
            "prerelease": prerelease,
            "created_at": "2024-01-01T00:00:00Z",
            "published_at": "2024-01-01T00:00:00Z",
            "assets": [],
            "upload_url": "https://example.com/upload",
            "html_url": "https://example.com",
            "assets_url": "https://example.com/assets",
            "tarball_url": "https://example.com/tarball",
            "zipball_url": "https://example.com/zipball",
            "url": "https://example.com/release"
        }))
        .expect("Failed to deserialize mock Release")
    }

    #[test]
    fn test_mock_release_creation() {
        let release = create_mock_release("v1.0.0", false);
        assert_eq!(release.tag_name, "v1.0.0");
        assert!(!release.prerelease);
    }

    #[test]
    fn test_mock_prerelease_creation() {
        let release = create_mock_release("v1.0.0-beta", true);
        assert_eq!(release.tag_name, "v1.0.0-beta");
        assert!(release.prerelease);
    }
}
