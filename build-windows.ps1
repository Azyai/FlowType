$ErrorActionPreference = "Stop"

function Require-Command {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,

        [string]$InstallHint = ""
    )

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        $message = "未找到命令: $Name"
        if ($InstallHint) {
            $message += "`n$InstallHint"
        }

        throw $message
    }
}

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $projectRoot

Write-Host ""
Write-Host "==============================" -ForegroundColor Cyan
Write-Host " FlowType Windows 打包脚本" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host "项目目录: $projectRoot"
Write-Host ""

Require-Command -Name "node" -InstallHint "请先安装 Node.js 20+ 并确保 node/npm 可用。"
Require-Command -Name "npm" -InstallHint "请先安装 Node.js 20+ 并确保 node/npm 可用。"
Require-Command -Name "cargo" -InstallHint "请先安装 Rust 工具链，并确认 cargo 已加入 PATH。"

if (-not (Test-Path "$projectRoot\node_modules")) {
    Write-Host "未检测到 node_modules，开始安装前端依赖..." -ForegroundColor Yellow
    npm install
    if ($LASTEXITCODE -ne 0) {
        throw "npm install 执行失败。"
    }
}

Write-Host "开始构建 Windows 安装包（NSIS）..." -ForegroundColor Green
npm run build:windows
if ($LASTEXITCODE -ne 0) {
    throw "Windows 打包失败，请检查上面的报错信息。"
}

$bundleRoot = Join-Path $projectRoot "src-tauri\target\release\bundle"
$nsisDir = Join-Path $bundleRoot "nsis"
$msiDir = Join-Path $bundleRoot "msi"

Write-Host ""
Write-Host "打包完成。" -ForegroundColor Green

if (Test-Path $nsisDir) {
    Write-Host "NSIS 输出目录: $nsisDir"
}

if (Test-Path $msiDir) {
    Write-Host "MSI 输出目录: $msiDir"
}

if (Test-Path $bundleRoot) {
    Write-Host "正在打开输出目录..." -ForegroundColor Cyan
    Start-Process explorer.exe $bundleRoot
}

Write-Host ""
Write-Host "按任意键退出..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
