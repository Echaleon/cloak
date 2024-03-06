use anyhow::{Context, Result};
use globset::GlobSet;
use regex::RegexSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Matcher {
    globs: Option<GlobSet>,
    globs_exclude: Option<GlobSet>,
    regexes: Option<RegexSet>,
    regexes_exclude: Option<RegexSet>,
}

// The result of a match, including the type of matcher that matched. Lossy holds the result of converting the path to a string
// if there was a lossy conversion. Globs can match on full paths, but lossy will still hold a string if the path was not a valid
// UTF-8 string for printing purposes.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MatchResult {
    pub result: bool,
    pub matcher_type: Option<MatcherType>,
    pub lossy: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MatcherType {
    Glob,
    Regex,
}

impl Matcher {
    // Build a new matcher.
    pub fn new(
        globs: Option<Vec<String>>,
        globs_exclude: Option<Vec<String>>,
        regexes: Option<Vec<String>>,
        regexes_exclude: Option<Vec<String>>,
    ) -> Result<Self> {
        Ok(Self {
            globs: match globs {
                Some(globs) => {
                    let mut builder = globset::GlobSetBuilder::new();
                    for glob in globs {
                        builder.add(
                            globset::Glob::new(&glob)
                                .with_context(|| format!("Failed to parse glob pattern {glob}"))?,
                        );
                    }
                    Some(
                        builder
                            .build()
                            .with_context(|| "Failed to build glob matcher")?,
                    )
                }
                None => None,
            },
            globs_exclude: match globs_exclude {
                Some(globs_exclude) => {
                    let mut builder = globset::GlobSetBuilder::new();
                    for glob in globs_exclude {
                        builder.add(globset::Glob::new(&glob).with_context(|| {
                            format!("Failed to parse glob exclude pattern {glob}")
                        })?);
                    }
                    Some(
                        builder
                            .build()
                            .with_context(|| "Failed to build glob exclude matcher")?,
                    )
                }
                None => None,
            },
            regexes: match regexes {
                Some(regexes) => {
                    Some(RegexSet::new(regexes).with_context(|| "Failed to build regex matcher")?)
                }
                None => None,
            },
            regexes_exclude: match regexes_exclude {
                Some(regexes_exclude) => Some(
                    RegexSet::new(regexes_exclude)
                        .with_context(|| "Failed to build regex exclude matcher")?,
                ),
                None => None,
            },
        })
    }

    // Check if a path matches the matcher. If there are no patterns, then the path matches.
    pub fn matches(&self, path: &Path) -> MatchResult {
        // Regex patterns need strings, so convert the path to a string. If there is a lossy conversion, then store the
        // lossy string, and set the lossy flag to true.
        let (path_str, lossy) = path
            .to_str()
            .map_or_else(|| (path.to_string_lossy(), true), |s| (s.into(), false));
        
        // Short-circuit if there are no patterns
        if self.globs.is_none()
            && self.globs_exclude.is_none()
            && self.regexes.is_none()
            && self.regexes_exclude.is_none()
        {
            return MatchResult {
                result: true,
                matcher_type: None,
                lossy: if lossy { Some(path_str.into()) } else { None },
            };
        }
        
        // Check if the path matches any of the glob exclude patterns
        if let Some(globs_exclude) = self.globs_exclude.as_ref() {
            if globs_exclude.is_match(path) {
                return MatchResult {
                    result: false,
                    matcher_type: Some(MatcherType::Glob),
                    lossy: if lossy { Some(path_str.into()) } else { None },
                };
            }
        }

        // Check if the path matches any of the regex exclude patterns
        if let Some(regexes_exclude) = self.regexes_exclude.as_ref() {
            if regexes_exclude.is_match(&path_str) {
                return MatchResult {
                    result: false,
                    matcher_type: Some(MatcherType::Regex),
                    lossy: if lossy { Some(path_str.into()) } else { None },
                };
            }
        }

        // Check if the path matches any of the glob patterns
        if let Some(globs) = self.globs.as_ref() {
            if globs.is_match(path) {
                return MatchResult {
                    result: true,
                    matcher_type: Some(MatcherType::Glob),
                    lossy: if lossy { Some(path_str.into()) } else { None },
                };
            }
        }

        // Check if the path matches any of the regex patterns
        if let Some(regexes) = self.regexes.as_ref() {
            if regexes.is_match(&path_str) {
                return MatchResult {
                    result: true,
                    matcher_type: Some(MatcherType::Regex),
                    lossy: if lossy { Some(path_str.into()) } else { None },
                };
            }
        }

        // If the path didn't match any of the patterns, then it doesn't match
        MatchResult {
            result: false,
            matcher_type: None,
            lossy: if lossy { Some(path_str.into()) } else { None },
        }
    }
}

// Automatically convert a MatchResult to a bool.
impl From<MatchResult> for bool {
    fn from(match_result: MatchResult) -> bool {
        match_result.result
    }
}

// Pretty print a MatcherType
impl std::fmt::Display for MatcherType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MatcherType::Glob => write!(f, "glob"),
            MatcherType::Regex => write!(f, "regex"),
        }
    }
}
