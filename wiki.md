# LX reference

[Back to the README](README.md) · [Example configuration](config.example) · [Default theme](themes/default.toml)

Detailed options and configuration for `lx`. For installation and common commands, start with the [README](README.md).

- [Flags](#flags)
- [Display modes](#display-modes)
- [Trees and Git ignore rules](#trees-and-git-ignore-rules)
- [Pipes and scripts](#pipes-and-scripts)
- [Git status](#git-status)
- [Symlinks](#symlinks)
- [Directory sizes](#directory-sizes)
- [Configuration](#configuration)
- [Themes](#themes)
- [Icons](#icons)
- [Dates and headings](#dates-and-headings)
- [Building from source](#building-from-source)

## Flags

- `-l`, `--long`: Use a long listing format with detailed file information
- `--total-size`: Use recursive directory contents totals in the Size column and for size sorting; scans all descendants
- `-g`, `--git`: Show Git status for entries and summarise changes beneath directories
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

## Display modes

`-l` and `-1` are treated as separate display modes, so they cannot be combined together. `-1` also cannot be combined with `-r`. `-c` is only available for short output, so it cannot be combined with `-l`, `-1`, or `-r`. Short flags can still be clustered in the usual Unix style, so combinations like `-la`, `-lr`, and `-alr` work as expected.

## Trees and Git ignore rules

Tree controls work with both short (`-r`) and long (`-lr`) output, alongside sorting, `-a`, exclusions, and `-g`. Without `--depth`, traversal is unlimited. The displayed tree stops at the depth limit and omits excluded directories. Directory contents totals and Git badges still include descendants beyond the displayed depth and beneath excluded directories. When contents totals are disabled or unused, directories beyond the depth limit or excluded from the tree are not opened by lx.

`--git-ignore` uses Git's ignore rules, including nested `.gitignore` files, exceptions, `.git/info/exclude`, and global exclusions. Tracked files remain visible even when an ignore pattern matches them. It also hides the repository's `.git` entry with `-a`, works in Git worktrees, and does not enable status badges unless `-g` is supplied. Outside a Git working tree it has no effect; inside one, Git must be available. An explicitly selected entry remains visible as the tree root even if ignored, but ignored directories are not expanded.

## Pipes and scripts

```bash
lx | head
lx --plain
lx --zero | xargs -0 -I {} printf '%s\n' '{}'
```

When stdout is a pipe or redirected to a file, the default listing and `-1` print raw filenames, one per line, without icons, colours, Git badges, titles, or padding. Explicit `-l`, `-c`, `-r`, or `--git` retain their requested display mode. Use `--plain` to guarantee plain output in any destination, or `--zero` for an unambiguous filename stream, including filenames containing newlines or non-UTF-8 bytes. Both modes honour sorting, hidden-file settings, and filters, and disable styling even when colour is forced through the environment.

`--plain` and `--zero` cannot be combined with each other or with `-l`, `-c`, `-r`, or `--git`. Directory listings emit entry names relative to the listed directory; a single-file listing retains the supplied path. To pass files from another directory to a command, run the pipeline inside that directory, for example `(cd src && lx --zero | xargs -0 -I {} printf '%s\n' '{}')`. Empty listings emit no separators. A consumer that closes early, such as `head`, causes lx to exit quietly and successfully.

## Git status

With `--git`, short listings show a status badge before each entry and long listings gain a `Git` column.

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

Git status is opt-in and uses one Git status query, limited to the listed directory, for the repository containing the target, including linked worktrees. It requires the `git` executable. Outside a repository, `--git` leaves the listing unchanged; if Git fails, lx reports a warning and prints the normal listing. Submodules receive the containing repository's summary rather than a separate status scan of their contents. Untracked files are enumerated to distinguish them from ignored files, which can add latency in large repositories.

## Symlinks

Symlinks display as `name -> target` in human-readable listings, using the target exactly as stored in the link. Missing targets and link loops add a `[broken]` marker. Long listings show the link's own metadata, with `l` as the permissions prefix. Directory links are displayed as links and are never followed by tree traversal, including when supplied as the listing target; append `/` or pass a path inside the linked directory to list its contents. Sorting, filtering, and Git badges refer to the link itself. Links count as files for `-f` and are excluded by `-d`.

## Directory sizes

Directory sizes default to the directory's own filesystem metadata size, without scanning its contents. Request contents totals for a single listing with `--total-size`:

```bash
lx -l --total-size /path/to/directory
lx -lr --total-size --depth 0 /path/to/directory # Show just the directory and its full total
lx -l --total-size -s size desc # Sort by contents size, largest first
```

Contents totals include hidden, excluded, and Git-ignored descendants. Empty directories show `0B`. Directory metadata is not added; regular files and symlinks contribute their own lengths, symlink targets are never followed, and each hard-linked name is counted separately. These totals describe content lengths rather than allocated disk space. A trailing `?` marks an incomplete total when an entry cannot be read or the total overflows.

With `--total-size`, totals are calculated when the Size column is displayed or sorting uses `size`, including plain output. Size sorting uses the displayed totals. Calculating a full total requires scanning every descendant, which can make listings of home directories or other large trees *extremely* slow. Totals are reused within each listing but are not cached between runs.

## Configuration

The default layout keeps icons off and uses underlined titles in long output. Columns appear as Permissions, Size, Modified, and Name; `--git` adds Git immediately before Name. Dates use `%-d %b %H:%M` (for example, `13 Oct 15:14`), and compact output wraps after ten rows. Existing configuration settings override these defaults. Add `"owner"` before `"size"` in `[display] long_format_fields` to show the Owner column before Size. Available fields are `permissions`, `nlink` (or `links`), `owner`, `group`, `size`, `modified`, `git`, and `filename`; lx reports unknown fields and leaves them out.

Configuration uses TOML at `~/.config/lx/config.toml`. The older extensionless `~/.config/lx/config` is still read when `config.toml` is absent. See [`config.example`](config.example) for layout and icon options.

## Themes

The built-in theme uses bold blue directories, bold red executables, plain white regular files, and grey underlined headers. Each metadata component can still be styled independently. The terminal background stays unchanged.

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
| `style.size.bytes`, `.kilobytes`, `.megabytes`, `.gigabytes`, `.terabytes` | `number`, `unit` |
| `style.users` | `user_root`, `user_you`, `user_other`, `group_root`, `group_yours`, `group_other` |
| `style.links` | `normal`, `multiple` |
| `style.git` | `clean`, `modified`, `staged`, `untracked`, `conflict`, `mixed` |
| Standalone tables | `style.header`, `style.date`, `style.tree`, `style.punctuation`, `style.symlink_path` |

File categories are recognised by filename or extension without reading contents. They overlay regular-file styles; executable and symlink styles take precedence. Hidden styling overlays the resulting filename style. Link names and targets have separate styles; broken targets use `filekinds.broken_symlink`. Group membership includes the effective and supplementary groups. Permission styling includes setuid, setgid, and sticky-bit markers.

General layout, sorting, icon glyphs, paths, and patterns belong in `config.toml`; theme files contain styles only.

Existing `[colors]`, `[colors.git]`, `[styles]`, `[icons.colors]`, `long_format_title_color`, `long_format_title_style`, and `[display.long_format_styles]` settings remain supported. Legacy column attributes apply only in long output; explicit inline style attributes take precedence. New themes should use `[style]`.

## Icons

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

## Dates and headings

Set `datetime_format` under `[display]` to change the long-format `modified` column. It uses Chrono’s strftime-style specifiers, so `%B %-d, %Y` renders a full month name (for example, `August 8, 2026`) and `%-I:%M %p` uses a 12-hour clock. Numeric and text directives are padded independently across the listing, keeping each date/time component aligned. An invalid format is reported and the default is used instead.

Long-format titles are enabled and underlined by default. Only the header text is underlined, leaving spacing between columns clear. Set `attributes = []` under `[style.header]` to remove underlining. Long and recursive long output use the same theme roles.

## Building from source

With Rust 1.85 or newer and Cargo installed:

```bash
git clone https://github.com/JackDerksen/lx-cli
cd lx-cli
cargo install --path .
```

For local development, `./install.sh` builds a release binary and copies it to `~/.local/bin/`. Make sure that directory is on your `PATH`; add the following to your shell configuration (`.bashrc`, `.zshrc`, etc.) if needed:

```bash
export PATH="$HOME/.local/bin:$PATH"
```
