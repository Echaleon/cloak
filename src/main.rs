use std::{ops::Deref, sync::Arc};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};

mod filesystem;
mod matcher;

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

    /// Set the number of threads to use. Defaults to the number of logical cores.
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
            .with_context(|| "Failed to build new threadpool".to_string())?;
    }

    // Get the paths to hide files and folders in. Needs to be arc because it is used in multiple threads.
    let paths = Arc::new(match opts.path {
        Some(paths) => paths,
        None => vec![".".to_string()],
    });

    // Get the types of objects to hide. Needs to be arc because it is used in multiple threads.
    let types = Arc::new(opts.types);

    // Build a matcher to match files and folders to hide, Needs to be arc because it is used in multiple threads.
    let matcher = Arc::new(matcher::Matcher::new(
        opts.pattern,
        opts.exclude,
        opts.regex,
        opts.regex_exclude,
    )?);

    // If the watch flag is set, then spawn a new thread to search for files and folders to hide.
    // Otherwise, just search for files and folders to hide.
    if opts.watch {
        {
            let paths = paths.clone();
            let matcher = matcher.clone();
            let types = types.clone();
            std::thread::spawn(move || {
                search(
                    paths.deref(),
                    matcher.deref(),
                    match types.deref() {
                        Some(types) => Some(types.deref()),
                        None => None,
                    },
                    opts.recursive,
                    opts.test,
                    opts.verbose,
                )
            });
        }
        watch(
            paths.deref(),
            matcher.deref(),
            match types.deref() {
                Some(types) => Some(types.deref()),
                None => None,
            },
            opts.recursive,
            opts.test,
            opts.verbose,
        )
    } else {
        search(
            paths.deref(),
            matcher.deref(),
            match types.deref() {
                Some(types) => Some(types.deref()),
                None => None,
            },
            opts.recursive,
            opts.test,
            opts.verbose,
        );
        Ok(())
    }
}

// Function to search for files and folders to hide
fn search(
    paths: &[String],
    matcher: &matcher::Matcher,
    types: Option<&[filesystem::ObjectType]>,
    recursive: bool,
    test: bool,
    verbose: bool,
) {
    // Iterate over the root paths using jwalk
    for path in paths {
        if verbose {
            println!("Searching for files and folders to hide in {}", path);
        }

        let mut walker = jwalk::WalkDir::new(path)
            .follow_links(true)
            .skip_hidden(false);

        if !recursive {
            walker = walker.max_depth(1);
        }

        // Now iterate over the files and folders, filtering out errors first, then filtering
        // by the types of objects to hide, then filtering by the matcher.
        for entry in walker
            .into_iter()
            .filter_map(|e| {
                // If there's an error, print it out and return None.
                e.with_context(|| "Failed to get path.".to_string())
                    .map_err(|e| eprintln!("{}", e))
                    .ok()
            })
            .filter(|e| match types {
                Some(types) => {
                    // If there's an error, print it out and return false.
                    filesystem::matches_type(e.path().deref(), types).unwrap_or_else(|e| {
                        eprintln!("{}", e);
                        false
                    })
                }
                None => true,
            })
            .filter(|e| {
                // If there's an error, print it out and return false. Otherwise, return the result of the matcher.
                e.path()
                    .to_str()
                    .map(|p| matcher.matches(p))
                    .unwrap_or_else(|| {
                        eprintln!(
                            "{}",
                            anyhow!("Failed to convert path {} to string", e.path().display())
                        );
                        false
                    })
            })
        {
            // If the test flag is set, then print out the path of the file or folder to hide.
            // Otherwise, hide the file or folder.
            if test {
                println!("Would hide {}", entry.path().display());
            } else {
                if verbose {
                    println!("Hiding {}", entry.path().display());
                }
                filesystem::hide(entry.path().deref()).unwrap();
            }
        }
    }
}

// Function to watch for changes and hide files and folders
fn watch(
    paths: &[String],
    matcher: &matcher::Matcher,
    types: Option<&[filesystem::ObjectType]>,
    recursive: bool,
    test: bool,
    verbose: bool,
) -> Result<()> {
    todo!()
}
