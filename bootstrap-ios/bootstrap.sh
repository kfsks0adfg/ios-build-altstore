#!/bin/sh
set -eu
umask 077

BOOTSTRAP_HOME="${IOS_BOOTSTRAP_HOME:-$HOME/.ios-bootstrap}"
STATE_DIR="$BOOTSTRAP_HOME/state"
BIN_DIR="$BOOTSTRAP_HOME/bin"
SIGNER="$BIN_DIR/ios-bootstrap-signer"
PAIRER="$BIN_DIR/ios-bootstrap-pair"
INSTALLER="$BIN_DIR/ios-bootstrap-install"
SIGNED_IPA="$STATE_DIR/signed-bootstrap.ipa"

# SideStore officiel. Aucun binaire tiers n'est réhébergé par ce dépôt.
DEFAULT_IPA_URL="https://github.com/SideStore/SideStore/releases/download/0.7.0-alpha/SideStore.ipa"
DEFAULT_IPA_SHA256="e334f86e6ceeab2e0d886c07611246533f474bb18838cf13ebc09ed1f0d622b5"

usage() {
  echo "Usage: $0 pair | sign [fichier.ipa|URL] | install | status"
}

require_install() {
  if [ ! -x "$SIGNER" ] || [ ! -x "$PAIRER" ] || [ ! -x "$INSTALLER" ]; then
    echo "Bootstrap incomplet. Relance install.sh." >&2
    exit 1
  fi
}

download_ipa() {
  source_value="${1:-$DEFAULT_IPA_URL}"
  target="$STATE_DIR/unsigned-bootstrap.ipa"
  case "$source_value" in
    http://*|https://*)
      expected="${IOS_BOOTSTRAP_IPA_SHA256:-}"
      if [ "$source_value" = "$DEFAULT_IPA_URL" ]; then
        expected="$DEFAULT_IPA_SHA256"
      fi
      if [ -z "$expected" ]; then
        echo "Définis IOS_BOOTSTRAP_IPA_SHA256 pour toute URL personnalisée." >&2
        exit 1
      fi
      curl --fail --location --proto '=https' --tlsv1.2 "$source_value" -o "$target"
      actual="$(sha256sum "$target" | awk '{print $1}')"
      if [ "$actual" != "$expected" ]; then
        echo "Hash IPA incorrect. Attendu $expected, reçu $actual" >&2
        exit 1
      fi
      ;;
    *)
      if [ ! -f "$source_value" ]; then
        echo "IPA introuvable : $source_value" >&2
        exit 1
      fi
      cp "$source_value" "$target"
      ;;
  esac
  echo "$target"
}

pair() {
  require_install
  "$PAIRER" --state-dir "$STATE_DIR"
  if [ -f "$STATE_DIR/device.json" ]; then
    cp "$STATE_DIR/rp-pairing.plist" "$HOME/SideStore.mobiledevicepairing"
    chmod 600 "$HOME/SideStore.mobiledevicepairing"
    echo "Jumelage prêt. Étape suivante : $0 sign"
  fi
}

sign_app() {
  require_install
  if [ ! -f "$STATE_DIR/device.json" ]; then
    echo "Lance d'abord : $0 pair" >&2
    exit 1
  fi
  ipa="$(download_ipa "${1:-}")"
  udid="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["udid"])' "$STATE_DIR/device.json")"
  device_name="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("deviceName") or "iPhone iSH bootstrap")' "$STATE_DIR/device.json")"
  printf "Adresse du compte Apple : "
  IFS= read -r apple_id
  "$SIGNER" \
    --apple-id "$apple_id" \
    --udid "$udid" \
    --device-name "$device_name" \
    --ipa "$ipa" \
    --output "$SIGNED_IPA" \
    --state-dir "$STATE_DIR/private"
  echo "Signature terminée. Active LocalDevVPN, puis lance : $0 install"
}

install_app() {
  require_install
  if [ ! -f "$SIGNED_IPA" ]; then
    echo "Aucun IPA signé. Lance d'abord : $0 sign" >&2
    exit 1
  fi
  "$INSTALLER" --state-dir "$STATE_DIR" --ipa "$SIGNED_IPA"
  echo
  echo "Dans SideStore, importe ensuite le fichier visible dans Fichiers > iSH :"
  echo "  SideStore.mobiledevicepairing"
}

status() {
  echo "Dossier : $BOOTSTRAP_HOME"
  [ -x "$SIGNER" ] && echo "Signeur : prêt" || echo "Signeur : absent"
  [ -f "$STATE_DIR/device.json" ] && echo "iPhone : jumelé" || echo "iPhone : non jumelé"
  [ -f "$SIGNED_IPA" ] && echo "IPA signé : prêt" || echo "IPA signé : absent"
}

mkdir -p "$STATE_DIR"
case "${1:-}" in
  pair) pair ;;
  sign) shift; sign_app "${1:-}" ;;
  install) install_app ;;
  status) status ;;
  *) usage; exit 1 ;;
esac
