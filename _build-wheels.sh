#!/bin/bash
set -eux
ARCH=$(uname -m)

export HOME=/tmp/home
mkdir -p "$HOME"
export CARGO_HOME=/tmp/cargo
export RUSTUP_HOME=/tmp/rustup
RAW=/tmp/wheels-raw
mkdir -p "$RAW" /io/dist

if ! command -v rustc >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
export PATH="$CARGO_HOME/bin:$PATH"

PYBINS=(/opt/python/cp{310,311,312,313,314,315}*/bin)

for PYBIN in "${PYBINS[@]}"; do
    TAG=$(basename "$(dirname "$PYBIN")")  # e.g. cp315-cp315t
    rm -rf /tmp/src && mkdir -p /tmp/src
    tar -C /io --exclude=dist --exclude=target --exclude=build \
        --exclude='*.egg-info' --exclude=__pycache__ --exclude='*.so' \
        -cf - . | tar -C /tmp/src -xf -

    # Each interpreter needs its own target dir: the build venv path is always the same, so with a
    # shared one cargo considers the first build up to date and every wheel gets the same binary.
    export CARGO_TARGET_DIR="/tmp/cargo-target/${TAG}"
    "${PYBIN}/python" -m venv /tmp/venv
    /tmp/venv/bin/pip install -q -U pip
    /tmp/venv/bin/pip wheel /tmp/src -w "$RAW" --no-deps
    rm -rf /tmp/venv
done

for whl in "$RAW"/*-linux_"${ARCH}".whl; do
    auditwheel repair "$whl" -w /io/dist/
done

# verify: every wheel has its own binary, and imports and passes the tests on its interpreter
/opt/python/cp312-cp312/bin/python - "$ARCH" <<'EOF'
import collections, glob, hashlib, sys, zipfile
wheels = collections.defaultdict(list)
for whl in sorted(glob.glob(f'/io/dist/*_{sys.argv[1]}.whl')):
    with zipfile.ZipFile(whl) as z:
        so = next(n for n in z.namelist() if n.endswith('.so'))
        wheels[hashlib.sha256(z.read(so)).hexdigest()].append(whl)
duplicates = [w for w in wheels.values() if len(w) > 1]
if duplicates:
    sys.exit(f'identical binaries in different wheels: {duplicates}')
EOF

for PYBIN in "${PYBINS[@]}"; do
    TAG=$(basename "$(dirname "$PYBIN")")
    WHEELS=(/io/dist/*-"${TAG}"-manylinux*_"${ARCH}".whl)
    [ "${#WHEELS[@]}" -eq 1 ] && [ -f "${WHEELS[0]}" ]
    "${PYBIN}/python" -m venv /tmp/venv-test
    /tmp/venv-test/bin/pip install -q "${WHEELS[0]}" pytest
    # RuntimeWarning as error: a free-threaded build must not re-enable the GIL on import
    (cd /tmp && /tmp/venv-test/bin/python -W error::RuntimeWarning -m pytest -q -p no:cacheprovider /tmp/src/tests)
    rm -rf /tmp/venv-test
done

ls -la /io/dist/