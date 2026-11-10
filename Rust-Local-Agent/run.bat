@echo off
cd /d "%~dp0"
if exist "target\debug\rust_local_agent.exe" (
    target\debug\rust_local_agent.exe %*
) else (
    echo Building project...
    "%USERPROFILE%\.cargo\bin\cargo.exe" run -- %*
)
