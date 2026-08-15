# Tauri 绿色版：复制 release/debug 产物到 dist（无需单独 WiX）
# 后端地址由 Vite 构建时读取项目根目录 .env（CLX_BASE_URL），无需额外参数。
#
# 用法（仓库根目录）：
#   .\app-tauri\scripts\build-portable.ps1
#   .\app-tauri\scripts\build-portable.ps1 -Debug

param(
    [switch]$Debug
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$AppTauri = Join-Path $Root "app-tauri"
$Version = (Get-Content (Join-Path $AppTauri "package.json") -Raw | ConvertFrom-Json).version
if (-not $Version) { $Version = "0.0.0" }

$binaryName = "clx-music-player"
$confPath = Join-Path $AppTauri "src-tauri\tauri.conf.json"
if (Test-Path $confPath) {
    $conf = Get-Content $confPath -Raw | ConvertFrom-Json
    if ($conf.mainBinaryName) {
        $binaryName = $conf.mainBinaryName
    }
}

if (-not (Get-Command "pnpm" -ErrorAction SilentlyContinue)) {
    throw "未找到 pnpm。"
}

Push-Location $AppTauri
try {
    if (-not (Test-Path "node_modules")) {
        Write-Host "安装前端依赖 …"
        pnpm install
    }

    if ($Debug) {
        Write-Host "构建 Debug …"
        pnpm tauri build --debug --no-bundle
        $profile = "debug"
    } else {
        Write-Host "构建 Release …"
        pnpm tauri build --no-bundle
        $profile = "release"
    }

    $exe = Join-Path $AppTauri "src-tauri\target\$profile\$binaryName.exe"
    if (-not (Test-Path $exe)) {
        throw "未找到 $exe"
    }

    $outDir = Join-Path $Root "dist\clx-tauri-$Version-win64-$profile"
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
    Copy-Item $exe (Join-Path $outDir "$binaryName.exe") -Force

    $zip = Join-Path $Root "dist\clx-tauri-$Version-win64-$profile.zip"
    if (Test-Path $zip) { Remove-Item $zip -Force }
    Compress-Archive -Path (Join-Path $outDir "*") -DestinationPath $zip

    Write-Host "`n完成："
    Write-Host "  目录: $outDir"
    Write-Host "  压缩包: $zip"
} finally {
    Pop-Location
}
