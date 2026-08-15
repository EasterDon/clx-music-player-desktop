# 绿色版打包：只需 Rust/Cargo，无需 WiX。输出 zip，解压即用。
# 后端地址由应用运行时读取项目根目录 .env，打包无需注入。
#
# 用法（仓库根目录）：
#   .\app-gpui\scripts\build-portable.ps1
#   .\app-gpui\scripts\build-portable.ps1 -Debug

param(
    [switch]$Debug
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$Gpui = Join-Path $Root "app-gpui"
$Version = (Select-String -Path (Join-Path $Gpui "Cargo.toml") -Pattern '^version\s*=\s*"(.+)"' | ForEach-Object { $_.Matches[0].Groups[1].Value })

if (-not $Version) { $Version = "0.0.0" }

Push-Location $Root
try {
    if ($Debug) {
        Write-Host "构建 Debug …"
        cargo build -p clx-gpui
        $profile = "debug"
    } else {
        if (-not (Get-Command "fxc.exe" -ErrorAction SilentlyContinue)) {
            Write-Host "未找到 fxc.exe，Release 可能失败。可改用 -Debug，或安装 Windows SDK（含 HLSL 编译器）。" -ForegroundColor Yellow
        }
        Write-Host "构建 Release …"
        cargo build --release -p clx-gpui
        $profile = "release"
    }

    $exe = Join-Path $Root "target\$profile\clx-gpui.exe"
    if (-not (Test-Path $exe)) {
        throw "未找到 $exe"
    }

    $outDir = Join-Path $Root "dist\clx-gpui-$Version-win64-$profile"
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
    Copy-Item $exe (Join-Path $outDir "clx-gpui.exe") -Force

    $zip = Join-Path $Root "dist\clx-gpui-$Version-win64-$profile.zip"
    if (Test-Path $zip) { Remove-Item $zip -Force }
    Compress-Archive -Path (Join-Path $outDir "*") -DestinationPath $zip

    Write-Host "`n完成："
    Write-Host "  目录: $outDir"
    Write-Host "  压缩包: $zip"
} finally {
    Pop-Location
}
