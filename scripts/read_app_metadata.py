#!/usr/bin/env python3
"""Extrait les métadonnées AltStore d'un Info.plist compilé."""

from __future__ import annotations

import argparse
import json
import plistlib
import re
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("info_plist", type=Path)
    parser.add_argument("--github-output", type=Path, required=True)
    parser.add_argument("--privacy-output", type=Path, required=True)
    args = parser.parse_args()

    with args.info_plist.open("rb") as handle:
        info = plistlib.load(handle)

    required = {
        "bundle_id": "CFBundleIdentifier",
        "version": "CFBundleShortVersionString",
        "build_version": "CFBundleVersion",
    }
    values: dict[str, str] = {}
    for output_name, plist_name in required.items():
        value = info.get(plist_name)
        if not isinstance(value, (str, int)) or not str(value).strip():
            raise SystemExit(f"Clé absente du Info.plist compilé: {plist_name}")
        values[output_name] = str(value)

    minimum = info.get("MinimumOSVersion")
    if minimum:
        values["min_os"] = str(minimum)

    privacy = {
        key: value
        for key, value in info.items()
        if re.fullmatch(r"NS.+UsageDescription", key)
        and isinstance(value, str)
    }
    args.privacy_output.write_text(
        json.dumps(privacy, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )

    with args.github_output.open("a", encoding="utf-8") as handle:
        for key, value in values.items():
            if "\n" in value or "\r" in value:
                raise SystemExit(f"Valeur Info.plist multilignes non prise en charge: {key}")
            handle.write(f"{key}={value}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

