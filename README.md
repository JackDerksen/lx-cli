# LX: A nicer way to list your files

A modern alternative to `ls` with more readable formatting, colours, optional icons, and customisable output.

## Features

- **Crazy fast**: just as fast (if not faster) compared to the stock `ls` command
- **Colourised output** with optional file type icons
- **Multiple display modes**: short (default), long (`-l`), one-per-line (`-1`), compact (`-c`), and recursive tree view (`-r`)
- **Configurable**: customise colours, font styles, hidden-file styling, spacing, display options, and tree styles
- **Smart alignment**: properly handles unicode characters and icons
- **Git status**: optional single-character status badges, including summaries of changes beneath directories


## Demo

<img width="3024" height="1900" alt="CleanShot 2026-08-08 at 18 32 07@2x" src="https://github.com/user-attachments/assets/ee340327-6152-4ddf-b25d-8681eb0744b6" />

Basic formatting difference compared to the standard `ls` command

---
<img width="3024" height="1900" alt="CleanShot 2026-08-08 at 18 33 05@2x" src="https://github.com/user-attachments/assets/17bf867c-faf7-483b-a7c7-906bd9d5a549" />

Nicer long format compared to the standard `ls` command (displayed fields can be customised)

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
- A [Nerd Font](https://www.nerdfonts.com/font-downloads) is optional if you enable Nerd Font icons; icons are disabled by default. Note that some terminals, such as Ghostty, already have icons built-in!

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

# Use compact columns, wrapping after ten rows by default
lx -c

# Show directory tree recursively
lx -r

# Limit the tree to two levels below the root
lx -r --depth 2

# Skip Git-ignored entries (combine with Git badges if desired)
lx -r --git-ignore
lx -lrG --depth 2 --git-ignore

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
- `--depth <LEVELS>`: Limit tree traversal; `0` shows only the root, `1` adds its immediate children (requires `-r`)
- `--git-ignore`: Omit Git-ignored entries and the repository's `.git` metadata (requires `-r`)

Tree controls work with both short (`-r`) and long (`-lr`) output, alongside sorting, `-a`, exclusions, and `-G`. Without `--depth`, traversal is unlimited. Directories beyond the depth limit or excluded from the tree are not opened by lx. Git badges still summarise all changes beneath a directory, including changes beyond the displayed depth.

`--git-ignore` uses Git's ignore rules, including nested `.gitignore` files, exceptions, `.git/info/exclude`, and global exclusions. Tracked files remain visible even when an ignore pattern matches them. It also hides the repository's `.git` entry with `-a`, works in Git worktrees, and does not enable status badges unless `-G` is supplied. Outside a Git working tree it has no effect; inside one, Git must be available. An explicitly selected entry remains visible as the tree root even if ignored, but ignored directories are not expanded.

`-l` and `-1` are treated as separate display modes, so they cannot be combined together. `-c` is only available for short output, so it cannot be combined with `-l`, `-1`, or `-r`. Short flags can still be clustered in the usual Unix style, so combinations like `-la`, `-lr`, and `-alr` work as expected.

When stdout is a pipe or redirected to a file, the default listing and `-1` print raw filenames, one per line, without icons, colours, Git badges, titles, or padding. Explicit `-l`, `-c`, `-r`, or `--git` retain their requested display mode. Use `--plain` to guarantee plain output in any destination, or `--zero` for an unambiguous filename stream, including filenames containing newlines or non-UTF-8 bytes. Both modes honour sorting, hidden-file settings, and filters, and disable styling even when colour is forced through the environment.

`--plain` and `--zero` cannot be combined with each other or with `-l`, `-c`, `-r`, or `--git`. Directory listings emit entry names relative to the listed directory; a single-file listing retains the supplied path. To pass files from another directory to a command, run the pipeline inside that directory, for example `(cd src && lx --zero | xargs -0 -I {} printf '%s\n' '{}')`. Empty listings emit no separators. A consumer that closes early, such as `head`, causes lx to exit quietly and successfully.

With `--git`, short listings show a status badge before each entry and long listings gain a `Git` column:

When `[icons] enabled = true`, changed Git entries use Nerd Font icons. Clean entries always show `-`; with icons disabled, all statuses use the original characters. Both forms share the same `style.git` colours and font attributes. Explicit file icon overrides do not change Git badges.

| Badge | Icon | Meaning | Default colour |
| --- | --- | --- | --- |
| `-` | `-` | No changes reported by Git (including ignored entries) | Light grey |
| `M` |  Pencil | Unstaged changes, including deletions | Yellow |
| `S` |  Circled check | Changes ready to commit, including additions and renames | Green |
| `?` |  Circled question | Not yet tracked by Git | Blue |
| `!` |  Warning | An unresolved merge conflict | Red |
| `*` |  Shuffle | Both staged and unstaged changes, or a directory containing different states | Orange |

Customise badges with semantic styles in your configuration or theme:

```toml
[style.git]
clean = { foreground = "#68656F" }
modified = { foreground = "#FFDA84", attributes = ["bold"] }
staged = { foreground = "#BCF092" }
```

Directories summarise changes anywhere beneath them, including deleted files. Any descendant conflict makes the directory show the conflict badge. These summaries include hidden and excluded descendants, so they describe repository state independently of listing filters.

Git status is opt-in and uses one Git status query for the repository containing the target, including linked worktrees. It requires the `git` executable. Outside a repository, `--git` leaves the listing unchanged; if Git fails, lx reports a warning and prints the normal listing. Submodules receive the containing repository's summary rather than a separate status scan of their contents. Untracked files are enumerated to distinguish them from ignored files, which can add latency in large repositories.

Symlinks display as `name -> target` in human-readable listings, using the target exactly as stored in the link. Missing targets and link loops add a `[broken]` marker. Long listings show the link's own metadata, with `l` as the permissions prefix. Directory links are displayed as links and are never followed by tree traversal, including when supplied as the listing target; append `/` or pass a path inside the linked directory to list its contents. Sorting, filtering, and Git badges refer to the link itself. Links count as files for `-f` and are excluded by `-d`.

## Configuration

The default layout keeps icons off and uses underlined titles in long output. Columns appear as Permissions, Size, Modified, and Name; `--git` adds Git immediately before Name. Dates use `%-d %b %H:%M` (for example, `13 Oct 15:14`), and compact output wraps after ten rows. Existing configuration settings override these defaults. Add `"owner"` before `"size"` in `[display] long_format_fields` to show the Owner column before Size.

Configuration uses TOML at `~/.config/lx/config.toml`. The older extensionless `~/.config/lx/config` is still read when `config.toml` is absent. See [`config.example`](config.example) for layout and icon options.

### Themes

The tool uses my custom theme by default. Directories are bold blue, executables bold red, regular files plain white, and headers grey and underlined. Each metadata component can still be styled independently. The terminal background stays unchanged.

Set `language_colors = true` under `[display]` to colour regular files by language or type (source, document, config, build, and so on) using the `style.file_type` roles. It is off by default.

Create `~/.config/lx/themes/my-theme.toml` and select it at the top of your configuration:

```toml
theme = "my-theme"

[display]
column_spacing = 2
```

Theme files contain only `[style]` tables. Start from [`themes/default.toml`](themes/default.toml), or define just the roles you want to change:

```toml
# ~/.config/lx/themes/my-theme.toml
[style.filekinds]
directory = { foreground = "#9BE3ED", attributes = ["bold"] }
executable = { foreground = "#FC808F", attributes = ["bold", "underline"] }

[style.permissions]
read = { foreground = "#BBE8EE" }
write = { foreground = "#FFC79E" }
execute = { foreground = "#D6D9FC" }

[style.header]
foreground = "#68656F"
attributes = ["underline"]
```

Use the same tables in `config.toml` for inline overrides. Precedence is built-in defaults, selected theme, explicit legacy settings, then inline `[style]` settings. Each property merges independently: omitted properties inherit, `attributes = []` removes inherited attributes, and `foreground = "default"` or `background = "default"` restores the terminal's colour. A missing or invalid theme reports a warning and keeps the built-in theme and your config overrides.

Styles accept `foreground`, optional `background`, and `attributes`. Colours support `#RGB`, `#RRGGBB`, six-digit hex without `#`, and the ANSI names `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, plus their `bright_` variants. Font attributes are `bold`, `dimmed`, `italic`, `underline`, `strikethrough`, and `reversed`. Attributes replace the inherited list. Colour and font styling honour `NO_COLOR`; plain and NUL output always remains undecorated. RGB colours and individual attributes depend on terminal support.

| Style table | Roles |
| --- | --- |
| `style.filekinds` | `regular`, `directory`, `executable`, `symlink`, `broken_symlink`, `pipe`, `socket`, `block_device`, `character_device`, `special` |
| `style.filenames` | `hidden` (inherits file-kind attributes unless explicitly replaced) |
| `style.file_type` | `image`, `video`, `audio`, `archive`, `document`, `source`, `config`, `build`, `temporary` |
| `style.icons` | `directory`, `executable`, `regular`, `hidden`, `symlink`, `broken_symlink` |
| `style.permissions` | `type`, `read`, `write`, `execute`, `special`, `absent` |
| `style.size.bytes`, `.kilobytes`, `.megabytes`, `.gigabytes` | `number`, `unit` |
| `style.users` | `user_root`, `user_you`, `user_other`, `group_root`, `group_yours`, `group_other` |
| `style.links` | `normal`, `multiple` |
| `style.git` | `clean`, `modified`, `staged`, `untracked`, `conflict`, `mixed` |
| Standalone tables | `style.header`, `style.date`, `style.tree`, `style.punctuation`, `style.symlink_path` |

File categories are recognised by filename or extension without reading contents. They overlay regular-file styles; executable and symlink styles take precedence. Hidden styling overlays the resulting filename style. Link names and targets have separate styles; broken targets use `filekinds.broken_symlink`. Group membership includes the effective and supplementary groups. Permission styling includes setuid, setgid, and sticky-bit markers.

Only fields that lx displays have theme roles; eza's repository branch, security-context, inode, and block fields are not part of this schema. General layout, sorting, icon glyphs, paths, and patterns remain in `config.toml` rather than theme files.

Existing `[colors]`, `[colors.git]`, `[styles]`, `[icons.colors]`, `long_format_title_color`, `long_format_title_style`, and `[display.long_format_styles]` settings remain supported. Legacy column attributes apply only in long output; explicit inline style attributes take precedence. New themes should use `[style]`.

### Icons and layout

Enable automatic [Nerd Font icons](https://www.nerdfonts.com/cheat-sheet) with:

```toml
[icons]
enabled = true
```

System folders get dedicated icons for Home, Desktop, Documents, Downloads, Music, Pictures, Movies/Videos, etc. where those folders are available. macOS also recognises Applications and Library; Linux recognises `/etc`, `/usr`, `/var`, `/tmp`, and `/home`. Linux user folders follow XDG settings, including renamed or relocated folders. Recognition uses the actual directory, so an unrelated folder named `Documents` keeps the generic folder icon. Symlinks retain their link icon.

Standard hidden folders in your home directory also get dedicated icons: `.config`, `.cache`, `.local`, `.Trash`, and `.ssh`. Linux additionally recognises the XDG data, state, executable, and trash directories, including relocated paths. Use `-a` to include hidden entries in listings. Both automatic and custom icons use `style.icons.hidden`.

Common development folders get icons by name wherever they appear: `.git`, `.github`, `src`, `source`, `sources`, `lib`, `test`, `tests`, `target`, `build`, `docs`, `assets`, `images`, `vendor`, `scripts`, `bin`, etc. These defaults apply to directories when automatic icons are enabled; files and symlinks keep their own icon selection.

Licence files named `LICENSE`, `LICENCE`, `COPYING`, or `COPYRIGHT` use the `\uE60A` icon when automatic icons are enabled. README files use the book icon (`\uF02D`). Matching ignores case and accepts extensions or suffixes separated by `.`, `-`, or `_`, such as `LICENSE.md`, `LICENSE-MIT`, and `README.md`. Custom global, path, and pattern icons still take precedence; symlinks keep their link icon.

Automatic file icons also recognise common source languages (including Rust, Python, JavaScript, TypeScript, React, Go, C/C++, Java, Ruby, PHP, Lua, Swift, Kotlin, and shell scripts), web styles, configuration formats, Markdown, text, PDF and office documents, images, audio/video, archives, databases, and fonts. Key filenames such as `Cargo.toml`, `package.json`, `pyproject.toml`, Makefiles, Dockerfiles, Compose files, `.env` variants, Git settings, shell startup files, and common dependency lockfiles get dedicated icons. `CHANGELOG`, `CONTRIBUTING`, and `AUTHORS` share the book icon. Filename-specific choices take precedence over extension icons; unknown files retain the generic file or executable icon. Recognition uses filenames only and does not read file contents.

Automatic icons work in short, compact, long, and recursive listings. In long output, icons sit immediately beside filenames within the `Name` column. Older `icon` entries in `long_format_fields` are ignored. Plain and NUL output remain undecorated.

Any explicit file-type icon overrides the automatic choice globally. For example, `directory = "📁"` replaces every directory icon, including system folders, and `directory = ""` hides them all. Omit a setting to use its automatic default. Existing custom icons still work without `enabled = true`; `enabled` controls the automatic defaults. Icon colours and attributes use `[style.icons]`.

Override individual paths under `[icons.paths]`:

```toml
[icons.paths]
"~/Documents" = "📚"
"~/.config" = "⚙️"
"~/.Trash" = "🗑️"
"projects" = "💻"
"/srv/archive" = ""
```

Global file-type settings take precedence over exact path overrides, followed by pattern rules, system-folder icons, common directory names, and generic icons. In particular, a global `directory` setting still replaces every directory icon, including those in `[icons.paths]`. Path overrides also work when automatic icons are disabled. Paths can be absolute, use `~/`, or be relative to your home directory. Each setting applies to the configured filesystem entry, including when listed using a different path to the same entry; it does not extend to children. Symlink settings affect the link itself. An empty icon hides that entry's icon. Missing or inaccessible paths are ignored when configuration is loaded.

Pattern rules are tried in configuration order; the first matching rule wins, including a rule with an empty icon:

```toml
[[icons.patterns]]
pattern = "*.rs"
kind = "file"
icon = "🦀"

[[icons.patterns]]
pattern = "*.md"
kind = "file"
icon = "📝"

[[icons.patterns]]
pattern = "*/projectName/notes/"
icon = "📓"
```

Patterns support `*` for any number of characters and `?` for one character, with case-sensitive matching. A pattern without a path separator matches the entry's name; a pattern containing `/` matches its absolute listing path. `*` can cross path separators. A trailing `/` restricts the rule to directories and is removed before matching, so `src/` matches directories named `src` at any depth. Rules can also set `kind` to `file`, `directory`, or `any` (the default). `file` includes symlinks, consistent with `-f`; patterns match a link's name or path rather than its target. Pattern rules work with automatic icons disabled and keep icons inline with filenames, including tree output.

Set `datetime_format` under `[display]` to change the long-format `modified` column. It uses Chrono’s strftime-style specifiers, so `%B %-d, %Y` renders a full month name (for example, `August 8, 2026`) and `%-I:%M %p` uses a 12-hour clock. Numeric and text directives are padded independently across the listing, keeping each date/time component aligned.

Long-format titles are enabled and underlined by default. Only the header text is underlined, leaving spacing between columns clear. Set `attributes = []` under `[style.header]` to remove underlining. Long and recursive long output use the same theme roles.

For the complete built-in theme and general configuration options, see [`themes/default.toml`](themes/default.toml) and [`config.example`](config.example).
