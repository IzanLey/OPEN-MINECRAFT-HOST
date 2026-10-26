#!/usr/bin/env bash
# Instalador de open-mc-host para Linux x86_64.
# Uso: curl -fsSL https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/install.sh | bash
# Version concreta: OMH_VERSION=v0.1.0 bash install.sh
set -euo pipefail

REPO="${OMH_REPO:-IzanLey/OPEN-MINECRAFT-HOST}"
ASSET="omh-linux-x86_64"
BIN_DIR="$HOME/.local/bin"
TARGET="$BIN_DIR/omh"

if [ -n "${OMH_VERSION:-}" ]; then
    BASE_URL="https://github.com/$REPO/releases/download/$OMH_VERSION"
else
    BASE_URL="https://github.com/$REPO/releases/latest/download"
fi
URL="$BASE_URL/$ASSET"
SUMS_URL="$BASE_URL/SHA256SUMS"

ARCH="$(uname -m)"
case "$ARCH" in
    x86_64 | amd64) ;;
    *)
        echo "Solo hay binarios para Linux x86_64 (tu arquitectura es $ARCH)."
        echo "Puedes compilar desde el codigo fuente: cargo build --release"
        exit 1
        ;;
esac

if command -v curl >/dev/null 2>&1; then
    DOWNLOADER="curl"
elif command -v wget >/dev/null 2>&1; then
    DOWNLOADER="wget"
else
    echo "Se necesita curl o wget para descargar open-mc-host."
    echo "  Debian/Ubuntu: sudo apt install curl"
    echo "  Fedora:        sudo dnf install curl"
    echo "  Arch:          sudo pacman -S curl"
    exit 1
fi

download() {
    if [ "$DOWNLOADER" = "curl" ]; then
        curl -fsSL "$1" -o "$2"
    else
        wget -qO "$2" "$1"
    fi
}

# Los temporales se borran pase lo que pase.
TMP_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Descargando open-mc-host desde $URL"
if ! download "$URL" "$TMP_DIR/$ASSET"; then
    echo "No se pudo descargar $URL"
    echo "Comprueba tu conexion y que exista un release con $ASSET."
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    if ! download "$SUMS_URL" "$TMP_DIR/SHA256SUMS"; then
        echo "No se pudo descargar SHA256SUMS, asi que no se puede verificar la descarga."
        exit 1
    fi
    EXPECTED="$(awk -v name="$ASSET" '{ f = $2; sub(/^\*/, "", f); if (f == name) { print $1; exit } }' "$TMP_DIR/SHA256SUMS")"
    if [ -z "$EXPECTED" ]; then
        echo "SHA256SUMS no contiene $ASSET. Instalacion cancelada."
        exit 1
    fi
    ACTUAL="$(sha256sum "$TMP_DIR/$ASSET" | awk '{ print $1 }')"
    if [ "$ACTUAL" != "$EXPECTED" ]; then
        echo "El hash de $ASSET no coincide."
        echo "  esperado: $EXPECTED"
        echo "  obtenido: $ACTUAL"
        exit 1
    fi
    echo "Hash SHA256 verificado."
else
    echo "Aviso: no se encontro sha256sum; no se pudo comprobar la integridad de la descarga."
fi

mkdir -p "$BIN_DIR"
chmod +x "$TMP_DIR/$ASSET"
# mv en un solo paso: si la descarga se corto antes, no queda un binario a medias.
mv -f "$TMP_DIR/$ASSET" "$TARGET"

echo "open-mc-host instalado en $TARGET"

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
        echo
        echo "Anade $BIN_DIR a tu PATH para usar el comando 'omh':"
        echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
        ;;
esac

# Ejecuta el omh recien instalado (sin exec) para mostrar el titulo y la guia.
export PATH="$BIN_DIR:$PATH"
omh
