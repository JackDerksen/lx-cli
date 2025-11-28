# LX: A nicer way to list your files

A modern alternative to `ls` with more readable formatting, colours, icons, and customizable output.

## Features

- 🎨 **Colorized output** with file type icons
- 📊 **Multiple display modes**: short (default) and long (`-l`)
- 👻 **Hidden files support**: use `-a` to show all files
- ⚙️ **Configurable**: customize colors, spacing, and display options
- 📏 **Smart alignment**: properly handles unicode characters and icons

## Requirements

- Rust (>= 1.56.1) and Cargo
- Either a [nerd font](https://www.nerdfonts.com/font-downloads) or a terminal like Ghostty which has nerd font icons pre-installed.

## Installation

The easiest way to install `lx` is using Cargo:

```bash
cargo install lx-cli
```

This will install the `lx` binary to `~/.cargo/bin/`, which is typically already in your `PATH`.

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

## Quick Start

After installation, simply run:

```bash
lx
```

## Usage

`lx` is used pretty much just like regular old `ls`. There are only a couple of flags at the moment, but more are on the way!

```bash
# List files in current directory
lx

# List files in long format
lx -l

# Show hidden files
lx -a

# Combine flags
lx -la

# List files in a specific directory
lx /path/to/directory
```

## Configuration

`lx` can be customized using a configuration file at `~/.config/lx/config`.

See `config.example` for all available options.

### Example Configuration 
### (this is different from default)

```toml
[colors]
directory = "blue"
executable = "red"
regular = "white"

[display]
column_spacing = 3
max_rows = 7
```

### Available Colors

- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`

### Configuration Options

#### `[colors]`
- `directory`: Color for directory names (default: `blue`)
- `executable`: Color for executable files (default: `green`)
- `regular`: Color for regular files (default: `white`)

#### `[display]`
- `column_spacing`: Number of spaces between columns (default: `2`)
- `max_rows`: Maximum number of rows before wrapping to next column in short format. Set to `0` for no limit (default: `0`)
  - When set, each file type (directories, executables, regular files) will wrap into multiple columns after reaching the max row count
  - For example, with `max_rows = 5` and 12 directories, they will be displayed in 3 columns with 5 rows each
  - File type separation is maintained - directories, executables, and files are kept in their own sections
  - Only applies to short format (default view), not long format (`-l`)
