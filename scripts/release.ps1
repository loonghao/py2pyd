# Release Helper Script for py2pyd
# This script helps automate the release process

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    
    [Parameter(Mandatory=$false)]
    [string]$Message = "Release version $Version"
)

# Validate version format (semantic versioning)
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Version must be in format x.y.z (e.g., 1.0.0)"
    exit 1
}

Write-Host "🚀 Preparing release for version $Version" -ForegroundColor Green

# Check if we're on main branch
$currentBranch = git rev-parse --abbrev-ref HEAD
if ($currentBranch -ne "main") {
    Write-Warning "You are not on the main branch. Current branch: $currentBranch"
    $continue = Read-Host "Do you want to continue? (y/N)"
    if ($continue -ne "y" -and $continue -ne "Y") {
        Write-Host "Release cancelled." -ForegroundColor Yellow
        exit 0
    }
}

# Check if working directory is clean
$status = git status --porcelain
if ($status) {
    Write-Error "Working directory is not clean. Please commit or stash your changes."
    git status
    exit 1
}

# Update version in Cargo.toml
Write-Host "📝 Updating version in Cargo.toml..." -ForegroundColor Blue
$cargoContent = Get-Content "Cargo.toml"
$cargoContent = $cargoContent -replace 'version = "\d+\.\d+\.\d+"', "version = `"$Version`""
$cargoContent | Set-Content "Cargo.toml"

# Verify the change
$newVersion = (Get-Content "Cargo.toml" | Select-String 'version = "(.+)"').Matches[0].Groups[1].Value
if ($newVersion -ne $Version) {
    Write-Error "Failed to update version in Cargo.toml"
    exit 1
}

Write-Host "✅ Version updated to $newVersion" -ForegroundColor Green

# Build and test locally
Write-Host "🔨 Building and testing locally..." -ForegroundColor Blue
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed. Please fix the issues before releasing."
    exit 1
}

cargo test
if ($LASTEXITCODE -ne 0) {
    Write-Error "Tests failed. Please fix the issues before releasing."
    exit 1
}

Write-Host "✅ Build and tests passed" -ForegroundColor Green

# Commit the version change
Write-Host "📦 Committing version change..." -ForegroundColor Blue
git add Cargo.toml
git commit -m "bump: version $Version

$Message

Signed-off-by: loonghao <hal.long@outlook.com>"

# Push to main branch
Write-Host "🚀 Pushing to main branch..." -ForegroundColor Blue
git push origin main

Write-Host "🎉 Release process initiated!" -ForegroundColor Green
Write-Host ""
Write-Host "The automated release workflow will now:" -ForegroundColor Cyan
Write-Host "  1. Detect the version change" -ForegroundColor Cyan
Write-Host "  2. Build binaries for all platforms" -ForegroundColor Cyan
Write-Host "  3. Create a Git tag (v$Version)" -ForegroundColor Cyan
Write-Host "  4. Create a GitHub release with artifacts" -ForegroundColor Cyan
Write-Host ""
Write-Host "You can monitor the progress at:" -ForegroundColor Yellow
Write-Host "https://github.com/loonghao/py2pyd/actions" -ForegroundColor Yellow
