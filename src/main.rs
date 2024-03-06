use anyhow::{Context, Result};
use clap::Parser;

mod filesystem;
mod filter;
mod matcher;
mod search;
mod watcher;

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
    /// These are matched after glob and regex exclude patterns, but before regex patterns.
    /// By default, all files and folders are hidden.
    /// (default: ["*"])
    #[clap(short, long)]
    pattern: Option<Vec<String>>,

    /// Glob pattern to exclude files and folders from hiding. Can be specified multiple times to add more patterns.
    /// These are matched first, before regex exclude patterns, and glob and regex patterns.
    /// By default, no files or folders are excluded.
    /// (default: [])
    #[clap(short = 'x', long)]
    exclude: Option<Vec<String>>,

    /// Regex pattern to match files and folders to hide. Can be specified multiple times to add more patterns.
    /// Regex patterns are matched against the full path of the file or folder.
    /// They are matched last, after glob and regex exclude patterns, and glob patterns.
    /// By default, all files and folders are hidden.
    /// (default: [".*"])
    #[clap(short = 'g', long)]
    regex: Option<Vec<String>>,

    /// Regex pattern to exclude files and folders from hiding. Can be specified multiple times to add more patterns.
    /// Regex patterns are matched against the full path of the file or folder.
    /// They are matched after glob exclude patterns, but before glob and regex patterns.
    /// By default, no files or folders are excluded.
    /// (default: [])
    #[clap(short = 'e', long)]
    regex_exclude: Option<Vec<String>>,

    /// Types of objects to hide. Can be specified multiple times to add more types.
    /// By default, all types are hidden.
    /// (default: ["file", "folder", "symlink"])
    #[clap(short, long)]
    types: Option<Vec<filesystem::ObjectType>>,

    /// Set the number of threads to use in the thread pool. Still will spawn a small number of threads for other tasks.
    /// (default: number of logical cores)
    #[clap(short = 'j', long)]
    threads: Option<usize>,

    /// Path(s) to the directory to hide files and folders in. Defaults to the current directory.
    /// (default: ".")
    #[clap(value_parser)]
    path: Option<Vec<String>>,
}

fn main() -> Result<()> {
    // Parse the command line arguments
    let opts: Opts = Opts::parse();

    // Set a new global threadpool with the number of threads specified by the user.
    if let Some(threads) = opts.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .with_context(|| "Failed to build new threadpool")?;
    }

    // Get the paths to hide files and folders in.
    let paths = opts
        .path
        .map_or_else(|| vec![".".to_owned()], |paths| paths);

    // Build a matcher to match files and folders to hide
    let matcher =
        matcher::Matcher::new(opts.pattern, opts.exclude, opts.regex, opts.regex_exclude)?;

    // If the watch flag is set, then spawn a new thread to search for files and folders to hide.
    // Otherwise, just search for files and folders to hide.
    if opts.watch {
        std::thread::scope(|s| {
            s.spawn(|| {
                search::search(
                    &paths,
                    &matcher,
                    opts.types.as_deref(),
                    opts.recursive,
                    opts.test,
                    opts.verbose,
                );
            });
            watcher::watch(
                &paths,
                &matcher,
                opts.types.as_deref(),
                opts.recursive,
                opts.test,
                opts.verbose,
            )
        })
    } else {
        search::search(
            &paths,
            &matcher,
            opts.types.as_deref(),
            opts.recursive,
            opts.test,
            opts.verbose,
        );
        Ok(())
    }
}
