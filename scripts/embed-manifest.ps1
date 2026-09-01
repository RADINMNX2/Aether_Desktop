<# Embeds the UAC manifest after Tauri links the EXE.
   Tauri already emits VERSION; embedding with winres in build.rs caused
   CVT1100/LNK1123 duplicate VERSION resources. mt.exe adds only the manifest. #>
param(
  [Parameter(Mandatory=$true)][string]$ExePath,
  [string]$ManifestPath = "src-tauri/windows-manifest.xml"
)
$ErrorActionPreference = 'Stop'
$exe = (Resolve-Path $ExePath).Path
$manifest = (Resolve-Path $ManifestPath).Path
$mtPath = $null
$cmd = Get-Command mt.exe -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($cmd) { $mtPath = $cmd.Source }
if (-not $mtPath) {
  $kits = @("${env:ProgramFiles(x86)}\Windows Kits\10\bin", "${env:ProgramFiles}\Windows Kits\10\bin") | Where-Object { $_ -and (Test-Path $_) }
  $found = Get-ChildItem $kits -Filter mt.exe -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.FullName -match '\\(x64|x86)\\mt\.exe$' } | Sort-Object FullName -Descending | Select-Object -First 1
  if ($found) { $mtPath = $found.FullName }
}
if (-not $mtPath -or -not (Test-Path $mtPath)) { throw "mt.exe not found. Install the Windows 10 SDK before embedding the UAC manifest." }
Write-Host "Embedding requireAdministrator manifest into $exe"
& $mtPath -nologo -manifest $manifest "-outputresource:$exe;#1"
if ($LASTEXITCODE -ne 0) { throw "mt.exe failed with exit code $LASTEXITCODE" }
$verify = Join-Path $env:TEMP "aether-manifest-$PID.xml"
try {
  & $mtPath -nologo "-inputresource:$exe;#1" "-out:$verify"
  if ($LASTEXITCODE -ne 0 -or -not (Test-Path $verify)) { throw "The embedded manifest could not be read back." }
  $xml = Get-Content $verify -Raw
  if ($xml -notmatch 'requestedExecutionLevel[^>]+level="requireAdministrator"') { throw "The executable lacks requireAdministrator." }
  if ($xml -notmatch 'Microsoft.Windows.Common-Controls') { throw "The executable lacks the v6 Common-Controls dependency required by TaskDialogIndirect." }
} finally { Remove-Item $verify -Force -ErrorAction SilentlyContinue }
Write-Host "Manifest verification passed."
