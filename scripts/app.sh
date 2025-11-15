#!/usr/bin/env bash
set -euo pipefail

# 1. 找到当前脚本所在的目录
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# 2. 项目根目录 = scripts 的上一层目录
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." &>/dev/null && pwd)"

export DEPLOY_JSON="$ROOT_DIR/apps/fixtures/tx/deploy.json"
export DEPLOY_ELF="$ROOT_DIR/apps/fixtures/elf/uav.bin"
export EXEC_JSON="$ROOT_DIR/apps/fixtures/tx/exec.json"

cargo run -p apps "$@"