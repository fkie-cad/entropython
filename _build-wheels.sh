#!/bin/bash
set -eux
ARCH=$(uname -m)

export HOME=/tmp/home
mkdir -p "$HOME"
export CARGO_HOME=/tmp/cargo
export RUSTUP_HOME=/tmp/rustup
export CARGO_TARGET_DIR=/tmp/cargo-target
RAW=/tmp/wheels-raw
mkdir -p "$RAW" /io/dist

if ! command -v rustc >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
export PATH="$CARGO_HOME/bin:$PATH"

for PYBIN in /opt/python/cp{310,311,312,313,314}*/bin; do
    rm -rf /tmp/src && mkdir -p /tmp/src
    tar -C /io --exclude=dist --exclude=target --exclude=build \
        --exclude='*.egg-info' --exclude=__pycache__ --exclude='*.so' \
        -cf - . | tar -C /tmp/src -xf -

    "${PYBIN}/python" -m venv /tmp/venv
    /tmp/venv/bin/pip install -q -U pip
    /tmp/venv/bin/pip wheel /tmp/src -w "$RAW" --no-deps
    rm -rf /tmp/venv
done

for whl in "$RAW"/*-linux_"${ARCH}".whl; do
    auditwheel repair "$whl" -w /io/dist/
done

ls -la /io/dist/