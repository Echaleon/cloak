use crate::{filesystem, filter, matcher};
use anyhow::Context;
use rayon::prelude::*;
use std::path::Path;
use std::time::Duration;

pub fn search(
    paths: &[impl AsRef<Path> + Send + Sync + 'static],
    matcher: &matcher::Matcher,
    types: Option<&[filesystem::ObjectType]>,
    recursive: bool,
    test: bool,
    verbose: bool,
) {
    // Iterate over the root paths using jwalk
    paths.par_iter().for_each(|dir| {
        if verbose {
            println!(
                "Searching for files and folders to hide in {}...",
                dir.as_ref().display()
            );
        }

        // The rayon thread pool can get busy, so try to start iteration continuously until it succeeds.
        loop {
            match jwalk::WalkDir::new(dir)
                .follow_links(true)
                .skip_hidden(false)
                .parallelism(jwalk::Parallelism::RayonDefaultPool {
                    busy_timeout: Duration::from_secs(3),
                })
                .max_depth(if recursive { usize::MAX } else { 1 })
                .try_into_iter()
            {
                Ok(iter) => break iter,
                Err(_) if verbose => eprintln!(
                    "Failed to start iteration on path {}. Retrying...",
                    dir.as_ref().display()
                ),
                Err(_) => continue,
            };
        }
        // Now iterate over the files and folders, filtering out errors first, then filtering
        // by the types of objects to hide, then filtering by the matcher.
        .filter_map(|dir| {
            // If there's an error, print it out and return None.
            dir.with_context(|| "Failed to get path.")
                .inspect_err(|e| eprintln!("{e}"))
                .ok()
        })
        .filter(|dir| filter::file_type_matches(&dir.path(), types, verbose))
        .filter(|dir| filter::path_matches_pattern(&dir.path(), matcher, verbose))
        .for_each(|entry| {
            // If the test flag is set, then print out the path of the file or folder to hide.
            // Otherwise, hide the file or folder.
            if test {
                println!("Would hide {}", entry.path().display());
            } else {
                if verbose {
                    println!("Hiding {}", entry.path().display());
                }
                filesystem::hide(&entry.path()).unwrap_or_else(|e| eprintln!("{e}"));
            }
        });
    });
}
