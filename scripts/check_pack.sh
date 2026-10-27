#!/usr/bin/env bash
set -euo pipefail

python3 -m py_compile scripts/*.py
python3 scripts/validate_altstore_source.py docs/altstore.template.json --allow-empty
python3 -m json.tool examples/react-native/HelloReactNative/package.json >/dev/null
python3 -m json.tool examples/react-native/HelloReactNative/app.json >/dev/null
python3 -m json.tool examples/react-native/HelloReactNative/ios/HelloReactNative/Images.xcassets/Contents.json >/dev/null

if command -v node >/dev/null 2>&1; then
  node --check docs/app.js
fi

echo "Contrôles locaux terminés. La compilation Xcode nécessite un runner macOS."

