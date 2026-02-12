#!/bin/sh
ORIGINAL_DIR="$(pwd)"
ORIGINAL_BRANCH="$(git branch --show-current)"
cleanup() {
    # Commit if needed
    git add -A
    git diff --cached --quiet || git commit -m "WIP"

    # Cleanup the worktree
    git worktree remove "$WORKTREE_PATH" --force
    rm -rf "$WORKTREE_PATH"

    cd "$ORIGINAL_DIR"

    # Check if there are any differences from the original branch
    if git diff --quiet "$ORIGINAL_BRANCH"..."$BRANCH_NAME"; then
        echo "No changes made. Branch $BRANCH_NAME and directory $WORKTREE_PATH were deleted."
        git branch -D "$BRANCH_NAME"
    else
        echo "Directory $WORKTREE_PATH was removed"
        echo "To review the work: 'git switch $BRANCH_NAME'"
    fi
}
trap cleanup EXIT
cd "$WORKTREE_PATH"
BRANCH_NAME="$(git branch --show-current)"
if [ -n "$MODEL" ]; then
    claude --permission-mode=acceptEdits --model="$MODEL" "$QUERY"
else
    claude --permission-mode=acceptEdits "$QUERY"
fi
