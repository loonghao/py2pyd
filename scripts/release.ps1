# Semantic Release Helper Script for py2pyd
# This script helps create conventional commits for automatic releases

param(
    [Parameter(Mandatory=$true)]
    [string]$Type,

    [Parameter(Mandatory=$true)]
    [string]$Message,

    [Parameter(Mandatory=$false)]
    [string]$Scope = "",

    [Parameter(Mandatory=$false)]
    [switch]$Breaking
)

# Validate commit type
$ValidTypes = @("feat", "fix", "docs", "style", "refactor", "perf", "test", "chore")
if ($Type -notin $ValidTypes) {
    Write-Error "Invalid commit type '$Type'. Valid types: $($ValidTypes -join ', ')"
    Write-Host ""
    Write-Host "Usage: .\scripts\release.ps1 -Type <type> -Message <message> [-Scope <scope>] [-Breaking]"
    Write-Host ""
    Write-Host "Types:"
    Write-Host "  feat     - New feature (minor version bump)"
    Write-Host "  fix      - Bug fix (patch version bump)"
    Write-Host "  docs     - Documentation changes (patch version bump)"
    Write-Host "  style    - Code style changes (patch version bump)"
    Write-Host "  refactor - Code refactoring (patch version bump)"
    Write-Host "  perf     - Performance improvements (patch version bump)"
    Write-Host "  test     - Test changes (patch version bump)"
    Write-Host "  chore    - Build/tool changes (patch version bump)"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\scripts\release.ps1 -Type feat -Message 'add Python 3.12 support'"
    Write-Host "  .\scripts\release.ps1 -Type fix -Message 'resolve memory leak in parser'"
    Write-Host "  .\scripts\release.ps1 -Type feat -Message 'redesign API' -Breaking"
    Write-Host "  .\scripts\release.ps1 -Type docs -Message 'update installation guide'"
    exit 1
}

Write-Host "🚀 Creating semantic release commit" -ForegroundColor Green

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

# Build commit message
$CommitMsg = ""
if ($Scope) {
    $CommitMsg = "${Type}(${Scope}): ${Message}"
} else {
    $CommitMsg = "${Type}: ${Message}"
}

# Add breaking change indicator
if ($Breaking) {
    if ($Scope) {
        $CommitMsg = "${Type}(${Scope})!: ${Message}"
    } else {
        $CommitMsg = "${Type}!: ${Message}"
    }
}

Write-Host "📝 Commit message: $CommitMsg" -ForegroundColor Blue

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

# Commit the changes
Write-Host "📦 Committing changes..." -ForegroundColor Blue
git add .
if ($Breaking) {
    git commit -m "$CommitMsg

BREAKING CHANGE: $Message

Signed-off-by: loonghao <hal.long@outlook.com>"
} else {
    git commit -m "$CommitMsg

Signed-off-by: loonghao <hal.long@outlook.com>"
}

# Push to main branch
Write-Host "🚀 Pushing to main branch..." -ForegroundColor Blue
git push origin main

Write-Host "🎉 Semantic release commit created!" -ForegroundColor Green
Write-Host ""
Write-Host "The automated semantic release workflow will now:" -ForegroundColor Cyan
Write-Host "  1. Analyze the commit message" -ForegroundColor Cyan
Write-Host "  2. Calculate the new version automatically" -ForegroundColor Cyan
Write-Host "  3. Update Cargo.toml with the new version" -ForegroundColor Cyan
Write-Host "  4. Build binaries for all platforms" -ForegroundColor Cyan
Write-Host "  5. Create a Git tag and GitHub release" -ForegroundColor Cyan
Write-Host ""
Write-Host "Expected version bump based on commit type:" -ForegroundColor Yellow
switch ($Type) {
    "feat" {
        if ($Breaking) {
            Write-Host "  Major version bump (e.g., 0.1.0 → 1.0.0)" -ForegroundColor Yellow
        } else {
            Write-Host "  Minor version bump (e.g., 0.1.0 → 0.2.0)" -ForegroundColor Yellow
        }
    }
    { $_ -in @("fix", "docs", "style", "refactor", "perf", "test", "chore") } {
        Write-Host "  Patch version bump (e.g., 0.1.0 → 0.1.1)" -ForegroundColor Yellow
    }
}
Write-Host ""
Write-Host "You can monitor the progress at:" -ForegroundColor Yellow
Write-Host "https://github.com/loonghao/py2pyd/actions" -ForegroundColor Yellow
