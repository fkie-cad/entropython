#!/bin/bash
set -eux

# create release for multiple python versions with manylinux
docker run --rm -v "$(pwd)":/io quay.io/pypa/manylinux2014_x86_64 bash /io/_build-wheels.sh

# also create a source release with build:
python3 -m pip install -U build
python3 -m build --sdist

# finally do a cross build for arm64:
# For this to work, qemu must be registered with docker. You can do so by running
# docker run --rm --privileged multiarch/qemu-user-static --reset -p yes
# (see also: https://github.com/multiarch/qemu-user-static)
docker run --rm -v "$(pwd)":/io quay.io/pypa/manylinux2014_aarch64 bash /io/_build-wheels.sh
