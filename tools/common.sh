set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
source "$ROOT/upstream.env"

WORK=${RIKKA_GECKO_WORK:-$ROOT/.cache}
TREE=$WORK/firefox
OBJ=$WORK/obj
OUT=${RIKKA_GECKO_OUT:-$ROOT/out}

export MOZCONFIG=$ROOT/mozconfig
export MOZBUILD_STATE_PATH=${MOZBUILD_STATE_PATH:-$WORK/mozbuild}
export SCCACHE_DIR=${SCCACHE_DIR:-$WORK/sccache}
export SCCACHE_CACHE_SIZE=${SCCACHE_CACHE_SIZE:-30G}

revision() {
    git -C "$ROOT" rev-parse --short=12 HEAD
}

artifact_name() {
    echo "rikka-gecko-$FIREFOX_VERSION-$(revision)-x86_64-linux"
}
