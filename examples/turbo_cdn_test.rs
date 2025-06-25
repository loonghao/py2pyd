use anyhow::Result;
use turbo_cdn::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Test turbo-cdn 0.4.1 API
    let test_url =
        "https://github.com/astral-sh/uv/releases/download/0.7.6/uv-x86_64-pc-windows-msvc.zip";

    println!("Testing turbo-cdn 0.4.1 API...");

    // Create TurboCdn client
    let downloader = TurboCdn::new().await?;

    // Test URL optimization
    match downloader.get_optimal_url(test_url).await {
        Ok(optimized_url) => {
            println!("✅ URL optimization successful!");
            println!("Original: {}", test_url);
            println!("Optimized: {}", optimized_url);
        }
        Err(e) => {
            println!("❌ URL optimization failed: {}", e);
        }
    }

    // Test smart download with automatic CDN optimization
    match downloader.download_smart(test_url).await {
        Ok(result) => {
            println!("✅ Smart download successful!");
            println!("Downloaded to: {}", result.path.display());
            println!("Size: {} bytes", result.size);
            println!("Speed: {:.2} MB/s", result.speed / 1024.0 / 1024.0);
            println!("Duration: {:.2}s", result.duration.as_secs_f64());
            println!("Source URL: {}", result.url);
            if result.resumed {
                println!("Download was resumed");
            }
        }
        Err(e) => {
            println!("❌ Smart download failed: {}", e);
        }
    }

    // Test URL optimization check
    if downloader.can_optimize_url(test_url) {
        println!("✅ URL can be optimized");
    } else {
        println!("ℹ️ URL cannot be optimized");
    }

    // Test getting all available CDN URLs
    match downloader.get_all_cdn_urls(test_url).await {
        Ok(urls) => {
            println!("✅ Available CDN URLs ({}):", urls.len());
            for (i, url) in urls.iter().enumerate() {
                println!("  {}. {}", i + 1, url);
            }
        }
        Err(e) => {
            println!("❌ Failed to get CDN URLs: {}", e);
        }
    }

    Ok(())
}
