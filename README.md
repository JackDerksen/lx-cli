# LX: A nicer way to list your files

A modern alternative to `ls` with more readable formatting, colours, icons, and customizable output.

## Features

- 🎨 **Colorized output** with file type icons
- 📊 **Multiple display modes**: short (default), long (`-l`), one-per-line (`-1`), and tree view (`-r`)
- 👻 **Hidden files support**: use `-a` to show all files
- 🌳 **Recursive tree view**: display directory hierarchies with `-r`
- ⚙️ **Configurable**: customize colors, spacing, display options, and tree styles
- 📏 **Smart alignment**: properly handles unicode characters and icons

<img width="2092" height="1314" alt="CleanShot 2025-11-28 at 20 34 55@2x" src="https://github.com/user-attachments/assets/d15d1bc9-a496-4c94-9820-5f9d6b0025e4" />
<img width="2092" height="1536" alt="CleanShot 2025-11-28 at 20 35 30@2x" src="https://github.com/user-attachments/assets/9ede5bd4-7e08-48c8-8ced-1c210e025619" />

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

# Show directory tree recursively
lx -r

# Combine flags
lx -l -a size

# List files in a specific directory
lx /path/to/directory
```

## Flags

- `-l`, `--long`: Use a long listing format with detailed file information
- `-a`, `--all`: Show all files, including hidden ones (starting with `.`)
- `-1`: Force single column output (useful for piping to other commands)
- `-r`, `--recursive`: Show directory tree recursively with proper hierarchy

## Configuration

`lx` can be customized using a configuration file at `~/.config/lx/config`.

See [`config.example`](config.example) for all available options and detailed configuration examples.

### Configuration Sections

The configuration file supports the following sections:

- **`[colors]`**: Customize text colors for different file types
- **`[icons]`**: Set custom icons for different file types
- **`[icons.colors]`**: Customize colors for icons separately from filenames
- **`[display]`**: Control layout options like column spacing and multi-column wrapping
- **`[display.tree]`**: Control tree display style for recursive listings (`style = "ascii"` or `style = "indent"`)

For a complete list of available colors, icons, and configuration options, please refer to [`config.example`](config.example).
