use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use std::fs::{self, File};
use std::io::copy;
use std::path::Path;
use tokio::runtime::Runtime;

/// Turbo CDN downloader for high-performance downloads
pub struct TurboDownloader {
    runtime: Runtime,
    client: turbo_cdn::TurboCdn,
}

impl TurboDownloader {
    /// Create a new TurboDownloader instance
    pub fn new() -> Result<Self> {
        let runtime = Runtime::new().with_context(|| "Failed to create Tokio runtime")?;

        let client = runtime
            .block_on(async { turbo_cdn::TurboCdn::new().await })
            .with_context(|| "Failed to create TurboCdn client")?;

        Ok(Self { runtime, client })
    }

    /// Download a file from URL to destination path
    pub fn download_file(&self, url: &str, dest: &Path) -> Result<()> {
        info!("Downloading {} to {}", url, dest.display());

        // Create parent directory if it doesn't exist
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Use turbo-cdn smart download with automatic CDN optimization
        let result = self
            .runtime
            .block_on(async { self.client.download_smart_to_path(url, dest).await })
            .with_context(|| format!("Failed to download from {}", url))?;

        info!(
            "Downloaded {} bytes to {} at {:.2} MB/s",
            result.size,
            dest.display(),
            result.speed / 1024.0 / 1024.0
        );

        Ok(())
    }

    /// Get optimized URL for a given URL
    pub fn get_optimized_url(&self, url: &str) -> Result<String> {
        debug!("Getting optimized URL for: {}", url);

        let optimized_url = self
            .runtime
            .block_on(async { self.client.get_optimal_url(url).await })
            .with_context(|| format!("Failed to get optimized URL for {}", url))?;

        debug!("Optimized URL: {}", optimized_url);
        Ok(optimized_url)
    }

    /// Download with progress callback (simplified version)
    pub fn download_with_progress<F>(
        &self,
        url: &str,
        dest: &Path,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(f64) + Send + 'static,
    {
        info!(
            "Downloading {} to {} with progress tracking",
            url,
            dest.display()
        );

        // For now, just call the regular download and simulate progress
        progress_callback(0.0);
        let result = self.download_file(url, dest);
        progress_callback(100.0);

        result
    }
}

/// Fallback download function using reqwest (for compatibility)
pub fn fallback_download_file(url: &str, dest: &Path) -> Result<()> {
    warn!("Using fallback download method for {}", url);

    let client = reqwest::blocking::Client::new();
    let mut response = client
        .get(url)
        .send()
        .with_context(|| format!("Failed to download from {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to download from {}: {}",
            url,
            response.status()
        ));
    }

    let mut file =
        File::create(dest).with_context(|| format!("Failed to create file: {}", dest.display()))?;

    copy(&mut response, &mut file)
        .with_context(|| format!("Failed to write to file: {}", dest.display()))?;

    Ok(())
}

/// Smart download function that tries turbo-cdn first, then falls back to reqwest
pub fn smart_download_file(url: &str, dest: &Path) -> Result<()> {
    // Try turbo-cdn first
    match TurboDownloader::new() {
        Ok(downloader) => match downloader.download_file(url, dest) {
            Ok(()) => {
                debug!("Successfully downloaded using turbo-cdn");
                return Ok(());
            }
            Err(e) => {
                warn!("Turbo-cdn download failed: {}, falling back to reqwest", e);
            }
        },
        Err(e) => {
            warn!("Failed to create turbo downloader: {}, using fallback", e);
        }
    }

    // Fallback to reqwest
    fallback_download_file(url, dest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_turbo_downloader_creation() {
        let result = TurboDownloader::new();
        assert!(result.is_ok(), "Should be able to create TurboDownloader");
    }

    #[test]
    fn test_smart_download_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("test_file.txt");

        // This should work with fallback even if turbo-cdn fails
        let result = smart_download_file("https://httpbin.org/get", &dest);
        // Note: This test might fail in CI without internet access
        // In a real test environment, you'd mock the HTTP calls
        println!("Smart download result: {:?}", result);
    }

    #[test]
    fn test_get_optimized_url() {
        let downloader = TurboDownloader::new().unwrap();
        let test_url =
            "https://github.com/astral-sh/uv/releases/download/0.7.6/uv-x86_64-pc-windows-msvc.zip";

        // This test might fail without internet access
        match downloader.get_optimized_url(test_url) {
            Ok(optimized) => {
                println!("Original: {}", test_url);
                println!("Optimized: {}", optimized);
                assert!(!optimized.is_empty());
            }
            Err(e) => {
                println!("Failed to get optimized URL (expected in CI): {}", e);
            }
        }
    }
}
