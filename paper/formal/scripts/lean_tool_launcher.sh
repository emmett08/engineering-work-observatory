#!/usr/bin/env bash
set -euo pipefail

formal_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tool_name="$(basename "$0")"
toolchain_root="$formal_root/elan/toolchains/leanprover--lean4---v4.33.1"
shim="$formal_root/lib/lean-proc-self-shim.so"

if [[ -f "$shim" ]]; then
  export LD_PRELOAD="$shim${LD_PRELOAD:+:$LD_PRELOAD}"
fi
export PATH="$toolchain_root/bin:$PATH"
exec "$toolchain_root/bin/$tool_name" "$@"
