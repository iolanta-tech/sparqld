//! Read startup settings from a selected file or sparqld.toml in the working directory.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use sparqld::SparqldOptions;

/// Optional startup settings for the HTTP and MCP interfaces.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct Config {
    /// Directory to serve.
    directory: Option<PathBuf>,
    /// Ordered source globs.
    patterns: Vec<String>,
    /// Maximum query duration in seconds.
    query_timeout: Option<i64>,
    /// Settings used only by the HTTP interface.
    http: HttpConfig,
    /// File to name in validation errors.
    #[serde(skip)]
    path: Option<PathBuf>,
}

/// Optional HTTP settings.
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct HttpConfig {
    /// Address on which to serve.
    pub(crate) host: Option<String>,
    /// Port on which to serve.
    pub(crate) port: Option<u16>,
    /// Whether to watch the served directory.
    pub(crate) watch: Option<bool>,
}

impl Config {
    /// Load the explicit file or discover sparqld.toml in the working directory.
    pub(crate) fn load(working_directory: &Path, explicit_path: Option<&Path>) -> io::Result<Self> {
        let path = working_directory.join(explicit_path.unwrap_or(Path::new("sparqld.toml")));
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) if explicit_path.is_none() && error.kind() == io::ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => {
                return Err(io::Error::new(
                    error.kind(),
                    format!("cannot read {}: {error}", path.display()),
                ));
            }
        };
        let mut config: Self = toml::from_str(&contents).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid configuration in {}: {error}", path.display()),
            )
        })?;
        if config
            .directory
            .as_ref()
            .is_some_and(|directory| directory.as_os_str().is_empty())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid configuration in {}: directory must not be empty",
                    path.display()
                ),
            ));
        }
        if let (Some(directory), Some(parent)) = (&mut config.directory, path.parent()) {
            *directory = parent.join(directory.as_path());
        }
        config.path = Some(path);
        Ok(config)
    }

    /// Resolve shared settings for either interface and return its sibling HTTP settings.
    pub(crate) fn sources(
        self,
        directory: Option<PathBuf>,
        patterns: Vec<String>,
        query_timeout: Option<Duration>,
    ) -> io::Result<(SparqldOptions, HttpConfig)> {
        let invalid_setting = |field: &str, error: String| {
            let location = self
                .path
                .as_ref()
                .map_or_else(String::new, |path| format!(" in {}", path.display()));
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid configuration{location}: {field}: {error}"),
            )
        };
        // Validate file settings before applying CLI overrides.
        for pattern in &self.patterns {
            sparqld::patterns::validate_pattern(pattern)
                .map_err(|error| invalid_setting("patterns", error))?;
        }
        let configured_timeout = self
            .query_timeout
            .map(|seconds| sparqld::parse_query_timeout(&seconds.to_string()))
            .transpose()
            .map_err(|error| invalid_setting("query-timeout", error))?;
        let directory = directory.or(self.directory).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "use --directory or set directory in the configuration file",
            )
        })?;
        Ok((
            SparqldOptions {
                directory,
                patterns: if patterns.is_empty() {
                    self.patterns
                } else {
                    patterns
                },
                query_timeout: query_timeout.or(configured_timeout),
            },
            self.http,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_file_source_settings_without_cli_overrides() {
        let config = toml::from_str::<Config>(
            r#"
            directory = "configured"
            patterns = ["*.ttl", "!private.ttl"]
            query-timeout = 5
            "#,
        )
        .unwrap();
        let (source, _) = config.sources(None, Vec::new(), None).unwrap();
        assert_eq!(source.directory, Path::new("configured"));
        assert_eq!(source.patterns, ["*.ttl", "!private.ttl"]);
        assert_eq!(source.query_timeout, Some(Duration::from_secs(5)));
    }

    #[test]
    fn uses_cli_source_overrides() {
        let config = toml::from_str::<Config>(
            r#"
            directory = "configured"
            patterns = ["*.ttl", "!private.ttl"]
            query-timeout = 5
            "#,
        )
        .unwrap();
        let (source, _) = config
            .sources(
                Some(PathBuf::from("explicit")),
                vec!["*.trig".into()],
                Some(Duration::from_secs(10)),
            )
            .unwrap();
        assert_eq!(source.directory, Path::new("explicit"));
        assert_eq!(source.patterns, ["*.trig"]);
        assert_eq!(source.query_timeout, Some(Duration::from_secs(10)));
    }

    #[test]
    fn requires_a_directory_from_one_source() {
        let cwd = tempfile::tempdir().unwrap();
        let config = Config::load(cwd.path(), None).unwrap();
        let error = config.sources(None, Vec::new(), None).err().unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(
            error
                .to_string()
                .contains("set directory in the configuration file")
        );
    }

    #[test]
    fn reports_missing_or_unreadable_explicit_files() {
        let cwd = tempfile::tempdir().unwrap();
        fs::write(cwd.path().join("sparqld.toml"), "directory = \"data\"").unwrap();
        let explicit_path = Path::new("custom.toml");
        let error = Config::load(cwd.path(), Some(explicit_path)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(error.to_string().contains("custom.toml"));

        fs::create_dir(cwd.path().join(explicit_path)).unwrap();
        let error = Config::load(cwd.path(), Some(explicit_path)).unwrap_err();
        assert!(error.to_string().contains("custom.toml"));
    }

    #[test]
    fn accepts_an_absolute_configuration_path_and_directory() {
        let cwd = tempfile::tempdir().unwrap();
        let settings = tempfile::tempdir().unwrap();
        let path = settings.path().join("custom.toml");
        let directory = cwd.path().join("data");
        fs::write(
            &path,
            toml::to_string(&std::collections::BTreeMap::from([(
                "directory",
                &directory,
            )]))
            .unwrap(),
        )
        .unwrap();
        let (sources, _) = Config::load(cwd.path(), Some(&path))
            .unwrap()
            .sources(None, Vec::new(), None)
            .unwrap();
        assert_eq!(sources.directory, directory);
    }

    #[test]
    fn accepts_the_configuration_directory_when_explicitly_selected() {
        let cwd = tempfile::tempdir().unwrap();
        let settings = tempfile::tempdir().unwrap();
        let path = settings.path().join("custom.toml");
        fs::write(&path, r#"directory = ".""#).unwrap();
        let (sources, _) = Config::load(cwd.path(), Some(&path))
            .unwrap()
            .sources(None, Vec::new(), None)
            .unwrap();
        assert_eq!(sources.directory, settings.path());
    }

    #[test]
    fn reports_invalid_settings_with_the_file_path() {
        let cwd = tempfile::tempdir().unwrap();
        let path = cwd.path().join("sparqld.toml");
        for (contents, diagnostic) in [
            ("directory = [", "invalid configuration"),
            (r#"directory = """#, "directory must not be empty"),
            (r#"patterns = ["!"]"#, "exclusion pattern must follow !"),
            (
                "query-timeout = 0",
                "query timeout must be at least 1 second",
            ),
            (r#"query-timeout = "1""#, "invalid type"),
            ("[http]\nport = 70000", "invalid value"),
            ("[http]\nwatched = false", "unknown field"),
        ] {
            fs::write(&path, contents).unwrap();
            let error = Config::load(cwd.path(), None)
                .and_then(|config| {
                    config.sources(
                        Some(PathBuf::from("explicit")),
                        vec!["*.ttl".into()],
                        Some(Duration::from_secs(10)),
                    )
                })
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert!(error.to_string().contains(path.to_str().unwrap()));
            assert!(error.to_string().contains(diagnostic), "{error}");
        }
    }
}
