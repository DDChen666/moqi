# Build a signed Windows release of Moqi: the installer, its updater
# signature, and the latest.json the app's update check reads. Uploads
# nothing; publishing the GitHub Release is a manual step (docs/發布新版.md).
#
#   .\scripts\windows\release.ps1 -Notes "這一版改了什麼"
#
# The updater key lives outside the repository (default
# %USERPROFILE%\.moqi-signing\moqi-updater.key). Never commit it.
param(
    [string]$Notes = '',
    [string]$KeyPath = (Join-Path $env:USERPROFILE '.moqi-signing\moqi-updater.key')
)

& {
    $ErrorActionPreference = 'Stop'
    $root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
    Push-Location $root
    try {
        if (-not (Test-Path $KeyPath)) { throw "Updater key not found: $KeyPath" }
        . (Join-Path $PSScriptRoot 'env.ps1')

        $env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content $KeyPath -Raw).Trim()
        if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) { $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = '' }
        # tauri.updater.conf.json turns on the updater artifacts (.sig) for
        # this build only, so everyday builds don't need the key.
        bun run tauri build --bundles nsis --config src-tauri/tauri.updater.conf.json
        if ($LASTEXITCODE -ne 0) { throw "tauri build failed ($LASTEXITCODE)" }

        $version = (Get-Content src-tauri/tauri.conf.json -Raw | ConvertFrom-Json).version
        $name = "Moqi_${version}_x64-setup.exe"
        $bundle = Join-Path $root "src-tauri\target\release\bundle\nsis"
        $setup = Join-Path $bundle $name
        $sig = "$setup.sig"
        if (-not (Test-Path $sig)) { throw "No signature next to the installer: $sig" }

        $out = Join-Path $root 'dist-release'
        New-Item -ItemType Directory -Force $out | Out-Null
        Copy-Item $setup, $sig $out -Force
        $latest = [ordered]@{
            version   = $version
            notes     = $Notes
            pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
            platforms = [ordered]@{
                'windows-x86_64' = [ordered]@{
                    signature = (Get-Content $sig -Raw).Trim()
                    url       = "https://github.com/DDChen666/moqi/releases/download/v$version/$name"
                }
            }
        }
        $json = $latest | ConvertTo-Json -Depth 5
        [IO.File]::WriteAllText((Join-Path $out 'latest.json'), $json, [Text.UTF8Encoding]::new($false))
        Write-Host "Release files in $out :"
        Get-ChildItem $out | ForEach-Object { Write-Host "  $($_.Name)" }
        Write-Host "Add the macOS entry to latest.json before publishing (docs/發布新版.md)."
    } finally {
        Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
        Pop-Location
    }
}
