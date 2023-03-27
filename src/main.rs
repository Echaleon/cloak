use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[clap(version)]
struct Opts {
    /// Flag for recursive search and watch
    /// (default: false)
    #[clap(short, long)]
    recursive: bool,

    /// Flag to watch for changes, rather than just run once
    /// (default: false)
    #[clap(short, long)]
    watch: bool,

    /// Flag to enable test mode, which will not actually hide files or folders.
    /// (default: false)
    #[clap(short = 'm', long)]
    test: bool,

    /// Flag to enable verbose mode, which will print out more information.
    /// (default: false)
    #[clap(short, long)]
    verbose: bool,

    /// Glob pattern to match files and folders to hide. Can be specified multiple times to add more patterns.
    /// By default, all files and folders are hidden.
    /// (default: ["*"])
    #[clap(short, long)]
    pattern: Option<Vec<String>>,

    /// Glob pattern to exclude files and folders from hiding. Can be specified multiple times to add more patterns.
    /// By default, no files or folders are excluded.
    /// (default: [])
    #[clap(short = 'x', long)]
    exclude: Option<Vec<String>>,

    /// Regex pattern to match files and folders to hide. Can be specified multiple times to add more patterns.
    /// Regex patterns are matched against the full path of the file or folder.
    /// They are matched after glob patterns.
    /// By default, all files and folders are hidden.
    /// (default: [".*"])
    #[clap(short = 'g', long)]
    regex: Option<Vec<String>>,

    /// Regex pattern to exclude files and folders from hiding. Can be specified multiple times to add more patterns.
    /// Regex patterns are matched against the full path of the file or folder.
    /// They are matched after glob patterns.
    /// By default, no files or folders are excluded.
    /// (default: [])
    #[clap(short = 'e', long)]
    regex_exclude: Option<Vec<String>>,

    /// Types of objects to hide. Can be specified multiple times to add more types.
    /// By default, all types are hidden.
    /// (default: ["file", "folder", "symlink"])
    #[clap(short, long)]
    types: Option<Vec<ObjectType>>,

    /// Path(s) to the directory to hide files and folders in. Defaults to the current directory.
    /// (default: ".")
    #[clap(value_parser)]
    path: Option<Vec<String>>,
}

// Enum of types of objects to hide
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ObjectType {
    File,
    Folder,
    Symlink,
}

fn main() {
    // TODO: Implement
}

