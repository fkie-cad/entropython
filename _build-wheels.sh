#!/bin/bash
set -eux
ARCH=$(uname -m)

# see docs: https://setuptools-rust.readthedocs.io/en/latest/building_wheels.html#building-for-multiple-python-versions
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
export PATH="$HOME/.cargo/bin:$PATH"

for PYBIN in /opt/python/cp{310,311,312,313,314}*/bin; do
    rm -rf /io/build/
    "${PYBIN}/pip" install -U setuptools setuptools-rust wheel
    "${PYBIN}/pip" wheel /io/ -w /io/dist/ --no-deps
done

for whl in /io/dist/*{cp310,cp311,cp312,cp313,cp314}*_"${ARCH}".whl; do
    auditwheel repair "$whl" -w /io/dist/
done

for PYBIN in /opt/python/cp{310,311,312,313,314}*/bin; do
    "${PYBIN}/pip" install entropython -f /io/dist/
done
