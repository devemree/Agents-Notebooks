param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$ArgsList
)

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

$exePath = Join-Path $scriptDir "target\debug\rust_local_agent.exe"
$cargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"

if (Test-Path $exePath) {
    & $exePath @ArgsList
} elseif (Test-Path $cargoPath) {
    & $cargoPath run -- @ArgsList
} else {
    Write-Error "Neither compiled binary nor cargo.exe was found!"
}
