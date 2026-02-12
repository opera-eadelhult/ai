# `ai`
My tiny opinionated wrapper around Claude Code to execute commands and manage git worktrees.


## Setup
Assuming you have a [Rust toolchain](https://rust-lang.org/tools/install/) installed, just run:

```bash
cargo install --locked --git https://github.com/opera-eadelhult/ai.git
```
```bash
ai help
ai agent --help # help flag can be used on every subcommand for more options
```


## `ai agent`: Manage AI agents and git worktrees
Spawn Claude in a git worktree and grant write permissions. Once the Claude session is exited, the work will be committed to a branch for you to review. 
The command will also manage all clean-up work like removing the worktree.


https://github.com/user-attachments/assets/3d1ec003-73c6-4f65-8270-3f7aa4d0829d



## `ai do`: Suggest and execute bash commands

Claude also has automatic access to read the current directory and query the web.

https://github.com/user-attachments/assets/decabc8f-0a11-45dc-8f18-21bb50d3561b

## `ai ask`: Ask questions

Just like `ai do`, Claude has read-access to the current directory and can query the web.

https://github.com/user-attachments/assets/b463143c-d909-4594-8735-0890d66b0c67



