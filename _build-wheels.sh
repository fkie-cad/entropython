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
tag() { basename "$(dirname "$1")"; }  # e.g. /opt/python/cp315-cp315t/bin -> cp315-cp315t

for PYBIN in "${PYBINS[@]}"; do
    rm -rf /tmp/src && mkdir -p /tmp/src
    tar -C /io --exclude=dist --exclude=target --exclude=build \
        --exclude='*.egg-info' --exclude=__pycache__ --exclude='*.so' \
        -cf - . | tar -C /tmp/src -xf -

    # Each interpreter needs its own target dir: the build venv path is always the same, so with a
    # shared one cargo considers the first build up to date and every wheel gets the same binary.
    export CARGO_TARGET_DIR="/tmp/cargo-target/$(tag "$PYBIN")"
    "${PYBIN}/python" -m venv /tmp/venv
    /tmp/venv/bin/pip install -q -U pip
    /tmp/venv/bin/pip wheel /tmp/src -w "$RAW" --no-deps
    rm -rf /tmp/venv
done

# repair into a separate dir and only copy to /io/dist after verification,
# so that leftover wheels in /io/dist neither break nor fool the checks
REPAIRED=/tmp/wheels-repaired
mkdir -p "$REPAIRED"
for whl in "$RAW"/*-linux_"${ARCH}".whl; do
    auditwheel repair "$whl" -w "$REPAIRED"
done

# verify: every wheel has its own extension binary
"${PYBINS[0]}/python" - "$REPAIRED" <<'EOF'
import collections, hashlib, pathlib, sys, zipfile
wheels = collections.defaultdict(list)
for whl in sorted(pathlib.Path(sys.argv[1]).glob('*.whl')):
    with zipfile.ZipFile(whl) as z:
        extensions = [n for n in z.namelist() if n.rsplit('/', 1)[-1].startswith('libentropython.')]
        if len(extensions) != 1:
            sys.exit(f'{whl.name}: expected one libentropython extension, found {extensions}')
        wheels[hashlib.sha256(z.read(extensions[0])).hexdigest()].append(whl.name)
duplicates = [names for names in wheels.values() if len(names) > 1]
if duplicates:
    sys.exit(f'identical binaries in different wheels: {duplicates}')
EOF

# verify: every wheel passes the tests on its interpreter, and free-threaded builds keep the GIL disabled
for PYBIN in "${PYBINS[@]}"; do
    TAG=$(tag "$PYBIN")
    WHEELS=("$REPAIRED"/*-"${TAG}"-*.whl)
    if [ "${#WHEELS[@]}" -ne 1 ] || [ ! -f "${WHEELS[0]}" ]; then
        echo "expected exactly one wheel for ${TAG}, found: ${WHEELS[*]}" >&2
        exit 1
    fi
    "${PYBIN}/python" -m venv /tmp/venv-test
    /tmp/venv-test/bin/pip install -q "${WHEELS[0]}" pytest
    (cd /tmp && env -u PYTHON_GIL /tmp/venv-test/bin/python -m pytest -q -p no:cacheprovider /tmp/src/tests)
    if [[ "$TAG" == *t ]]; then
        env -u PYTHON_GIL /tmp/venv-test/bin/python -c \
            'import sys, entropython; assert not sys._is_gil_enabled(), "importing entropython re-enabled the GIL"'
    fi
    rm -rf /tmp/venv-test
done

cp "$REPAIRED"/*.whl /io/dist/
ls -la /io/dist/
