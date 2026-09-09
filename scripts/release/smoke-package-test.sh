#!/usr/bin/env bash
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
make_archive() {
  python3 - "$TMP/$1.tar.gz" "$1" <<'PY'
import hashlib, io, sys, tarfile
out, variant = sys.argv[1:]
root = {
    "root-traversal": "nekomusume-0.1.0-aarch64-unknown-linux-gnu/..",
    "root-slash": "wrapper/nekomusume-0.1.0-aarch64-unknown-linux-gnu",
    "root-prefix": "nekomusume-0.1.0-aarch64-unknown-linux-gnu.evil",
}.get(variant, "nekomusume-0.1.0-aarch64-unknown-linux-gnu")
files = {
    "bin/neko-cli": b"not executed on a non-native target\n",
    "share/doc/nekomusume/LICENSE-APACHE": b"apache\n",
    "share/doc/nekomusume/LICENSE-MIT": b"mit\n",
    "share/doc/nekomusume/README.txt": b"readme\n",
}
checksums = "".join(f"{hashlib.sha256(data).hexdigest()}  ./{name}\n" for name, data in sorted(files.items()))
if variant == "checksum-traversal":
    checksums += f"{hashlib.sha256(b'escape').hexdigest()}  ../escape\n"
elif variant == "checksum-absolute":
    checksums += f"{hashlib.sha256(b'escape').hexdigest()}  /tmp/escape\n"
elif variant == "checksum-duplicate":
    checksums += checksums.splitlines(keepends=True)[0]
elif variant == "checksum-unexpected":
    checksums += f"{hashlib.sha256(b'extra').hexdigest()}  ./extra.txt\n"
files["SHA256SUMS"] = checksums.encode()
dirs = [root, f"{root}/bin", f"{root}/share", f"{root}/share/doc", f"{root}/share/doc/nekomusume"]
with tarfile.open(out, "w:gz") as archive:
    if variant == "bad-dir-mode":
        dirs[1] = (dirs[1], 0o777)
    else:
        dirs = [(name, 0o755) for name in dirs]
    for entry in dirs:
        name, mode = entry if isinstance(entry, tuple) else (entry, 0o755)
        info = tarfile.TarInfo(name)
        info.type = tarfile.DIRTYPE
        info.mode = mode
        archive.addfile(info)
    for relative, data in files.items():
        name = f"{root}/{relative}"
        info = tarfile.TarInfo(name)
        info.mode = 0o755 if relative == "bin/neko-cli" else 0o644
        if variant == "bad-mode" and relative == "share/doc/nekomusume/README.txt":
            info.mode = 0o600
        if variant == "bad-exec-mode" and relative == "bin/neko-cli":
            info.mode = 0o777
        if variant == "bad-doc-mode" and relative == "share/doc/nekomusume/README.txt":
            info.mode = 0o666
        if variant == "bad-checksum-mode" and relative == "SHA256SUMS":
            info.mode = 0o666
        if variant == "bad-dir-mode" and relative == "share/doc/nekomusume/LICENSE-MIT":
            pass
        if variant == "symlink" and relative == "share/doc/nekomusume/README.txt":
            info.type = tarfile.SYMTYPE
            info.linkname = "/etc/passwd"
            archive.addfile(info)
            continue
        if variant == "hardlink" and relative == "share/doc/nekomusume/README.txt":
            info.type = tarfile.LNKTYPE
            info.linkname = f"{root}/share/doc/nekomusume/LICENSE-MIT"
            archive.addfile(info)
            continue
        if variant == "bad-checksum" and relative == "share/doc/nekomusume/README.txt":
            data = b"tampered\n"
        info.size = len(data)
        archive.addfile(info, io.BytesIO(data))
    if variant == "unexpected":
        data = b"sibling must not rescue invalid layout\n"
        info = tarfile.TarInfo(f"{root}/extra.txt")
        info.mode, info.size = 0o644, len(data)
        archive.addfile(info, io.BytesIO(data))
    elif variant == "traversal":
        data = b"escape\n"
        info = tarfile.TarInfo(f"{root}/../escape")
        info.mode, info.size = 0o644, len(data)
        archive.addfile(info, io.BytesIO(data))
    elif variant == "fifo":
        info = tarfile.TarInfo(f"{root}/share/doc/nekomusume/pipe")
        info.type, info.mode = tarfile.FIFOTYPE, 0o644
        archive.addfile(info)
PY
}
expect_reject() {
  make_archive "$1"
  if "$ROOT/scripts/release/smoke-package.sh" "$TMP/$1.tar.gz" >"$TMP/$1.log" 2>&1; then
    echo "package smoke unexpectedly accepted $1" >&2
    exit 1
  fi
}
make_archive valid
"$ROOT/scripts/release/smoke-package.sh" "$TMP/valid.tar.gz" | grep -q 'execution skipped'
for case in root-traversal root-slash root-prefix traversal symlink hardlink fifo unexpected bad-checksum bad-mode bad-exec-mode bad-doc-mode bad-checksum-mode bad-dir-mode checksum-traversal checksum-absolute checksum-duplicate checksum-unexpected; do
  expect_reject "$case"
done
echo package-smoke-regressions-ok
