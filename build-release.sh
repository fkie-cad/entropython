#!/bin/bash
set -eux

rm -rf dist/ build/ target/ *.egg-info

# create release for multiple python versions with manylinux
docker run --rm -u "$(id -u):$(id -g)" -v "$(pwd)":/io \
    quay.io/pypa/manylinux_2_28_x86_64 bash /io/_build-wheels.sh

# also create a source release with build:
uv build --sdist --out-dir dist/

# finally do a cross build for arm64:
# For this to work, qemu must be registered with docker. You can do so by running
# docker run --rm --privileged multiarch/qemu-user-static --reset -p yes
# (see also: https://github.com/multiarch/qemu-user-static)
docker run --rm -u "$(id -u):$(id -g)" -v "$(pwd)":/io \
    quay.io/pypa/manylinux_2_28_aarch64 bash /io/_build-wheels.sh
