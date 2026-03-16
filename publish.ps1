# Script pour publier tous les crates LumenX sur crates.io dans l'ordre des dépendances

$ErrorActionPreference = "Stop"

Write-Host "🚀 Publishing LumenX v0.5.1 crates to crates.io" -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan

# Ordre de publication basé sur les dépendances
$crates = @(
    "lumenx-core"
    "lumenx-detect"
    "lumenx-score"
    "lumenx-report"
    "lumenx-testgen"
    "lumenx-analyze"
    "lumenx-fix"
    "lumenx-cli"
)

$originalDir = Get-Location

foreach ($crate in $crates) {
    Write-Host ""
    Write-Host "📦 Publishing $crate..." -ForegroundColor Yellow
    Write-Host "--------------------------" -ForegroundColor Gray

    $cratePath = "crates\$crate"

    # Vérifier que le dossier existe
    if (-not (Test-Path $cratePath)) {
        Write-Host "❌ Crate directory not found: $cratePath" -ForegroundColor Red
        exit 1
    }

    # Dry-run d'abord
    Write-Host "🔍 Dry-run..." -ForegroundColor Cyan
    Push-Location $cratePath
    $dryRunResult = cargo publish --dry-run -p $crate 2>&1
    Pop-Location

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Dry-run successful for $crate" -ForegroundColor Green

        # Publication réelle
        Write-Host "🚀 Publishing..." -ForegroundColor Cyan
        Push-Location $cratePath
        cargo publish -p $crate
        $exitCode = $LASTEXITCODE
        Pop-Location

        if ($exitCode -eq 0) {
            Write-Host "✅ $crate published successfully!" -ForegroundColor Green

            # Attendre que crates.io indexe
            Write-Host "⏳ Waiting for crates.io to index..." -ForegroundColor Gray
            Start-Sleep -Seconds 5
        } else {
            Write-Host "❌ Failed to publish $crate (exit code: $exitCode)" -ForegroundColor Red
            Write-Host ""
            Write-Host "Already published crates:" -ForegroundColor Yellow
            foreach ($c in $crates) {
                if ($c -eq $crate) {
                    break
                }
                Write-Host "  - $c" -ForegroundColor Gray
            }
            Set-Location $originalDir
            exit 1
        }
    } else {
        Write-Host "❌ Dry-run failed for $crate" -ForegroundColor Red
        Write-Host $dryRunResult -ForegroundColor Red
        Set-Location $originalDir
        exit 1
    }
}

Write-Host ""
Write-Host "================================================" -ForegroundColor Cyan
Write-Host "🎉 All crates published successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Users can now install with:" -ForegroundColor Cyan
Write-Host "  cargo install lumenx-cli" -ForegroundColor White
