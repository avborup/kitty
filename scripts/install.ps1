Write-Host "Installing kitty..."
try {
    $DOWNLOAD_DIR = "$HOME\bin"
    if (-Not (Test-Path ($DOWNLOAD_DIR))) {
        New-Item -ItemType "directory" -Path $DOWNLOAD_DIR | Out-Null
    }
    Invoke-WebRequest -Uri "https://github.com/avborup/kitty/releases/latest/download/kitty-x86_64-pc-windows-msvc.exe" -OutFile "$DOWNLOAD_DIR\kitty.exe"
    Write-Host "Kitty successfully installed to $HOME\bin\kitty.exe"
    Write-Host "If you want to run kitty from anywhere, you should add this directory to your PATH environment variable."
}
catch {
    Write-Host "An error occured while installing kitty: $PSItem"
}

Write-Host "Installing default config file..."
try {
    $CONFIG_PATH = "$env:APPDATA\kitty"
    if (-Not (Test-Path ($CONFIG_PATH))) {
        New-Item -ItemType "directory" -Path $CONFIG_PATH | Out-Null
    }
    Invoke-WebRequest -Uri "https://raw.githubusercontent.com/avborup/kitty/master/kitty.yml" -OutFile "$CONFIG_PATH\kitty.yml"
    Write-Host "Default config file successfully installed to $env:APPDATA\kitty\kitty.yml"
}
catch {
    Write-Host "An error occured while installing the config file: $PSItem"
}