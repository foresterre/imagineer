dav1d_version := "1.5.0"
dav1d_repo := "https://code.videolan.org/videolan/dav1d.git"
dav1d_src := justfile_directory() / "target" / "dav1d-src"
dav1d_prefix := justfile_directory() / "target" / "dav1d"

_install-dav1d-linux: _install-dav1d-unix
_install-dav1d-macos: _install-dav1d-unix

_install-dav1d-windows:
    if (-not (Test-Path "{{ dav1d_src }}/.git")) { git clone --depth 1 -b {{ dav1d_version }} {{ dav1d_repo }} "{{ dav1d_src }}" } else { git -C "{{ dav1d_src }}" fetch --depth 1 origin {{ dav1d_version }}; git -C "{{ dav1d_src }}" checkout FETCH_HEAD }
    if (Test-Path "{{ dav1d_src }}/build") { Remove-Item -Recurse -Force "{{ dav1d_src }}/build" }
    $env:PATH = (($env:PATH -split ';') | Where-Object { $_ -notmatch '\\usr\\bin' }) -join ';'; meson setup -Ddefault_library=static --prefix "{{ dav1d_prefix }}" --libdir lib "{{ dav1d_src }}/build" "{{ dav1d_src }}"
    $env:PATH = (($env:PATH -split ';') | Where-Object { $_ -notmatch '\\usr\\bin' }) -join ';'; ninja -C "{{ dav1d_src }}/build"
    meson install -C "{{ dav1d_src }}/build"

_install-dav1d-unix:
    #!/usr/bin/env bash
    set -euo pipefail
    src="{{ dav1d_src }}"
    if [ ! -d "$src/.git" ]; then
        git clone --depth 1 -b {{ dav1d_version }} {{ dav1d_repo }} "$src"
    else
        git -C "$src" fetch --depth 1 origin {{ dav1d_version }}
        git -C "$src" checkout FETCH_HEAD
    fi
    rm -rf "$src/build"
    meson setup -Ddefault_library=static --prefix "{{ dav1d_prefix }}" --libdir lib "$src/build" "$src"
    ninja -C "$src/build"
    meson install -C "$src/build"
    printf 'dav1d installed at %s\n' "{{ dav1d_prefix }}"
    printf 'export PKG_CONFIG_PATH="%s/lib/pkgconfig:$PKG_CONFIG_PATH"\n' "{{ dav1d_prefix }}"
