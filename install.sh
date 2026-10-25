#!/usr/bin/env bash
# Instalador de open-mc-host para Linux.
# Uso: curl -fsSL https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/install.sh | bash
set -euo pipefail

REPO="${OMH_REPO:-IzanLey/OPEN-MINECRAFT-HOST}"
BRANCH="${OMH_BRANCH:-main}"
INSTALL_DIR="$HOME/.local/share/open-mc-host"
BIN_DIR="$HOME/.local/bin"
URL="https://raw.githubusercontent.com/$REPO/$BRANCH/omh.py"

if ! command -v python3 >/dev/null 2>&1; then
    echo "Se necesita Python 3.8 o superior."
    echo "  Debian/Ubuntu: sudo apt install python3"
    echo "  Fedora:        sudo dnf install python3"
    echo "  Arch:          sudo pacman -S python"
    exit 1
fi

if ! python3 -c 'import sys; sys.exit(0 if sys.version_info >= (3, 8) else 1)'; then
    echo "Tu Python es demasiado antiguo. Se necesita Python 3.8 o superior."
    exit 1
fi

mkdir -p "$INSTALL_DIR" "$BIN_DIR"

echo "Descargando open-mc-host desde $URL"
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$URL" -o "$INSTALL_DIR/omh.py"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$INSTALL_DIR/omh.py" "$URL"
else
    echo "Se necesita curl o wget."
    exit 1
fi

cat > "$BIN_DIR/omh" <<EOF
#!/usr/bin/env sh
exec python3 "$INSTALL_DIR/omh.py" "\$@"
EOF
chmod +x "$BIN_DIR/omh" "$INSTALL_DIR/omh.py"

echo "open-mc-host instalado en $INSTALL_DIR"

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
        echo
        echo "Anade $BIN_DIR a tu PATH para usar el comando 'omh':"
        echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
        ;;
esac

python3 "$INSTALL_DIR/omh.py"
