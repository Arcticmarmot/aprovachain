#!/usr/bin/env bash
set -euo pipefail

ROCKSDB_VERSION="${ROCKSDB_VERSION:-v10.4.2}"

ARCH="$(uname -m)"

# 1. found script dir
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
# 2. project root dir
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"
# 3. default prefix dir
PREFIX="${ROOT_DIR}/vendor/rocksdb-${ROCKSDB_VERSION}-${ARCH}"

echo "=== librocksdb install ==="
echo "version:   ${ROCKSDB_VERSION}"
echo "arch:   ${ARCH}"
echo "install location:   ${PREFIX}"
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

echo "3) install librocksdb: ${PREFIX}"
mkdir -p "${PREFIX}/lib" "${PREFIX}/include"

cp -a librocksdb.so* "${PREFIX}/lib/"

cp -a include/rocksdb "${PREFIX}/include/rocksdb"

echo "done."
echo "ROCKSDB_LIB_DIR=${PREFIX}/lib"
echo "ROCKSDB_INCLUDE_DIR=${PREFIX}/include"