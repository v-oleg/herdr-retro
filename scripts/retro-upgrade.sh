#!/usr/bin/env bash
# Move the retro-sidebar patch onto a newer herdr release, rebuild and install it.
#
#   herdr-retro-upgrade              upgrade to the newest release
#   herdr-retro-upgrade v0.9.5       upgrade to a specific release
#   herdr-retro-upgrade --no-handoff leave the running server alone
#
# Stops without installing anything if the rebase conflicts or the tests fail.
set -euo pipefail

target=""
handoff=1
for arg in "$@"; do
    case "$arg" in
        --no-handoff) handoff=0 ;;
        v*) target="$arg" ;;
        *) echo "usage: herdr-retro-upgrade [vX.Y.Z] [--no-handoff]" >&2; exit 2 ;;
    esac
done

cd "$(dirname "$(realpath "$0")")/.."
bin="$HOME/.local/bin/herdr-retro"

if [ "$(git branch --show-current)" != "retro-sidebar" ]; then
    echo "Switch to the retro-sidebar branch first." >&2
    exit 1
fi
if ! git diff --quiet HEAD; then
    echo "Commit or stash your changes first." >&2
    exit 1
fi

base="$(git describe --tags --abbrev=0 --match 'v*' HEAD)"
if [ -z "$target" ]; then
    target="$(git ls-remote --tags --sort=-v:refname upstream 'v[0-9]*' \
        | awk '{print $2}' | grep -v '\^{}' | grep -E '/v[0-9]+\.[0-9]+\.[0-9]+$' \
        | head -1 | sed 's|refs/tags/||')"
fi

if [ "$base" = "$target" ]; then
    echo "==> Already based on $target, rebuilding"
else
    echo "==> Moving the patch from $base to $target"
    git fetch -q upstream tag "$target"
    # --onto: release tags sit on separate branches, so only the patch is replayed.
    if ! git rebase --onto "$target" "$base" >/dev/null 2>&1; then
        conflicts="$(git diff --name-only --diff-filter=U)"
        git rebase --abort
        echo >&2
        echo "The patch conflicts with $target, so nothing changed and herdr-retro still works." >&2
        echo "Files that need fixing by hand:" >&2
        echo "$conflicts" | sed 's/^/  /' >&2
        exit 1
    fi
fi

zig="$(sed -nE 's/.*minimum_zig_version = "([^"]+)".*/\1/p' vendor/libghostty-vt/build.zig.zon)"
mise install -q "zig@$zig"
run() { mise exec rust@stable "zig@$zig" -- "$@"; }

echo "==> Testing (Zig $zig)"
run cargo test --bin herdr client::shell 2>&1 | grep -E '^test result|FAILED|^error'
echo "==> Building"
run cargo build --release

if [ -x "$bin" ]; then
    cp -f "$bin" "$bin.prev"
fi
install -m 755 target/release/herdr "$bin"
echo "==> Installed $("$bin" --version), previous build saved as $bin.prev"

if [ "$handoff" = 1 ] && "$bin" status 2>/dev/null | grep -q 'status: running'; then
    echo "==> Handing the running server over to the new build"
    "$bin" server live-handoff --import-exe "$bin"
    echo "    Panes keep running. If your herdr window closed, run herdr to reattach."
else
    echo "==> Running server left alone"
fi

if [ "$base" != "$target" ]; then
    echo
    echo "Save it to GitHub with:"
    echo "  git -c credential.helper= -c credential.helper='!gh auth git-credential' push --force-with-lease origin retro-sidebar"
fi
