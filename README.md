# mark

`mark` is a small Rust CLI for saving named directory shortcuts.

It supports two kinds of marks:

- **global marks**: available anywhere
- **repo-local marks**: available only inside the current Git repository

When you ask `mark` for a name, it prints the target path. With a small shell wrapper, you can make it `cd` there directly.

## Why

Shell aliases are fine for a few fixed locations, but they are awkward when:

- you want shortcuts tied to a specific Git repository
- you want to save the directory you are currently in
- you want a simple JSON-backed tool instead of editing shell config by hand

`mark` solves that by storing named paths in a local database and resolving them based on your current working directory.

## Features

- Add a mark for the current directory
- List marks in the current scope
- Remove marks
- Detect whether you are inside a Git repo
- Prefer repo-local marks when inside a repo
- Fall back to global marks when no repo-local mark exists
- Store repo-local marks as paths relative to the repo root

## Example

```
Repo: /home/RubberDuckHero/programming/rust/mark

Repo marks
  release          /home/RubberDuckHero/programming/rust/mark/target/release
  src              /home/RubberDuckHero/programming/rust/mark/src

Global marks
  haskell          /home/RubberDuckHero/programming/haskell
  home             /home/RubberDuckHero
  root             /
  rust             /home/RubberDuckHero/programming/rust
```

## How it works

If you are inside a Git repository, `mark` uses the repository root as the current scope.

- `mark add foo` stores the current directory as a **repo-local** mark named `foo`
- `mark foo` first looks for `foo` in the current repo
- if not found, it falls back to global marks

If you are **not** inside a Git repository, marks are stored and resolved in the **global** scope.

## Installation

### Build from source

```bash
cargo build --release
```

The compiled binary will be at:

```bash
target/release/mark
```

Move it somewhere on your `PATH`, for example:

```bash
install -m 755 target/release/mark ~/.local/bin/mark
```

Make sure `~/.local/bin` is on your `PATH`.

## Bash integration

Because a child process cannot change the current directory of your shell, the `mark` binary itself only **prints** the resolved path.

To make `mark foo` actually change directory, add this function to your `~/.bashrc`:

```bash
mark() {
	case "$1" in
		list|add|rm|help|-h|--help)
			command mark "$@"
			;;
		*)
			cd "$(command mark "$@")"
			;;
	esac
}
```

Then reload your shell:

```bash
source ~/.bashrc
```

### What this wrapper does

- `mark list`, `mark add ...`, `mark rm ...`, and help flags call the real binary normally
- anything else is treated as a mark lookup
- the wrapper runs `command mark ...` to avoid recursively calling the shell function
- the printed path is passed into `cd`

So:

```bash
mark work
```

becomes:

```bash
cd "$(command mark work)"
```

## Usage

### Add a mark

Save the current directory under a name:

```bash
mark add work
```

If you are inside a Git repo, this creates a repo-local mark.
If you are outside a Git repo, this creates a global mark.

### Jump to a mark

With the Bash wrapper installed:

```bash
mark work
```

Without the wrapper, the binary prints the path instead:

```bash
command mark work
```

### List marks

```bash
mark list
```

Inside a repo, this shows:

- repo-local marks for the current repo
- global marks

Outside a repo, this shows only global marks.

### Remove a mark

```bash
mark rm work
```

This removes the mark from the **current scope**.

That means:

- inside a repo, it removes the repo-local mark
- outside a repo, it removes the global mark

## Examples

### Global mark

```bash
cd ~/Documents
mark add docs
mark docs
```

With the Bash wrapper, the last command changes into `~/Documents`.

### Repo-local mark

```bash
cd ~/src/my-project/server
mark add api
```

Later, from anywhere inside `~/src/my-project`:

```bash
mark api
```

This resolves to the saved directory inside that repo.

### Repo-local takes priority over global

Suppose you have:

- a global mark named `src`
- a repo-local mark also named `src`

When you run:

```bash
mark src
```

from inside that repo, the repo-local mark wins.
Outside the repo, only the global mark is considered.

## Data storage

`mark` stores its database as JSON under your local data directory, in:

```text
<local-data-dir>/mark/marks.json
```

On Linux this is typically:

```text
~/.local/share/mark/marks.json
```

The structure is roughly:

```json
{
  "__global__": {
    "docs": "/home/alice/Documents"
  },
  "/home/alice/src/my-project": {
    "api": "server",
    "ui": "frontend"
  }
}
```

Repo-local paths are stored relative to the repo root.
Global paths are stored as canonical absolute paths.

## Command summary

```bash
mark list
mark add <name>
mark rm <name>
mark <name>
```

## Notes

- Git repo detection works by walking upward from the current directory until a `.git` entry is found
- repo-local marks are tied to the canonicalized repo root path
- adding a mark with an existing name in the same scope overwrites it
- removing a non-existent mark is a no-op
- the tool prints an error if a mark cannot be found

## Requirements

This project uses Rust and depends on these crates:

- `anyhow`
- `clap`
- `dirs`
- `serde_json`

## License

Copyright 2026 RubberDuckHero

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
