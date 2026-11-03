#!/usr/bin/env bash
# Ubuntu 24.04. Invoke with bash after installing the pinned toolchain.
# Evidence is retained; no recursive deletion or worktree cleanup is performed.
set -euo pipefail

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
gates=(namespace toolchain vendor tamper features fmt clippy test doc wasm)

if [[ ${1:-} != --inside ]]; then
    [[ $# == 1 && $1 == /* ]] || fail 'usage: bash scripts/check-offline-linux.sh /abs/evidence'
    uid=$(id -u)
    gid=$(id -g)
    [[ $uid != 0 ]] || fail 'outer invocation must be the normal nonroot runner user'
    script=$(readlink -f -- "${BASH_SOURCE[0]}")
    repo=$(cd -- "$(dirname -- "$script")/.." && pwd -P)
    evidence=$(realpath -m -- "$1")
    case "$evidence/" in "$repo/"*) fail 'evidence must be outside the checkout';; esac
    mkdir -p -- "$evidence"
    evidence=$(cd -- "$evidence" && pwd -P)
    case "$evidence/" in "$repo/"*) fail 'evidence must be outside the checkout';; esac
    task=$(mktemp -d -- "$evidence/MAM-938.XXXXXXXX")
    mkdir -- "$task/home" "$task/cargo-home" "$task/tmp" "$task/logs"
    for gate in "${gates[@]}"; do printf 'NOT RUN\n' > "$task/$gate.status"; done
    printf 'Evidence: %s\n' "$task"
    if [[ -n ${RUSTUP_HOME:-} ]]; then
        rustup_home=$RUSTUP_HOME
    else
        rustup_home=${HOME:?}/.rustup
    fi
    [[ $rustup_home == /* && -d $rustup_home ]] || fail 'RUSTUP_HOME must be an existing absolute path'
    cargo_binary=$(rustup which --toolchain 1.99.0 cargo)
    [[ $cargo_binary == /* && -x $cargo_binary ]] || fail 'installed toolchain cargo was not resolved'
    toolbin=$(cd -- "$(dirname -- "$cargo_binary")" && pwd -P)
    [[ -x $toolbin/rustc && -x $toolbin/cargo-fmt && -x $toolbin/clippy-driver ]] || fail 'toolchain components missing'
    wasm_lib=$("$toolbin/rustc" --print target-libdir --target wasm32-unknown-unknown)
    compgen -G "$wasm_lib/libstd-*.rlib" > /dev/null || fail 'preinstalled wasm std missing'
    outer_ns=$(readlink /proc/self/ns/net)
    launch=(/usr/bin/sudo -- /usr/bin/unshare --net -- /usr/bin/setpriv
        "--reuid=$uid" "--regid=$gid" --clear-groups --bounding-set=-all
        --inh-caps=-all --ambient-caps=-all --no-new-privs /usr/bin/env -i
        "HOME=$task/home" "CARGO_HOME=$task/cargo-home" "RUSTUP_HOME=$rustup_home"
        RUSTUP_TOOLCHAIN=1.99.0 "CARGO_TARGET_DIR=$task/target" "TMPDIR=$task/tmp"
        "PATH=$toolbin:/usr/bin:/bin" /bin/bash "$script" --inside "$repo" "$task" "$outer_ns")
    "${launch[@]}" 2>&1 | tee "$task/launcher.log"
    exit 0
fi

[[ $# == 4 ]] || fail 'invalid internal arguments'
repo=$2
task=$3
outer_ns=$4
[[ $(id -u) != 0 ]] || fail 'inside process must be nonroot'
[[ $repo == /* && $task == /* && -d $task/logs ]] || fail 'invalid internal paths'
[[ ${HOME:?} == "$task/home" && ${CARGO_HOME:?} == "$task/cargo-home"
   && ${CARGO_TARGET_DIR:?} == "$task/target" && ${TMPDIR:?} == "$task/tmp" ]] || fail 'invalid isolated directories'
[[ ${RUSTUP_HOME:?} == /* && ${RUSTUP_TOOLCHAIN:?} == 1.99.0 ]] || fail 'invalid toolchain environment'
[[ -d $HOME && -d $CARGO_HOME && -d $TMPDIR ]] || fail 'isolated directories missing'
shopt -s nullglob dotglob
initial_entries=("$HOME"/* "$CARGO_HOME"/* "$TMPDIR"/*)
(( ${#initial_entries[@]} == 0 )) || fail 'HOME, CARGO_HOME and TMPDIR must start empty'
shopt -u nullglob dotglob
[[ ! -e $CARGO_TARGET_DIR && ! -L $CARGO_TARGET_DIR ]] || fail 'target must not exist initially'
export CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0
cd -- "$repo"

run_gate() {
    local name=$1 rc
    shift
    printf 'RUNNING\n' > "$task/$name.status"
    if "$@" 2>&1 | tee "$task/logs/$name.log"; then
        printf 'PASS\n' > "$task/$name.status"
        printf 'PASS %s\n' "$name"
    else
        rc=$?
        printf 'FAIL %s\n' "$rc" > "$task/$name.status"
        printf 'FAIL %s; remaining gates retain NOT RUN status\n' "$name" >&2
        return "$rc"
    fi
}

verify_namespace() {
    python3 -I - "$outer_ns" <<'PY'
import errno
import fcntl
import os
from pathlib import Path
import socket
import struct
import sys

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(sys.version_info >= (3, 11), "Python 3.11+ required")
require(os.getuid() != 0 and os.geteuid() != 0, "root inside namespace rejected")
inside = os.readlink("/proc/self/ns/net")
require(inside != sys.argv[1], "network namespace is unchanged")
status = dict(line.split(":", 1) for line in Path("/proc/self/status").read_text().splitlines() if ":" in line)
for key in ("CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"):
    require(int(status[key].strip(), 16) == 0, "capabilities were not fully dropped")
require(status["NoNewPrivs"].strip() == "1", "no-new-privs missing")
require(sorted(name for _, name in socket.if_nameindex()) == ["lo"], "non-loopback interface present")
with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as control:
    result = fcntl.ioctl(control.fileno(), 0x8913, struct.pack("256s", b"lo"))
    require(not (struct.unpack_from("H", result, 16)[0] & 1), "loopback is UP")
for line in Path("/proc/net/route").read_text().splitlines()[1:]:
    fields = line.split()
    require(len(fields) >= 3 and fields[1] != "00000000", "IPv4 default route present")
for line in Path("/proc/net/ipv6_route").read_text().splitlines():
    fields = line.split()
    require(len(fields) >= 2 and not (fields[0] == "0" * 32 and fields[1] == "00"),
            "IPv6 default route present")
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as connection:
    connection.settimeout(3)
    try:
        connection.connect(("1.1.1.1", 443))
    except OSError as error:
        require(error.errno == errno.ENETUNREACH,
                "numeric TCP probe failed without the required ENETUNREACH")
    else:
        raise SystemExit("numeric TCP connection unexpectedly succeeded")
print(f"PASS kernel network namespace: outer={sys.argv[1]} inside={inside} uid={os.getuid()}")
print("PASS capabilities dropped; only loopback, DOWN; no default routes; TCP errno=ENETUNREACH")
PY
}

toolchain_check() {
    local version target_lib
    version=$(rustc --version) || return
    [[ $version == 'rustc 1.99.0 '* ]] || { printf 'wrong rustc version\n' >&2; return 1; }
    printf '%s\n' "$version"
    cargo --version || return
    target_lib=$(rustc --print target-libdir --target wasm32-unknown-unknown) || return
    compgen -G "$target_lib/libstd-*.rlib" > /dev/null || { printf 'wasm std missing\n' >&2; return 1; }
}

run_gate namespace verify_namespace
run_gate toolchain toolchain_check
run_gate vendor python3 -I scripts/check-vendor-crates.py
run_gate tamper python3 -I scripts/test-vendor-crates.py --log-dir "$task/logs/tamper" --cargo "$(command -v cargo)"
run_gate features cargo --frozen tree -e features -p engine-cli
run_gate fmt cargo fmt --all -- --check
run_gate clippy cargo --frozen clippy --workspace --all-targets -- -D warnings
run_gate test cargo --frozen test --workspace
run_gate doc env 'RUSTDOCFLAGS=-D warnings' cargo --frozen doc --no-deps --workspace
run_gate wasm cargo --frozen check --workspace --target wasm32-unknown-unknown
printf 'All listed Linux gates passed. Effective feature review remains a coordinator review step.\n'
