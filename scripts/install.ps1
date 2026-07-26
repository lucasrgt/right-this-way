$ErrorActionPreference = "Stop"

$target = "x86_64-pc-windows-msvc"
$destination = if ($env:RTW_INSTALL_DIR) { $env:RTW_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "Programs\rtw\bin" }
$archive = Join-Path $env:TEMP "rtw-$target.zip"
$extract = Join-Path $env:TEMP "rtw-$target"

New-Item -ItemType Directory -Force -Path $destination | Out-Null
Invoke-WebRequest "https://github.com/lucasrgt/right-this-way/releases/latest/download/rtw-$target.zip" -OutFile $archive
Remove-Item -Recurse -Force $extract -ErrorAction SilentlyContinue
Expand-Archive $archive $extract -Force
$binary = Get-ChildItem $extract -Recurse -Filter rtw.exe | Select-Object -First 1
Copy-Item $binary.FullName (Join-Path $destination "rtw.exe") -Force

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($userPath -split ";") -notcontains $destination) {
    [Environment]::SetEnvironmentVariable("Path", (($userPath.TrimEnd(";") + ";" + $destination).TrimStart(";")), "User")
}

Remove-Item -Recurse -Force $extract
Remove-Item -Force $archive
Write-Output "Installed rtw to $destination\rtw.exe"
