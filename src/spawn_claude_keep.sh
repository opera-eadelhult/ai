#!/bin/sh
ORIGINAL_DIR="$(pwd)"
cleanup() {
    # Commit if needed
    git add -A
    git diff --cached --quiet || git commit -m "WIP"

    echo "Directory $WORKTREE_PATH was kept"
    echo "To review the work, explore your current working directory,\nor: 'git switch $BRANCH_NAME'"
}
trap cleanup EXIT
cd "$WORKTREE_PATH"
BRANCH_NAME="$(git branch --show-current)"
claude --permission-mode=acceptEdits "$QUERY"
