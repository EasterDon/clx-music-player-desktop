# Tauri MSI：pnpm tauri build（使用 app-tauri/wix 自定义 WiX 模板）
# 后端地址由 Vite 构建时读取项目根目录 .env（CLX_BASE_URL），无需额外参数。
#
# 用法（仓库根目录）：
#   .\app-tauri\scripts\build-msi.ps1
#   .\app-tauri\scripts\build-msi.ps1 -Debug

param(
    [switch]$Debug
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$AppTauri = Join-Path $Root "app-tauri"
$Version = (Get-Content (Join-Path $AppTauri "package.json") -Raw | ConvertFrom-Json).version
if (-not $Version) { $Version = "0.0.0" }

function Find-MsiOutput {
    param([string]$Profile)
    $dir = Join-Path $AppTauri "src-tauri\target\$Profile\bundle\msi"
    if (-not (Test-Path $dir)) { return @() }
    Get-ChildItem $dir -Filter "*.msi" -ErrorAction SilentlyContinue
}

if (-not (Get-Command "pnpm" -ErrorAction SilentlyContinue)) {
    throw "未找到 pnpm。请先安装 Node.js 与 pnpm。"
}
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    throw "未找到 cargo，请先安装 Rust。"
}

Push-Location $AppTauri
try {
    if (-not (Test-Path "node_modules")) {
        Write-Host "安装前端依赖 …" -ForegroundColor Cyan
        pnpm install
    }

    if ($Debug) {
        Write-Host "构建 Debug MSI（Tauri）…" -ForegroundColor Cyan
        pnpm tauri build --debug
        $profile = "debug"
    } else {
        Write-Host "构建 Release MSI（Tauri）…" -ForegroundColor Cyan
        pnpm tauri build
        $profile = "release"
    }

    $msis = Find-MsiOutput -Profile $profile
    if ($msis.Count -eq 0) {
        throw "未在 src-tauri\target\$profile\bundle\msi 找到 .msi，请检查构建日志。"
    }

    Write-Host "`n完成（版本 $Version）：" -ForegroundColor Green
    foreach ($msi in $msis) {
        Write-Host "  $($msi.FullName)"
    }
    Write-Host "`n提示：同一 UpgradeCode 的 MSI 可直接覆盖安装升级，勿随意修改 tauri.conf.json 中的 upgradeCode。"
} finally {
    Pop-Location
}
