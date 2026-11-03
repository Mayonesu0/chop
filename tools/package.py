#!/usr/bin/env python3
"""Writes target/chop-<version>.hnmod, the file Horizon's registry and launcher take.

It holds horizon-native.json (made from compatibility.json) and the bundle's .rs and .toml files, each of which
starts with the copyright notice. LICENSE stays in the repository: the registry's checker refuses any other file in
a native bundle. Pinned mod: no `hooks`, the launcher builds it on
the Horizon commit compatibility.json names. The archive is reproducible (sorted entries, fixed dates and modes):
the same sources always give the same bytes and the same SHA-256. Run from anywhere: python3 tools/package.py
"""
import hashlib
import json
import pathlib
import sys
import tomllib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent


def main() -> int:
    compat = json.loads((ROOT / "compatibility.json").read_text())
    manifests = sorted((ROOT / "mixins").glob("*/horizon-mixin.toml"))
    if len(manifests) != 1:
        sys.exit("a .hnmod holds one bundle: keep one mixins/<id>/horizon-mixin.toml")
    bundle = manifests[0].parent
    mixin = tomllib.loads(manifests[0].read_text())
    mod = mixin["mod"]
    if bundle.name != mod["id"]:
        sys.exit(f"rename mixins/{bundle.name}/ to mixins/{mod['id']}/ (the registry finds it by id)")
    pinned = mixin.get("compatibility", {}).get("horizon_revision")
    if pinned != compat["horizon_revision"]:
        sys.exit(f"horizon-mixin.toml pins {pinned}, compatibility.json {compat['horizon_revision']}")
    native = {
        "schema": 1,
        "id": mod["id"],
        "version": mod["version"],
        "game": compat["game"],
        "horizon": compat["horizon_revision"],
        "mixins": compat["horizon_mixins_revision"],
    }
    files = {"horizon-native.json": (json.dumps(native, indent=2, sort_keys=True) + "\n").encode()}
    for path in sorted(bundle.rglob("*")):
        if path.is_file() and path.suffix in (".rs", ".toml"):
            files[f"mixins/{mod['id']}/{path.relative_to(bundle).as_posix()}"] = path.read_bytes()
    out = ROOT / "target" / f"{mod['id']}-{mod['version']}.hnmod"
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    print(json.dumps({"file": str(out), "size": out.stat().st_size,
                      "sha256": hashlib.sha256(out.read_bytes()).hexdigest()}, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
