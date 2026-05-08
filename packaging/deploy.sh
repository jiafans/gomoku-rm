#!/usr/bin/env bash
# Build → e_machine guard → scp → install on rM2 AppLoad
set -euo pipefail

cd "$(dirname "$0")/.."

TARGET=armv7-unknown-linux-gnueabihf
BIN_NAME=gomoku-rm
BIN=target/${TARGET}/release/${BIN_NAME}
RM_HOST=${RM_HOST:-root@10.11.99.1}
RM_APP_DIR=/home/root/xovi/exthome/appload/gomoku
ICON=icon.png
MANIFEST=packaging/external.manifest.json

# 1) Build (cargo-zigbuild on Apple Silicon avoids cross/Docker x86_64 toolchain issues)
echo "==> cargo zigbuild --target ${TARGET} --release"
cargo zigbuild --target "${TARGET}" --release

# 2) e_machine guard (踩过 chessmarkable 0.8.1-1 的坑)
EMACH=$(xxd -s 18 -l 2 -p "${BIN}")
if [[ "${EMACH}" == "0000" ]]; then
  echo "==> e_machine = 0000 (broken), patching to 2800 (EM_ARM)"
  printf '\x28\x00' | dd of="${BIN}" bs=1 seek=18 count=2 conv=notrunc 2>/dev/null
  EMACH=$(xxd -s 18 -l 2 -p "${BIN}")
fi
if [[ "${EMACH}" != "2800" ]]; then
  echo "FATAL: ELF e_machine is ${EMACH}, expected 2800. file says: $(file -b "${BIN}")"
  exit 1
fi
echo "    e_machine OK: ${EMACH} ($(file -b "${BIN}" | cut -c1-60))"

# 3) Sanity: file looks like ARM ELF
file "${BIN}" | grep -q "ARM" || { echo "FATAL: not ARM ELF"; exit 1; }

# 4) Push to rM2 (relies on ssh key OR sshpass + RM_PASS env)
SSH_OPTS=(-o StrictHostKeyChecking=accept-new -o ConnectTimeout=8)
SSH_CMD=(ssh "${SSH_OPTS[@]}" "${RM_HOST}")
SCP_CMD=(scp "${SSH_OPTS[@]}")
if [[ -n "${RM_PASS:-}" ]]; then
  SSH_CMD=(sshpass -e ssh "${SSH_OPTS[@]}" "${RM_HOST}")
  SCP_CMD=(sshpass -e scp "${SSH_OPTS[@]}")
  export SSHPASS="${RM_PASS}"
fi

echo "==> mkdir on device"
"${SSH_CMD[@]}" "mkdir -p ${RM_APP_DIR}"

echo "==> scp binary + manifest + icon"
"${SCP_CMD[@]}" "${BIN}" "${RM_HOST}:${RM_APP_DIR}/${BIN_NAME}"
"${SCP_CMD[@]}" "${MANIFEST}" "${RM_HOST}:${RM_APP_DIR}/external.manifest.json"
if [[ -f "${ICON}" ]]; then
  "${SCP_CMD[@]}" "${ICON}" "${RM_HOST}:${RM_APP_DIR}/icon.png"
else
  echo "    (no icon.png locally — AppLoad will show default placeholder)"
fi

echo "==> chmod +x"
"${SSH_CMD[@]}" "chmod +x ${RM_APP_DIR}/${BIN_NAME}"

echo "==> deployed; on rM2 open AppLoad and tap Gomoku"
"${SSH_CMD[@]}" "ls -la ${RM_APP_DIR}"
