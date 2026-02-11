#!/bin/sh
ORIGINAL_DIR="$(pwd)"
cleanup() {
    cd "$WORKTREE_PATH"
    git add -A
    git diff --cached --quiet || git commit -m "WIP"
    cd "$ORIGINAL_DIR"
    git worktree remove "$WORKTREE_PATH" --force
    rm -rf "$WORKTREE_PATH"
    echo "Removed worktree: $WORKTREE_PATH"
}
trap cleanup EXIT
cd "$WORKTREE_PATH"
claude "$QUERY"
