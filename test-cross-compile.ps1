# Test script for cross-compilation
# Based on rust-actions-toolkit troubleshooting guide

Write-Host "Testing cross-compilation fixes for libmimalloc-sys..." -ForegroundColor Green

# Set environment variables for better error messages
$env:CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_DEBUG = "true"

# Test Windows targets that commonly fail with mimalloc
$targets = @(
    "x86_64-pc-windows-gnu",
    "i686-pc-windows-gnu"
)

foreach ($target in $targets) {
    Write-Host "`nTesting target: $target" -ForegroundColor Yellow
    
    # Check if target is installed
    $installed = rustup target list --installed | Select-String $target
    if (-not $installed) {
        Write-Host "Installing target $target..." -ForegroundColor Cyan
        rustup target add $target
    }
    
    # Test compilation
    Write-Host "Building for $target..." -ForegroundColor Cyan
    $result = cargo build --target $target --release 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ SUCCESS: $target compiled successfully" -ForegroundColor Green
    } else {
        Write-Host "❌ FAILED: $target compilation failed" -ForegroundColor Red
        Write-Host "Error output:" -ForegroundColor Red
        Write-Host $result -ForegroundColor Red
    }
}

Write-Host "`nCross-compilation test completed." -ForegroundColor Green
