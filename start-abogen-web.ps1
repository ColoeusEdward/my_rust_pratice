<#
Silently start abogen-web (Flask TTS service) in the background, for use by
hello_cargo's abogen_tts module.

Assumes abogen-web was installed via uv tool / pip, with the executable at
%USERPROFILE%\.local\bin\abogen-web.exe by default.

Usage:
  powershell -File start-abogen-web.ps1
  powershell -File start-abogen-web.ps1 -Port 8808 -ExePath "C:\path\to\abogen-web.exe"
#>

param(
    [int]$Port = 8808,
    [string]$ExePath = "$env:USERPROFILE\.local\bin\abogen-web.exe",
    [string]$LogDir = "$PSScriptRoot\logs",
    [int]$ReadyTimeoutSeconds = 60
)

$ErrorActionPreference = "Stop"

function Test-PortListening {
    param([int]$Port)
    $conn = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
    return $null -ne $conn
}

if (Test-PortListening -Port $Port) {
    Write-Host "abogen-web is already listening on port $Port, skipping startup."
    exit 0
}

if (-not (Test-Path $ExePath)) {
    Write-Error "abogen-web executable not found: $ExePath"
    exit 1
}

if (-not (Test-Path $LogDir)) {
    New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
}

$stdoutLog = Join-Path $LogDir "abogen-web.out.log"
$stderrLog = Join-Path $LogDir "abogen-web.err.log"

$process = Start-Process -FilePath $ExePath `
    -WindowStyle Hidden `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog `
    -PassThru

Write-Host "Started abogen-web (PID $($process.Id)), waiting for port $Port to become ready..."

$deadline = (Get-Date).AddSeconds($ReadyTimeoutSeconds)
while ((Get-Date) -lt $deadline) {
    if (Test-PortListening -Port $Port) {
        Write-Host "abogen-web is ready and listening on port $Port. Log: $stdoutLog"
        exit 0
    }
    if ($process.HasExited) {
        Write-Error "abogen-web process exited (exit code $($process.ExitCode)); check log: $stderrLog"
        exit 1
    }
    Start-Sleep -Seconds 1
}

Write-Warning "Port $Port not ready after $ReadyTimeoutSeconds seconds (model loading can be slow); the process is still running in the background, verify manually later."
