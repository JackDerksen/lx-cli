# LX: A nicer way to list your files

A modern alternative to `ls` with readable formatting, colours, optional icons, and customisable output.

- Short, long, compact, single-column, and recursive tree views.
- Git status badges, with summaries of changes beneath directories.
- Sorting, filename filters, and plain or NUL-separated output for scripts.
- Custom themes, columns, date formats, and icons.

![LX long listing compared with the standard ls command](https://github.com/user-attachments/assets/17bf867c-faf7-483b-a7c7-906bd9d5a549)

## Install

Requires Rust 1.85 or newer and Cargo:

```bash
cargo install lx-cli
```

Cargo installs `lx` to `~/.cargo/bin/`; make sure it is on your `PATH`. See [building from source](wiki.md#building-from-source) for local installation and development.

## Usage

Use `lx` like `ls`; it lists the current directory unless you supply a path. Run `lx --help` for all options.

```bash
lx                                      # List the current directory
lx -la                                  # Long listing, including hidden files
lx -1                                   # One entry per line
lx -c                                   # Compact columns
lx -r --depth 2                         # Tree, limited to two levels below the root
lx -lrg --depth 2 --git-ignore          # Long tree with Git badges, omitting ignored entries
lx -d                                   # Directories only
lx -x '.git,target,Cargo.*'             # Exclude names or glob patterns
lx -s size desc                         # Sort by size, largest first
lx --plain                              # Undecorated filenames, one per line
lx --zero                               # NUL-separated filenames for scripts
```

Piped output defaults to plain filenames, one per line. Use `--zero` when filenames may contain newlines. [Pipes and scripts](wiki.md#pipes-and-scripts) covers explicit display modes and paths in pipelines.

## Customise

Configuration lives at `~/.config/lx/config.toml`. Start with [config.example](config.example), or add just the settings you want:

```toml
[icons]
enabled = true

[display]
long_format_fields = ["permissions", "owner", "size", "modified", "filename"]
```

Icons are off by default. Enabling them requires a terminal or font with Nerd Font glyphs; see [Nerd Fonts](https://www.nerdfonts.com/font-downloads).
Alternatively, I'd recommend a terminal emulator like Ghostty, which has icons built-in.

See the [reference](wiki.md) for [all flags](wiki.md#flags), [Git badges](wiki.md#git-status), [directory sizes](wiki.md#directory-sizes), [themes](wiki.md#themes), and [custom icons](wiki.md#icons). The [default theme](themes/default.toml) lists every style role.
