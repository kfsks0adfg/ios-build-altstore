#!/bin/sh
set -eu
umask 077

VERSION="bootstrap-v0.1.0"
REPOSITORY="kfsks0adfg/ios-build-altstore"
BOOTSTRAP_HOME="${IOS_BOOTSTRAP_HOME:-$HOME/.ios-bootstrap}"
BIN_DIR="$BOOTSTRAP_HOME/bin"
STATE_DIR="$BOOTSTRAP_HOME/state"
RELEASE_BASE="https://github.com/$REPOSITORY/releases/download/$VERSION"
RAW_BASE="https://raw.githubusercontent.com/$REPOSITORY/$VERSION/bootstrap-ios"
ARCHIVE="ios-bootstrap-i686-unknown-linux-musl.tar.gz"

case "$(uname -m)" in
  i386|i486|i586|i686|x86) ;;
  *)
    echo "Ce paquet vise iSH et son environnement x86 32 bits." >&2
    exit 1
    ;;
esac

echo "Installation des dépendances iSH…"
apk add --no-cache ca-certificates curl tar zip
mkdir -p "$BIN_DIR" "$STATE_DIR"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT INT TERM

curl --fail --location --proto '=https' --tlsv1.2 "$RELEASE_BASE/$ARCHIVE" -o "$tmp_dir/$ARCHIVE"
curl --fail --location --proto '=https' --tlsv1.2 "$RELEASE_BASE/SHA256SUMS" -o "$tmp_dir/SHA256SUMS"
(
  cd "$tmp_dir"
  grep "  $ARCHIVE\$" SHA256SUMS | sha256sum -c -
  tar -xzf "$ARCHIVE"
)
for binary in ios-bootstrap-signer ios-bootstrap-pair ios-bootstrap-install; do
  cp "$tmp_dir/$binary" "$BIN_DIR/$binary"
  chmod 700 "$BIN_DIR/$binary"
done
curl --fail --location --proto '=https' --tlsv1.2 "$RAW_BASE/bootstrap.sh" -o "$BOOTSTRAP_HOME/bootstrap.sh"
chmod 700 "$BOOTSTRAP_HOME/bootstrap.sh"

echo
echo "Installation terminée. Première étape :"
echo "  $BOOTSTRAP_HOME/bootstrap.sh pair"
