use anyhow::{Context, Result};
use globset::GlobSet;
use regex::RegexSet;

pub struct Matcher {
    globs: Option<GlobSet>,
    globs_exclude: Option<GlobSet>,
    regexes: Option<RegexSet>,
    regexes_exclude: Option<RegexSet>,
}

impl Matcher {
    // Build a new matcher.
    pub fn new(
        globs: Option<Vec<String>>,
        globs_exclude: Option<Vec<String>>,
        regexes: Option<Vec<String>>,
        regexes_exclude: Option<Vec<String>>,
    ) -> Result<Matcher> {
        Ok(Matcher {
            globs: match globs {
                Some(globs) => {
                    let mut builder = globset::GlobSetBuilder::new();
                    for glob in globs {
                        builder.add(
                            globset::Glob::new(&glob).with_context(|| {
                                format!("Failed to parse glob pattern {}", glob)
                            })?,
                        );
                    }
                    Some(
                        builder
                            .build()
                            .with_context(|| "Failed to build glob matcher".to_string())?,
                    )
                }
                None => None,
            },
            globs_exclude: match globs_exclude {
                Some(globs_exclude) => {
                    let mut builder = globset::GlobSetBuilder::new();
                    for glob in globs_exclude {
                        builder.add(globset::Glob::new(&glob).with_context(|| {
                            format!("Failed to parse glob exclude pattern {}", glob)
                        })?);
                    }
                    Some(
                        builder
                            .build()
                            .with_context(|| "Failed to build glob exclude matcher".to_string())?,
                    )
                }
                None => None,
            },
            regexes: match regexes {
                Some(regexes) => Some(
                    RegexSet::new(regexes)
                        .with_context(|| "Failed to build regex matcher".to_string())?,
                ),
                None => None,
            },
            regexes_exclude: match regexes_exclude {
                Some(regexes_exclude) => Some(
                    RegexSet::new(regexes_exclude)
                        .with_context(|| "Failed to build regex exclude matcher".to_string())?,
                ),
                None => None,
            },
        })
    }

    // Check if a path matches the matcher. If there are no patterns, then the path matches.
    pub fn matches(&self, path: &str) -> bool {
        if self.globs.is_none()
            && self.globs_exclude.is_none()
            && self.regexes.is_none()
            && self.regexes_exclude.is_none()
        {
            return true;
        }

        // Check if the path matches any of the glob exclude patterns
        if let Some(globs_exclude) = &self.globs_exclude {
            if globs_exclude.is_match(path) {
                return false;
            }
        }

        // Check if the path matches any of the regex exclude patterns
        if let Some(regexes_exclude) = &self.regexes_exclude {
            if regexes_exclude.is_match(path) {
                return false;
            }
        }

        // Check if the path matches any of the glob patterns
        if let Some(globs) = &self.globs {
            if globs.is_match(path) {
                return true;
            }
        }

        // Check if the path matches any of the regex patterns
        if let Some(regexes) = &self.regexes {
            if regexes.is_match(path) {
                return true;
            }
        }

        false
    }
}

