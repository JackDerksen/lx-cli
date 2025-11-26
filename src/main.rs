use clap::Parser;
use colored::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    target: String,

    #[arg(short, long, help = "Use a long listing format")]
    long: bool,
}

enum FileIcon {
    Directory,
    Executable,
    RegularFile,
}

impl FileIcon {
    fn as_str(&self) -> &str {
        match self {
            FileIcon::Directory => " ",
            FileIcon::Executable => " ",
            FileIcon::RegularFile => " ",
        }
    }
}

#[derive(Debug)]
struct FileEntry {
    path: std::path::PathBuf,
    is_dir: bool,
    is_executable: bool,
}

impl FileEntry {
    fn get_icon(&self) -> &str {
        if self.is_dir {
            FileIcon::Directory.as_str()
        } else if self.is_executable {
            FileIcon::Executable.as_str()
        } else {
            FileIcon::RegularFile.as_str()
        }
    }

    fn get_color(&self) -> Color {
        if self.is_dir {
            Color::Blue
        } else if self.is_executable {
            Color::Green
        } else {
            Color::White
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let target_path = Path::new(&args.target);

    if !target_path.exists() {
        eprintln!(
            "lx: cannot access '{}': No such file or directory",
            args.target
        );
        return Ok(());
    }

    // Collect entries with metadata
    let mut entries: Vec<FileEntry> = Vec::new();
    for entry in fs::read_dir(target_path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let is_executable = !is_dir && (metadata.permissions().mode() & 0o111) != 0;

        entries.push(FileEntry {
            path,
            is_dir,
            is_executable,
        });
    }

    if args.long {
        // Long format (-l): show everything with details
        let mut max_path_width = 0;
        for entry in &entries {
            let path_name = entry.path.to_string_lossy();
            if path_name.len() > max_path_width {
                max_path_width = path_name.len();
            }
        }
        let padded_width = max_path_width + 2;

        for entry in entries {
            let path_name = entry.path.to_string_lossy();
            let permissions = "drwxr-xr-x"; // Get from metadata properly
            let icon = entry.get_icon();
            let filename = entry.path.file_name().unwrap().to_string_lossy();
            let color = entry.get_color();

            let path_column = format!("{:<width$}", path_name, width = padded_width);

            println!(
                "{}{} {} {}",
                path_column,
                permissions,
                icon,
                filename.color(color).bold()
            );
        }
    } else {
        // Standard format: organize by type in columns
        let mut directories: Vec<&FileEntry> = Vec::new();
        let mut executables: Vec<&FileEntry> = Vec::new();
        let mut regular_files: Vec<&FileEntry> = Vec::new();

        for entry in &entries {
            if entry.is_dir {
                directories.push(entry);
            } else if entry.is_executable {
                executables.push(entry);
            } else {
                regular_files.push(entry);
            }
        }

        // Print directories
        if !directories.is_empty() {
            println!("{}", "Directories:".bold().underline());
            for entry in directories {
                let filename = entry.path.file_name().unwrap().to_string_lossy();
                println!(
                    "  {} {}",
                    entry.get_icon(),
                    filename.color(entry.get_color()).bold()
                );
            }
            println!();
        }

        // Print executables
        if !executables.is_empty() {
            println!("{}", "Executables:".bold().underline());
            for entry in executables {
                let filename = entry.path.file_name().unwrap().to_string_lossy();
                println!(
                    "  {} {}",
                    entry.get_icon(),
                    filename.color(entry.get_color()).bold()
                );
            }
            println!();
        }

        // Print regular files
        if !regular_files.is_empty() {
            println!("{}", "Files:".bold().underline());
            for entry in regular_files {
                let filename = entry.path.file_name().unwrap().to_string_lossy();
                println!(
                    "  {} {}",
                    entry.get_icon(),
                    filename.color(entry.get_color())
                );
            }
        }
    }

    Ok(())
}
