#!/bin/sh
ORIGINAL_DIR="$(pwd)"
cleanup() {
    # Commit if needed
    git add -A
    git diff --cached --quiet || git commit -m "WIP"

    # Cleanup the worktree
    git worktree remove "$WORKTREE_PATH" --force
    rm -rf "$WORKTREE_PATH"

    echo "Worktree at $WORKTREE_PATH was removed"
    echo "To review the work: 'git switch $BRANCH_NAME'"
    cd "$ORIGINAL_DIR"
}
trap cleanup EXIT
cd "$WORKTREE_PATH"
BRANCH_NAME="$(git branch --show-current)"
claude "$QUERY"
