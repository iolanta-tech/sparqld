//! Matching rules for selecting source files by served-directory-relative path.

use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

#[derive(Clone, Debug)]
pub(crate) struct SourcePatterns {
    includes: GlobSet,
    excludes: GlobSet,
    has_includes: bool,
}

impl SourcePatterns {
    #[cfg(test)]
    pub(crate) fn all() -> Self {
        Self {
            includes: GlobSetBuilder::new()
                .build()
                .expect("an empty glob set is valid"),
            excludes: GlobSetBuilder::new()
                .build()
                .expect("an empty glob set is valid"),
            has_includes: false,
        }
    }

    pub(crate) fn compile(patterns: &[String]) -> Result<Self, String> {
        let (excludes, includes): (Vec<_>, Vec<_>) = patterns
            .iter()
            .map(|raw_pattern| {
                let (pattern, excluded) = split_pattern(raw_pattern)?;
                glob_variants(pattern).map(|globs| (excluded, globs))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .partition(|(excluded, _)| *excluded);
        let has_includes = !includes.is_empty();

        Ok(Self {
            includes: build_set(includes.into_iter().flat_map(|(_, globs)| globs))?,
            excludes: build_set(excludes.into_iter().flat_map(|(_, globs)| globs))?,
            has_includes,
        })
    }

    pub(crate) fn matches(&self, path: &Path) -> bool {
        (!self.has_includes || self.includes.is_match(path)) && !self.excludes.is_match(path)
    }
}

fn glob_variants(pattern: &str) -> Result<Vec<Glob>, String> {
    std::iter::once(pattern.to_owned())
        .chain((!pattern.contains(['*', '?', '[', '{'])).then(|| format!("{pattern}/**")))
        .map(|pattern| Glob::new(&pattern).map_err(|error| error.to_string()))
        .collect()
}

fn build_set(globs: impl IntoIterator<Item = Glob>) -> Result<GlobSet, String> {
    globs
        .into_iter()
        .fold(GlobSetBuilder::new(), |mut builder, glob| {
            builder.add(glob);
            builder
        })
        .build()
        .map_err(|error| error.to_string())
}

/// Validates one command-line source pattern.
pub fn validate_pattern(pattern: &str) -> Result<String, String> {
    let (glob, _) = split_pattern(pattern)?;
    glob_variants(glob)?;
    Ok(pattern.to_owned())
}

fn split_pattern(pattern: &str) -> Result<(&str, bool), String> {
    if pattern.is_empty() {
        return Err("pattern must not be empty".to_owned());
    }
    let (pattern, excluded) = pattern
        .strip_prefix('!')
        .map_or((pattern, false), |pattern| (pattern, true));
    if pattern.is_empty() {
        return Err("exclusion pattern must follow !".to_owned());
    }
    Ok((pattern, excluded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_all_paths() {
        assert!(SourcePatterns::all().matches(Path::new("nested/source.ttl")));
    }

    #[test]
    fn applies_includes_before_exclusions() {
        let patterns =
            SourcePatterns::compile(&["**/*.ttl".to_owned(), "!archive/**".to_owned()]).unwrap();

        assert!(patterns.matches(Path::new("nested/source.ttl")));
        assert!(!patterns.matches(Path::new("archive/source.ttl")));
        assert!(!patterns.matches(Path::new("nested/source.jsonld")));
    }

    #[test]
    fn exclusions_without_includes_start_from_all_paths() {
        let patterns = SourcePatterns::compile(&["!archive/**".to_owned()]).unwrap();

        assert!(patterns.matches(Path::new("source.ttl")));
        assert!(!patterns.matches(Path::new("archive/source.ttl")));
    }

    #[test]
    fn plain_directory_exclusions_match_descendants() {
        let patterns = SourcePatterns::compile(&["!node_modules".to_owned()]).unwrap();

        assert!(patterns.matches(Path::new("source.ttl")));
        assert!(!patterns.matches(Path::new("node_modules/package/index.jsonld")));
    }

    #[test]
    fn rejects_empty_patterns() {
        assert_eq!(
            validate_pattern(""),
            Err("pattern must not be empty".to_owned())
        );
        assert_eq!(
            validate_pattern("!"),
            Err("exclusion pattern must follow !".to_owned())
        );
    }
}
