#!/usr/bin/env bash
set -euo pipefail

ROCKSDB_VERSION="${ROCKSDB_VERSION:-v10.4.2}"

ARCH="$(uname -m)"

# 1. found script dir
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
# 2. project root dir
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"
# 3. default prefix dir
PREFIX="${ROOT_DIR}/vendor/rocksdb-${ROCKSDB_VERSION}"
# 4. ld.so.conf.d 里要写的配置文件
CONF_FILE="/etc/ld.so.conf.d/rocksdb-${ROCKSDB_VERSION}-${ARCH}.conf"

echo "=== librocksdb install ==="
echo "version:   ${ROCKSDB_VERSION}"
echo "arch:   ${ARCH}"
echo "install location:   ${PREFIX}"
echo "ld conf location:${CONF_FILE}"
echo

echo "create ld dir: ${PREFIX}"
echo "${PREFIX}/lib" | sudo tee "${CONF_FILE}" >/dev/null
sudo ldconfig

echo "done."
echo "ROCKSDB_LIB_DIR=${PREFIX}/lib"
echo "ROCKSDB_INCLUDE_DIR=${PREFIX}/include"
echo "RocksDB installed to: ${PREFIX}"
echo "lib dir registered in: ${CONF_FILE}"