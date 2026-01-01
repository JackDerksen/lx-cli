# LX: A nicer way to list your files

A modern alternative to `ls` with more readable formatting, colours, icons, and customizable output.

## Features

- 🎨 **Colorized output** with file type icons
- 📊 **Multiple display modes**: short (default), long (`-l`), one-per-line (`-1`), and tree view (`-r`)
- 👻 **Hidden files support**: use `-a` to show all files
- 🌳 **Recursive tree view**: display directory hierarchies with `-r`
- ⚙️ **Configurable**: customize colors, spacing, display options, and tree styles
- 📏 **Smart alignment**: properly handles unicode characters and icons


## Demo

<img width="3024" height="1894" alt="CleanShot 2025-12-31 at 21 58 16@2x" src="https://github.com/user-attachments/assets/44e6b315-4679-4e73-b875-8eb044d3de8d" />
Basic formatting difference compared to the standard ls command

---
<img width="3024" height="1896" alt="CleanShot 2025-12-31 at 21 52 38@2x" src="https://github.com/user-attachments/assets/fc7415c7-0427-46cf-9b45-95b4a78cf1b4" />
Nicer long format compared to the standard ls command (displayed fields can be customized)

---
<img width="3024" height="1896" alt="CleanShot 2025-12-31 at 21 53 02@2x" src="https://github.com/user-attachments/assets/6651dcb4-e571-4245-9009-cb9d50b5c542" />
Recursive sub-directory tree view with the '-r' flag

---
<img width="3024" height="1900" alt="CleanShot 2025-12-31 at 21 53 35@2x" src="https://github.com/user-attachments/assets/68a16075-0d8b-4124-aec2-253740fa26ec" />
The '-r' flag can be combined with the '-l' long flag for viewing subdirectory info

---
<img width="3024" height="1898" alt="CleanShot 2025-12-31 at 21 54 04@2x" src="https://github.com/user-attachments/assets/e44422e9-6c69-4d34-9322-9449aab5c62f" />
'-1' flag for one-per-line output formatting


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
