use anyhow::Result;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Test turbo-cdn async API
    let test_url = "https://github.com/astral-sh/uv/releases/download/0.7.6/uv-x86_64-pc-windows-msvc.zip";
    
    println!("Testing turbo-cdn async API...");
    
    // Test URL optimization
    match turbo_cdn::async_api::quick::optimize_url(test_url).await {
        Ok(optimized_url) => {
            println!("✅ URL optimization successful!");
            println!("Original: {}", test_url);
            println!("Optimized: {}", optimized_url);
        }
        Err(e) => {
            println!("❌ URL optimization failed: {}", e);
        }
    }
    
    // Test download
    match turbo_cdn::async_api::quick::download_url(test_url).await {
        Ok(result) => {
            println!("✅ Download successful!");
            println!("Downloaded to: {}", result.path.display());
        }
        Err(e) => {
            println!("❌ Download failed: {}", e);
        }
    }
    
    Ok(())
}
