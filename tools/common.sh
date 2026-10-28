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

patches() {
    (cd "$ROOT/patches" && find . -name '*.patch' -type f -printf '%P\n' | sort)
}

apply_patches() {
    git -C "$TREE" reset -q --hard
    git -C "$TREE" clean -q -fdx
    local patch
    while read -r patch; do
        git -C "$TREE" apply --whitespace=nowarn "$ROOT/patches/$patch"
    done < <(patches)
}

reproduces() {
    local patch=$1 file=$2 index
    [[ -f $TREE/$file ]] || return 1
    index=$(mktemp -d)
    local expected=
    if GIT_INDEX_FILE=$index/index git -C "$TREE" read-tree HEAD &&
        GIT_INDEX_FILE=$index/index git -C "$TREE" apply --cached "$patch" 2> /dev/null; then
        expected=$(GIT_INDEX_FILE=$index/index git -C "$TREE" rev-parse ":$file")
    fi
    rm -rf "$index"
    [[ -n $expected && $expected == $(git -C "$TREE" hash-object -- "$file") ]]
}

export_patch() {
    local file=$1 patch=$ROOT/patches/$1.patch
    if [[ -f $TREE/$file ]] && grep -qE '^(<{7}|>{7})( |$)' "$TREE/$file"; then
        echo "$TREE/$file still has conflict markers" >&2
        return 1
    fi
    if [[ -f $patch ]] && reproduces "$patch" "$file"; then
        return
    fi
    mkdir -p "$(dirname "$patch")"
    git -C "$TREE" diff HEAD -- "$file" > "$patch"
}

export_patches() {
    local -A held=() changed=()
    local file patch
    for file in "$@"; do
        held[$file]=1
    done
    while read -r file; do
        changed[$file]=1
        [[ -v held[$file] ]] || export_patch "$file"
    done < <(git -C "$TREE" diff HEAD --name-only)
    while read -r patch; do
        file=${patch%.patch}
        [[ -v changed[$file] || -v held[$file] ]] || rm "$ROOT/patches/$patch"
    done < <(patches)
    find "$ROOT/patches" -mindepth 1 -type d -empty -delete
}
