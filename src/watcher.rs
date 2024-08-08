use crate::{filesystem, filter, matcher};
use anyhow::{anyhow, Context, Result};
use notify::{event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

// Function to watch for changes and hide files and folders
pub fn watch(
    paths: &[String],
    matcher: &matcher::Matcher,
    types: Option<&[filesystem::ObjectType]>,
    recursive: bool,
    test: bool,
    verbose: bool,
) -> Result<()> {
    rayon::scope(|s| {
        // Open a channel to receive events from the watcher
        let (tx, rx) = std::sync::mpsc::channel();

        // Create a new watcher
        let mut watcher: RecommendedWatcher = Watcher::new(tx, notify::Config::default())
            .with_context(|| {
                "Failed to create new watcher. Make sure you have the required permissions."
            })?;

        // Add the paths to watch to the watcher
        for path in paths {
            watcher
                .watch(
                    Path::new(path),
                    if recursive {
                        RecursiveMode::Recursive
                    } else {
                        RecursiveMode::NonRecursive
                    },
                )
                .with_context(|| {
                    format!(
                        "Failed to watch path {path}. Make sure you have the required permissions"
                    )
                })?;
        }

        // Begin looping infinitely through the events received from the watcher
        loop {
            let event = rx.recv().with_context(|| "Critical error in watcher")?;

            // If the event is an error, print it out and continue to the next event, otherwise
            // pass the event to the rayon thread pool to handle.
            match event {
                Ok(event) => {
                    s.spawn(move |_| {
                        handle_event(&event, matcher, types, test, verbose);
                    });
                }
                Err(e) => eprintln!("{e}"),
            }
        }
    })
}

// Helper function for the watch function that is run on the rayon thread pool. It does the actual
// handling of the events.
fn handle_event(
    event: &notify::Event,
    matcher: &matcher::Matcher,
    types: Option<&[filesystem::ObjectType]>,
    test: bool,
    verbose: bool,
) {
    // Get the path from the event. If an event is not one that is supposed to be handled, then
    // return early. If the path is not found, then print out an error and return early.
    let path = match get_path(event) {
        Some(Ok(path)) => path,
        Some(Err(e)) => {
            eprintln!("{e}");
            return;
        }
        None => return,
    };

    // Check if the path matches the types of objects to hide.
    if !filter::file_type_matches(&path, types, verbose) {
        return;
    }

    // Check if the path matches the matcher.
    if !filter::path_matches_pattern(&path, matcher, verbose) {
        return;
    }

    // If the test flag is set, then print out the path of the file or folder to hide.
    // Otherwise, hide the file or folder.
    if test {
        println!("Would hide {}", path.display());
    } else {
        if verbose {
            println!("Hiding {}", path.display());
        }
        filesystem::hide(&path).unwrap_or_else(|e| eprintln!("{e}"));
    }
}

// Get the path from an event. Returns an error if the event is one that is supposed to be handled
// but the path is not found.
fn get_path(event: &notify::Event) -> Option<Result<&PathBuf>> {
    if matches!(event.kind, event::EventKind::Create(_)) {
        Some(
            event
                .paths
                .get(0)
                .ok_or_else(|| anyhow!("Failed to get path from event")),
        )
    } else if matches!(
        event.kind,
        event::EventKind::Modify(event::ModifyKind::Name(_))
    ) && !matches!(
        event.kind,
        event::EventKind::Modify(event::ModifyKind::Name(event::RenameMode::From))
    ) {
        Some(
            event
                .paths
                .get(1)
                .or_else(|| event.paths.get(0))
                .ok_or_else(|| anyhow!("Failed to get path from event")),
        )
    } else {
        None
    }
}
