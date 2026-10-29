#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -ne 1 ]]; then
  printf 'Usage: %s INSTALL_ROOT\n' "$0" >&2
  exit 2
fi

install_root="$1"
script_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
elan_version='4.2.3'
lean_toolchain='leanprover/lean4:v4.33.1'
toolchain_dir='leanprover--lean4---v4.33.1'
platform="$(uname -s)-$(uname -m)"

case "$platform" in
  Linux-x86_64)
    asset='elan-x86_64-unknown-linux-gnu.tar.gz'
    digest='df0b2b3a439961ffcbb3985214365ffe40f49bc871df04dff268c7d8e21ca8b2'
    ;;
  Linux-aarch64)
    asset='elan-aarch64-unknown-linux-gnu.tar.gz'
    digest='cb69af0803b04157bc30201c29c12fca882bb3ad8b43476b8d2d3064810bc3ac'
    ;;
  Darwin-x86_64)
    asset='elan-x86_64-apple-darwin.tar.gz'
    digest='10d037a69731c0593723e018130c5f54afde175796b4af8ba1317e561e55598c'
    ;;
  Darwin-arm64)
    asset='elan-aarch64-apple-darwin.tar.gz'
    digest='7cae4c03b2f0de4053fb04a91359d5804551e6e37a6ddd1b2e0097dc561ae4a9'
    ;;
  *)
    printf 'Unsupported platform: %s\n' "$platform" >&2
    exit 2
    ;;
esac

mkdir -p "$install_root/elan" "$install_root/bin" "$install_root/lib"
install_root="$(cd "$install_root" && pwd)"
elan_home="$install_root/elan"
installer_tmp="$(mktemp -d /tmp/formal-lean-install.XXXXXX)"
cleanup() {
  case "$installer_tmp" in
    /tmp/formal-lean-install.*) rm -rf -- "$installer_tmp" ;;
  esac
}
trap cleanup EXIT

if [[ ! -x "$elan_home/bin/elan" ]]; then
  url="https://github.com/leanprover/elan/releases/download/v${elan_version}/${asset}"
  curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --max-time 180 \
    -o "$installer_tmp/elan.tar.gz" "$url"
  printf '%s  %s\n' "$digest" "$installer_tmp/elan.tar.gz" | sha256sum -c -
  tar --no-same-owner -xzf "$installer_tmp/elan.tar.gz" -C "$installer_tmp"
  ELAN_HOME="$elan_home" "$installer_tmp/elan-init" \
    -y --no-modify-path --default-toolchain none
fi

toolchain_root="$elan_home/toolchains/$toolchain_dir"
if [[ ! -x "$toolchain_root/bin/lean" ]]; then
  set +e
  install_output="$(ELAN_HOME="$elan_home" "$elan_home/bin/elan" \
    toolchain install "$lean_toolchain" 2>&1)"
  install_status="$?"
  set -e
  printf '%s\n' "$install_output"
else
  install_status=0
  printf 'Lean toolchain already installed: %s\n' "$lean_toolchain"
fi

if [[ ! -x "$toolchain_root/bin/lean" ]]; then
  printf 'Lean installation failed with status %s\n' "$install_status" >&2
  exit 1
fi

for tool in lean lake leanc leanchecker; do
  install -m 0755 "$script_root/lean_tool_launcher.sh" "$install_root/bin/$tool"
done

set +e
version_output="$("$toolchain_root/bin/lean" --version 2>&1)"
version_status="$?"
set -e
if [[ "$version_status" -ne 0 ]] &&
   [[ "$version_output" == *'failed to locate application'* ]] &&
   [[ "$(uname -s)" == 'Linux' ]]; then
  if ! command -v cc >/dev/null 2>&1; then
    printf 'Lean needs the sandbox self-location shim, but cc is unavailable.\n' >&2
    exit 1
  fi
  cc -shared -fPIC -O2 -Wall -Wextra -Werror \
    -o "$install_root/lib/lean-proc-self-shim.so" \
    "$script_root/lean_proc_self_shim.c" -ldl
fi

"$install_root/bin/lean" --version
"$install_root/bin/lake" --version
printf 'Lean tools: %s/bin\n' "$install_root"
