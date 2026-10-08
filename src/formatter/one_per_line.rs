use crate::config::Config;
use crate::file_entry::{FileEntry, FileType};
use crate::formatter::format_short_entry;
use crate::sort::{SortOptions, sort_entries};

pub fn format_one_per_line(mut entries: Vec<FileEntry>, config: &Config, sort: SortOptions) {
    sort_entries(&mut entries, sort);

    if sort.is_custom() {
        print_entries(&entries, config);
        return;
    }

    let mut directories: Vec<FileEntry> = Vec::new();
    let mut executables: Vec<FileEntry> = Vec::new();
    let mut regular_files: Vec<FileEntry> = Vec::new();

    for entry in entries {
        match entry.get_file_type() {
            FileType::Directory => directories.push(entry),
            FileType::Executable => executables.push(entry),
            FileType::RegularFile => regular_files.push(entry),
        }
    }

    print_entries(&directories, config);
    print_entries(&executables, config);
    print_entries(&regular_files, config);
}

fn print_entries(entries: &[FileEntry], config: &Config) {
    for entry in entries {
        println!("{}", format_short_entry(entry, config));
    }
}
