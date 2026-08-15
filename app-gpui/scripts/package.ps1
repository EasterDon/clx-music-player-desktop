# 一键打包入口（在仓库根目录执行）
# 后端地址由应用/构建读取项目根目录 .env，无需额外参数。
#
#   .\app-gpui\scripts\package.ps1              # 绿色版 zip（推荐，无需 WiX）
#   .\app-gpui\scripts\package.ps1 -Msi         # MSI 安装包（首次自动装 WiX）
#   .\app-gpui\scripts\package.ps1 -Debug       # 调试版绿色包
#   .\app-gpui\scripts\package.ps1 -Msi -Debug  # 调试版 MSI

param(
    [switch]$Msi,
    [switch]$Debug,
    [switch]$SkipWixInstall
)

$scriptDir = $PSScriptRoot
if ($Msi) {
    & (Join-Path $scriptDir "build-msi.ps1") -Debug:$Debug -SkipWixInstall:$SkipWixInstall
} else {
    & (Join-Path $scriptDir "build-portable.ps1") -Debug:$Debug
}
