$ErrorActionPreference = "Stop"

Write-Host "Starting server..."
$proc = Start-Process -FilePath "cargo" -ArgumentList "run" -PassThru -WindowStyle Hidden

try {
    Write-Host "Waiting for server to be ready..."
    $ready = $false
    for ($i = 0; $i -lt 30; $i++) {
        try {
            Invoke-WebRequest -Uri "http://127.0.0.1:3000/api-docs/openapi.json" -UseBasicParsing -ErrorAction Stop | Out-Null
            $ready = $true
            break
        } catch {
            Start-Sleep -Seconds 1
        }
    }

    if (-not $ready) {
        Write-Error "Server did not become ready in time."
        exit 1
    }

    Write-Host "Fetching OpenAPI spec..."
    Invoke-WebRequest -Uri "http://127.0.0.1:3000/api-docs/openapi.json" -OutFile "openapi.json" -UseBasicParsing
    Write-Host "Spec written to openapi.json"
} finally {
    Write-Host "Stopping server..."
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}
