_install-dav1d-prerequisites-linux:
    sudo apt-get update
    sudo apt-get install -y meson ninja-build nasm pkg-config

_install-dav1d-prerequisites-macos:
    brew install meson ninja nasm pkg-config

_install-dav1d-prerequisites-windows:
    python -m pip install --upgrade meson ninja
    winget install --id NASM.NASM --exact --source winget --accept-source-agreements --accept-package-agreements
