# LX: A nicer way to list your files

A modern alternative to `ls` with more readable formatting, colours, icons, and customizable output.

## Features

- **Crazy fast**: just as fast (if not faster) compared to the stock `ls` command
- **Colorized output** with file type icons
- **Multiple display modes**: short (default), long (`-l`), one-per-line (`-1`), compact (`-c`), and recursive tree view (`-r`)
- **Configurable**: customise colours, hidden-file styling, spacing, display options, and tree styles
- **Smart alignment**: properly handles unicode characters and icons
- **Git status**: optional single-character status badges, including summaries of changes beneath directories


## Demo

<img width="3024" height="1900" alt="CleanShot 2026-08-08 at 18 32 07@2x" src="https://github.com/user-attachments/assets/ee340327-6152-4ddf-b25d-8681eb0744b6" />

Basic formatting difference compared to the standard `ls` command

---
<img width="3024" height="1900" alt="CleanShot 2026-08-08 at 18 33 05@2x" src="https://github.com/user-attachments/assets/17bf867c-faf7-483b-a7c7-906bd9d5a549" />

Nicer long format compared to the standard `ls` command (displayed fields can be customized)

---
<img width="3024" height="1900" alt="CleanShot 2026-08-08 at 18 33 52@2x" src="https://github.com/user-attachments/assets/d53796f7-461a-4e97-b5e9-766a033584bb" />

Recursive sub-directory tree view with the `-r` flag

---
<img width="3024" height="1900" alt="CleanShot 2026-08-08 at 18 34 40@2x" src="https://github.com/user-attachments/assets/e608a0a5-03a8-49f9-9562-78fdf1926894" />

The `-r` flag can be combined with the '-l' long flag for viewing subdirectory info

---
<img width="3024" height="1900" alt="CleanShot 2026-08-08 at 18 35 19@2x" src="https://github.com/user-attachments/assets/0530bcdf-6ba6-4f07-a673-e12a13243779" />

`-1` flag to force single-column output formatting


## Requirements

