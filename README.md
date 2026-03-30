# hikkiban

hikkiban is a small, cross-platform command-line clipboard tool. The name comes from the Japanese word "hikkiban" (a clipboard), and the utility provides simple copy/paste operations for files and standard input/output. It targets macOS and WSL2 users primarily but works anywhere the `clipboard_anywhere` crate supports the clipboard backend.

## Features

- Copy file contents or stdin to the system clipboard
- Paste clipboard contents to a file or stdout
- Lightweight CLI with verbosity flags (`-v`, `-vv`, ...)

## Installation

Build from source with Cargo:

```bash
cargo install --path .
```

Or build locally for testing:

```bash
cargo build --release
```

The resulting binary is called `cb`.

## Usage

Basic commands:

- Copy a file to the clipboard:

```bash
hikkiban copy --file notes.txt
# or
hikkiban copy -f notes.txt
```

- Copy from stdin (useful with pipes):

```bash
echo "hello world" | hikkiban copy
# or
hikkiban copy < notes.txt
```

- Paste clipboard contents to stdout:

```bash
hikkiban paste
```

- Paste clipboard contents into a file:

```bash
hikkiban paste --file out.txt
# or
hikkiban paste -f out.txt
```

Use verbosity flags for more logging output:

```bash
hikkiban -v copy -f notes.txt   # info
hikkiban -vv paste               # debug
```

## Notes

- Primary targets are macOS and WSL2, but the underlying clipboard backend may support other platforms.

## License

See the project repository for license information.