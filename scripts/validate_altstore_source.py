#!/usr/bin/env python3
"""Validation structurelle minimale du schéma AltStore Classic documenté."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from urllib.parse import urlparse


def require(mapping: dict, keys: set[str], context: str) -> None:
    missing = sorted(key for key in keys if key not in mapping)
    if missing:
        raise ValueError(f"{context}: clés absentes: {', '.join(missing)}")


def https_url(value: object, context: str) -> None:
    parsed = urlparse(str(value))
    if parsed.scheme != "https" or not parsed.netloc:
        raise ValueError(f"{context}: URL HTTPS invalide: {value}")


def validate(path: Path, allow_empty: bool) -> None:
    data = json.loads(path.read_text(encoding="utf-8"))
    require(data, {"name", "apps", "news", "nsfw"}, "source")
    if not isinstance(data["apps"], list):
        raise ValueError("source.apps doit être une liste")
    if not allow_empty and not data["apps"]:
        raise ValueError("source.apps ne doit pas être vide")

    bundle_ids: set[str] = set()
    for index, app in enumerate(data["apps"]):
        context = f"apps[{index}]"
        require(
            app,
            {
                "name",
                "bundleIdentifier",
                "developerName",
                "localizedDescription",
                "iconURL",
                "versions",
                "appPermissions",
            },
            context,
        )
        if app["bundleIdentifier"] in bundle_ids:
            raise ValueError(f"{context}: bundleIdentifier dupliqué")
        bundle_ids.add(app["bundleIdentifier"])
        https_url(app["iconURL"], f"{context}.iconURL")
        permissions = app["appPermissions"]
        require(permissions, {"entitlements", "privacy"}, f"{context}.appPermissions")
        if not isinstance(app["versions"], list) or not app["versions"]:
            raise ValueError(f"{context}.versions doit contenir au moins une version")
        for vindex, version in enumerate(app["versions"]):
            vcontext = f"{context}.versions[{vindex}]"
            require(
                version,
                {"version", "buildVersion", "date", "downloadURL", "size"},
                vcontext,
            )
            https_url(version["downloadURL"], f"{vcontext}.downloadURL")
            if not isinstance(version["size"], int) or version["size"] <= 0:
                raise ValueError(f"{vcontext}.size doit être un entier positif")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("--allow-empty", action="store_true")
    args = parser.parse_args()
    try:
        validate(args.source, args.allow_empty)
    except (OSError, json.JSONDecodeError, ValueError) as error:
        raise SystemExit(f"Source AltStore invalide: {error}") from error
    print(f"Source AltStore valide: {args.source}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

