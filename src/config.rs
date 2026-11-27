#[cfg(test)]
mod tests {
    use super::*;

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
    fn reports_invalid_settings_with_the_file_path() {
        let cwd = tempfile::tempdir().unwrap();
        let path = cwd.path().join("sparqld.toml");
        for (contents, diagnostic) in [
            ("directory = [", "invalid configuration"),
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
            let error = Config::load(cwd.path(), None).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert!(error.to_string().contains(path.to_str().unwrap()));
            assert!(error.to_string().contains(diagnostic), "{error}");
        }
    }
}
