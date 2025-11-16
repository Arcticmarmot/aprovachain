#!/usr/bin/env bash
set -euo pipefail

ROCKSDB_VERSION="${ROCKSDB_VERSION:-v10.4.2}"

ARCH="$(uname -m)"


# default prefix dir
LIB_PREFIX="/usr/local/lib"
INCLUDE_PREFIX="/usr/local/include"

echo "=== librocksdb install ==="
echo "version:   ${ROCKSDB_VERSION}"
echo "arch:   ${ARCH}"
echo

# temp dir
WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

cd "$WORKDIR"

echo "1) clone facebook/rocksdb..."
git clone --branch "$ROCKSDB_VERSION" --depth 1 https://github.com/facebook/rocksdb.git
cd rocksdb

echo "2) compile shared lib (DEBUG_LEVEL=0, release mode)..."
DEBUG_LEVEL=0 make shared_lib -j2

echo "3) install librocksdb..."
sudo cp -a librocksdb.so* "${LIB_PREFIX}"

sudo cp -a include/rocksdb "${INCLUDE_PREFIX}"

echo "done."