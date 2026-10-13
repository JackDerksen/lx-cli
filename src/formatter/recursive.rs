use crate::config::Config;
use crate::file_entry::FileEntry;
use crate::filter::EntryFilter;
use crate::formatter::long::{
    calculate_column_widths_with_filename_prefixes, print_long_entries_with_filename_prefixes,
};
use crate::formatter::print_line;
use crate::formatter::tree::{TreeEntry, TreeRenderer};
use crate::formatter::{format_long, format_one_per_line, format_short_entry};
use crate::git::GitRepository;
use crate::reader::{MetadataMode, read_entry, read_target};
use crate::sort::{DefaultSort, SortOptions};
use std::io;
use std::path::Path;

pub struct TreeOptions<'a> {
    pub show_hidden: bool,
    pub use_long_format: bool,
    pub depth: Option<usize>,
    pub ignored: Option<&'a GitRepository>,
}

pub fn format_recursive(
    path: &Path,
    config: &Config,
    options: &TreeOptions<'_>,
    filter: &EntryFilter,
    sort: SortOptions,
    git: Option<&GitRepository>,
) -> io::Result<()> {
    let metadata_mode = if options.use_long_format || sort.requires_full_metadata() {
        MetadataMode::Full
    } else {
        MetadataMode::Basic
    };
    if !path.is_dir() || path.is_symlink() {
        let mut entries = filter.apply(read_target(path, options.show_hidden, metadata_mode)?);
        if let Some(repository) = git {
            repository.annotate(path, &mut entries)?;
        }

        if options.use_long_format {
            format_long(entries, config, sort)?;
        } else {
            format_one_per_line(entries, config, sort)?;
        }
        return Ok(());
    }

    let mut root = read_entry(path, metadata_mode)?;
    if let Some(repository) = git {
        root.git_status = Some(repository.status(&path.canonicalize()?));
    }
    if !filter.includes(&root) {
        return Ok(());
    }

    let uses_icons = !options.use_long_format
        && [
            config.icons.directory.as_str(),
            config.icons.executable.as_str(),
            config.icons.regular.as_str(),
            config.icons.symlink.as_str(),
        ]
        .iter()
        .any(|icon| !icon.is_empty());
    let default_sort = if options.use_long_format {
        DefaultSort::TypeThenName
    } else {
        DefaultSort::Name
    };
    let renderer = TreeRenderer::new(
        &config.display.tree.style,
        uses_icons,
        filter,
        sort,
        default_sort,
        git,
        options,
    );
    let tree_entries = renderer.collect(path, metadata_mode)?;

    if options.use_long_format {
        print_nested_long_tree(&root, &tree_entries, config)?;
    } else {
        print_tree_root(&root, config)?;
        print_short_tree(&tree_entries, config)?;
    }

    Ok(())
}

fn print_nested_long_tree(
    root: &FileEntry,
    tree_entries: &[TreeEntry],
    config: &Config,
) -> io::Result<()> {
    let mut display_entries = Vec::with_capacity(tree_entries.len() + 1);
    let mut filename_prefixes = vec![String::new()];
    display_entries.push(display_entry(root));
    display_entries.extend(
        tree_entries
            .iter()
            .map(|tree_entry| display_entry(&tree_entry.entry)),
    );
    filename_prefixes.extend(
        tree_entries
            .iter()
            .map(|tree_entry| format!("{} ", tree_entry.branch)),
    );

    let fields = &config.display.long_format_fields;
    let widths = calculate_column_widths_with_filename_prefixes(
        &display_entries,
        &filename_prefixes,
        fields,
        config,
    );
    print_long_entries_with_filename_prefixes(
        &display_entries,
        &filename_prefixes,
        config,
        fields,
        &widths,
    )
}

fn print_tree_root(root: &FileEntry, config: &Config) -> io::Result<()> {
    print_line(format_args!(
        "{}",
        format_short_entry(&display_entry(root), config)
    ))
}

fn print_short_tree(tree_entries: &[TreeEntry], config: &Config) -> io::Result<()> {
    for tree_entry in tree_entries {
        print_line(format_args!(
            "{} {}",
            tree_entry.branch,
            format_short_entry(&display_entry(&tree_entry.entry), config)
        ))?;
    }
    Ok(())
}

fn display_entry(entry: &FileEntry) -> FileEntry {
    let mut display_entry = entry.clone();
    if display_entry.is_dir {
        display_entry.path.push("/");
    }
    display_entry
}
