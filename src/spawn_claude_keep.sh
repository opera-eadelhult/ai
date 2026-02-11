#!/bin/sh
cd "$WORKTREE_PATH"
claude "$QUERY"
git add -A
git diff --cached --quiet || git commit -m "WIP"
