#!/usr/bin/env bash
set -euo pipefail

# 1. 找到当前脚本所在的目录
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# 2. 项目根目录 = scripts 的上一层目录
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"

export ROCKSDB_LIB_DIR="$HOME/rocksdb/lib"
export ROCKSDB_INCLUDE_DIR="$HOME/rocksdb/include"
export LD_LIBRARY_PATH="$HOME/rocksdb/lib:${LD_LIBRARY_PATH:-}"

cargo run -p node "$@"