- Rust 1.85 or newer and Cargo
- Either a [nerd font](https://www.nerdfonts.com/font-downloads) or a terminal like Ghostty which has nerd font icons pre-installed.

## Installation

The easiest way to install `lx` is using Cargo:

```bash
cargo install lx-cli
```

This will install the `lx` binary to `~/.cargo/bin/`, which is typically already in your `PATH`.

<details>
<summary>Building from source</summary>

Alternatively, you can build it from source:

```bash
git clone https://github.com/JackDerksen/lx-cli
cd lx-cli
cargo build --release
cp target/release/lx ~/.local/bin/
```

For local development, use the provided install script to automatically build and install:

```bash
./install.sh
```

This script will build the project and copy the binary to `~/.local/bin/`.

Make sure `~/.local/bin` is in your `PATH`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Add this to your shell configuration file (`.bashrc`, `.zshrc`, etc.) to make it permanent.

</details>

## Quick Start

After installation, simply run:

```bash
lx
```

## Usage

`lx` is used pretty much just like regular old `ls`:

```bash
# List files in current directory
lx

# List files in long format
lx -l

# Show hidden files
lx -a

# Display one file per line
lx -1

# Piped output defaults to plain filenames, one per line
lx | head

# Force plain output, including in a terminal
lx --plain

# Use NUL separators to preserve filenames containing newlines
lx --zero | xargs -0 -I {} printf '%s\n' '{}'

# Use compact columns, wrapping after five rows by default
lx -c

# Show directory tree recursively
lx -r

# Show Git status beside entries (also works with -l, -1, -c, and -r)
lx --git
lx -G
lx -lrG src

# Show only directories or only files
lx -d
lx -f

# Exclude names or glob patterns from a listing
lx -x '.git,target,Cargo.*'

# Sort by size, largest first
lx -s size desc

# Combine flags
lx -la src
lx -lr
lx -alr /path/to/directory

# List files in a specific directory
lx /path/to/directory
```

## Flags

- `-l`, `--long`: Use a long listing format with detailed file information
- `-G`, `--git`: Show Git status for entries and summarise changes beneath directories
- `-a`, `--all`: Show all files, including hidden ones (starting with `.`)
- `-f`, `--files`: Show only files (not available with `-r`)
- `-d`, `--directories`: Show only directories (not available with `-r`)
- `-x`, `--exclude <PATTERN>`: Exclude comma-separated names or `*`/`?` glob patterns; repeat the flag to add more patterns
- `-s`, `--sort <FIELD> [ORDER]`: Sort by `name`, `size`, `modified`, `type`, `permissions`, `links`, `owner`, or `group`; use optional `asc` (the default) or `desc`
- `--sort-order <ORDER>`: Explicit alternative for specifying `asc` or `desc`; requires `--sort`
- `-1`: Force single-column output (useful for piping to other commands)
- `--plain`: Print undecorated filenames, one per line
- `-0`, `--zero`: Print undecorated filenames separated by NUL bytes
- `-c`, `--compact`: Use compact columns, wrapping after `compact_max_rows` rows
- `-r`, `--recursive`: Show directory tree recursively with proper hierarchy

`-l` and `-1` are treated as separate display modes, so they cannot be combined together. `-c` is only available for short output, so it cannot be combined with `-l`, `-1`, or `-r`. Short flags can still be clustered in the usual Unix style, so combinations like `-la`, `-lr`, and `-alr` work as expected.

When stdout is a pipe or redirected to a file, the default listing and `-1` print raw filenames, one per line, without icons, colours, Git badges, titles, or padding. Explicit `-l`, `-c`, `-r`, or `--git` retain their requested display mode. Use `--plain` to guarantee plain output in any destination, or `--zero` for an unambiguous filename stream, including filenames containing newlines or non-UTF-8 bytes. Both modes honour sorting, hidden-file settings, and filters, and disable styling even when colour is forced through the environment.

`--plain` and `--zero` cannot be combined with each other or with `-l`, `-c`, `-r`, or `--git`. Directory listings emit entry names relative to the listed directory; a single-file listing retains the supplied path. To pass files from another directory to a command, run the pipeline inside that directory, for example `(cd src && lx --zero | xargs -0 -I {} printf '%s\n' '{}')`. Empty listings emit no separators. A consumer that closes early, such as `head`, causes lx to exit quietly and successfully.

With `--git`, short listings show a single-character status badge before each entry and long listings gain a `Git` column:

| Badge | Meaning | Default colour |
| --- | --- | --- |
| `-` | No changes reported by Git (including ignored entries) | Bright black |
| `M` | Unstaged changes, including deletions | Yellow |
| `S` | Changes ready to commit, including additions and renames | Green |
| `?` | Not yet tracked by Git | Cyan |
| `!` | An unresolved merge conflict | Red |
| `*` | Both staged and unstaged changes, or a directory containing different states | Yellow |

Customise badge colours under `[colors.git]` in `~/.config/lx/config`. Each setting is optional; omitted settings keep their defaults. The available colour names are the same as for `[colors]`:

```toml
[colors.git]
clean = "bright_black"
modified = "yellow"
staged = "green"
untracked = "cyan"
conflict = "red"
mixed = "yellow"
```

Directories summarise changes anywhere beneath them, including deleted files. Any descendant conflict makes the directory show `!`. These summaries include hidden and excluded descendants, so they describe repository state independently of listing filters.

Git status is opt-in and uses one Git status query for the repository containing the target, including linked worktrees. It requires the `git` executable. Outside a repository, `--git` leaves the listing unchanged; if Git fails, lx reports a warning and prints the normal listing. Submodules receive the containing repository's summary rather than a separate status scan of their contents. Untracked files are enumerated to distinguish them from ignored files, which can add latency in large repositories.

## Configuration

`lx` can be customized using a configuration file at `~/.config/lx/config`.

See [`config.example`](config.example) for all available options and detailed configuration examples.

### Configuration Sections

The configuration file supports the following sections:

- **`[colors]`**: Customise text colours for different file types, including hidden files
- **`[colors.git]`**: Customise Git badge colours for each status
- **`[icons]`**: Set custom icons for different file types
- **`[icons.colors]`**: Customise colours for icons separately from filenames, including hidden file icons
- **`[display]`**: Control layout options, column wrapping, and default sorting
- **`[display.tree]`**: Control tree display style for recursive listings (`style = "ascii"` or `style = "indent"`)

Set `datetime_format` under `[display]` to change the long-format `modified` column. It uses Chrono’s strftime-style specifiers, so `%B %-d, %Y` renders a full month name (for example, `August 8, 2026`) and `%-I:%M %p` uses a 12-hour clock. Numeric and text directives are padded independently across the listing, keeping each date/time component aligned.

For a complete list of available colors, icons, and configuration options, please refer to [`config.example`](config.example).
