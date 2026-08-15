# Tauri 版一键打包（在仓库根目录执行）
# 后端地址由 Vite 构建时读取项目根目录 .env（CLX_BASE_URL），无需额外参数。
#
#   .\app-tauri\scripts\package.ps1           # 绿色版目录
#   .\app-tauri\scripts\package.ps1 -Msi    # MSI 安装包（支持覆盖升级）
#   .\app-tauri\scripts\package.ps1 -Msi -Debug

param(
    [switch]$Msi,
    [switch]$Debug
)

$scriptDir = $PSScriptRoot
if ($Msi) {
    & (Join-Path $scriptDir "build-msi.ps1") -Debug:$Debug
} else {
    & (Join-Path $scriptDir "build-portable.ps1") -Debug:$Debug
}
