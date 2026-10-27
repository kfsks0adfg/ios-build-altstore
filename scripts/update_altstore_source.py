#!/usr/bin/env python3
"""Ajoute ou remplace une version d'app dans une source AltStore Classic."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def words(value: str) -> str:
    """Autorise des fichiers .env simples sans problèmes d'espaces shell."""
    return value.replace("_", " ")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--app-name", required=True)
    parser.add_argument("--bundle-id", required=True)
    parser.add_argument("--developer-name", required=True)
    parser.add_argument("--subtitle", required=True)
    parser.add_argument("--description", required=True)
    parser.add_argument("--icon-url", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--build-version", required=True)
    parser.add_argument("--date", required=True)
    parser.add_argument("--download-url", required=True)
    parser.add_argument("--size", required=True, type=int)
    parser.add_argument("--min-os")
    parser.add_argument("--privacy-file", type=Path)
    parser.add_argument("--entitlement", action="append", default=[])
    parser.add_argument(
        "--max-versions",
        type=int,
        default=0,
        help="0 conserve tout l’historique (valeur par défaut)",
    )
    args = parser.parse_args()

    try:
        data = json.loads(args.source.read_text(encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError) as error:
        raise SystemExit(f"Source AltStore illisible: {error}") from error

    apps = data.setdefault("apps", [])
    data.setdefault("news", [])
    data.setdefault("nsfw", False)
    if not isinstance(apps, list):
        raise SystemExit("La clé apps doit être une liste")

    app = next(
        (item for item in apps if item.get("bundleIdentifier") == args.bundle_id),
        None,
    )
    if app is None:
        app = {"bundleIdentifier": args.bundle_id, "versions": []}
        apps.append(app)

    privacy: dict[str, str] = {}
    if args.privacy_file and args.privacy_file.exists():
        privacy = json.loads(args.privacy_file.read_text(encoding="utf-8"))

    app.update(
        {
            "name": args.app_name,
            "bundleIdentifier": args.bundle_id,
            "developerName": words(args.developer_name),
            "subtitle": words(args.subtitle),
            "localizedDescription": words(args.description),
            "iconURL": args.icon_url,
            "appPermissions": {
                "entitlements": sorted(set(args.entitlement)),
                "privacy": privacy,
            },
        }
    )

    versions = app.setdefault("versions", [])
    versions[:] = [
        item
        for item in versions
        if not (
            str(item.get("version")) == args.version
            and str(item.get("buildVersion")) == args.build_version
        )
    ]
    release = {
        "version": args.version,
        "buildVersion": args.build_version,
        "date": args.date,
        "localizedDescription": f"Build automatique {args.version} ({args.build_version})",
        "downloadURL": args.download_url,
        "size": args.size,
    }
    if args.min_os:
        release["minOSVersion"] = args.min_os
    versions.insert(0, release)
    if args.max_versions > 0:
        del versions[args.max_versions :]

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

