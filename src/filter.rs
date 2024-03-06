use std::path::Path;
use crate::filesystem;
use crate::filesystem::ObjectType;
use crate::matcher::Matcher;

// Handler function to check if a path matches the given file_types, handling errors and printing out verbose messages,
// as necessary.
pub fn file_type_matches(path: &Path, types: Option<&[ObjectType]>, verbose: bool) -> bool {
    types.map_or(true, |types| {
        // If there's an error, print it out and return false.
        filesystem::matches_type(path, types)
            .inspect(|r| {
                if verbose && !r {
                    println!(
                        "Skipping {} because it's not a file or folder",
                        path.display()
                    );
                }
            })
            .inspect_err(|e| eprintln!("{e}"))
            .unwrap_or(false)
    })
}

// Helper function to check if a path matches the given matcher
pub fn path_matches_pattern(path: &Path, matcher: &Matcher, verbose: bool) -> bool {
    let res = matcher.matches(path);
    if verbose {
        if let Some(path) = res.lossy {
            eprintln!("Path {path} is not valid UTF-8. This may cause issues.");
        }
        if !res.result {
            if let Some(matcher_type) = res.matcher_type {
                println!(
                    "Skipping {} because it is excluded by a {matcher_type} pattern", path.display()
                );
            } else {
                println!(
                    "Skipping {} because it did not match any patterns", path.display()
                );
            }
        }
    }
    res.result
}