# MSI 安装包：一条命令完成（首次会自动用 winget 安装 WiX，仅需一次）。
# 后端地址由应用运行时读取项目根目录 .env，打包无需注入。
#
# 用法（仓库根目录）：
#   .\app-gpui\scripts\build-msi.ps1
#   .\app-gpui\scripts\build-msi.ps1 -Debug

param(
    [switch]$Debug,
    [switch]$SkipWixInstall
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..\..")

function Find-WixBin {
    if ($env:WIX -and (Test-Path (Join-Path $env:WIX "candle.exe"))) {
        return $env:WIX
    }
    $candidates = @(
        "${env:ProgramFiles(x86)}\WiX Toolset v3.14\bin",
        "${env:ProgramFiles(x86)}\WiX Toolset v3.11\bin",
        "${env:ProgramFiles}\WiX Toolset v3.14\bin",
        "${env:ProgramFiles}\WiX Toolset v3.11\bin"
    )
    foreach ($dir in $candidates) {
        if (Test-Path (Join-Path $dir "candle.exe")) {
            return $dir
        }
    }
    return $null
}

function Ensure-Wix {
    $bin = Find-WixBin
    if ($bin) {
        $env:WIX_PATH = $bin
        $env:PATH = "$bin;$env:PATH"
        return
    }

    if ($SkipWixInstall) {
        throw "未找到 WiX（candle/light）。请安装 WiX Toolset 或使用绿色版：.\app-gpui\scripts\build-portable.ps1"
    }

    if (-not (Get-Command "winget" -ErrorAction SilentlyContinue)) {
        throw @"
未找到 WiX，且本机没有 winget，无法自动安装。
请任选其一：
  1) 安装 WiX：https://wixtoolset.org/docs/wix3/
  2) 安装「应用安装程序」(winget) 后重新运行本脚本
  3) 不打 MSI，改用绿色版：.\app-gpui\scripts\build-portable.ps1
"@
    }

    Write-Host "首次打包 MSI：正在通过 winget 安装 WiX Toolset（只需一次，可能需要管理员确认）…" -ForegroundColor Cyan
    winget install -e --id WiXToolset.WiXToolset --accept-package-agreements --accept-source-agreements
    if ($LASTEXITCODE -ne 0) {
        throw "winget 安装 WiX 失败（退出码 $LASTEXITCODE）。可手动安装 WiX 后重试。"
    }

    $bin = Find-WixBin
    if (-not $bin) {
        throw "WiX 已安装但未在 PATH 中找到 candle.exe。请重新打开终端，或将 WiX 的 bin 目录加入 PATH。"
    }
    $env:WIX_PATH = $bin
    $env:PATH = "$bin;$env:PATH"
    Write-Host "WiX 已就绪: $bin" -ForegroundColor Green
}

if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    throw "未找到 cargo，请先安装 Rust。"
}

Ensure-Wix

if (-not (Get-Command "cargo-wix" -ErrorAction SilentlyContinue)) {
    Write-Host "正在安装 cargo-wix …"
    cargo install cargo-wix --locked
}

Push-Location $Root
try {
    # 简体中文界面：需同时指定 culture 与 wxl 本地化文件，
    # 否则安装向导会按 en-US 回退，中文文案因代码页不符而乱码。
    $LocaleWxl = Join-Path $Root "app-gpui\wix\locales\zh-CN.wxl"
    if (-not (Test-Path $LocaleWxl)) {
        throw "未找到本地化文件：$LocaleWxl"
    }
    $LocaleArgs = @("-c", "zh-CN", "-l", $LocaleWxl)

    if ($Debug) {
        Write-Host "构建 Debug 并生成 MSI …"
        cargo wix -p clx-gpui @LocaleArgs -d --dbg-build --nocapture
    } else {
        if (-not (Get-Command "fxc.exe" -ErrorAction SilentlyContinue)) {
            Write-Host "提示：未找到 fxc.exe，Release 可能失败。可加 -Debug，或安装 Windows SDK。" -ForegroundColor Yellow
        }
        Write-Host "构建 Release 并生成 MSI …"
        cargo wix -p clx-gpui @LocaleArgs --nocapture
    }

    $msiDir = Join-Path $Root "target\wix"
    if (Test-Path $msiDir) {
        Write-Host "`nMSI 输出："
        Get-ChildItem $msiDir -Filter "*.msi" | ForEach-Object { Write-Host "  $($_.FullName)" }
    }
} finally {
    Pop-Location
}
