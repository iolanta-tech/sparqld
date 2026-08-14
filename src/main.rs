use std::path::PathBuf;

use clap::{Parser, ValueHint};

// @todo(sparqld-configuration): Add project configuration in sparqld.toml.
// description: >
//   Add a sparqld.toml file at the served root. Its v1 extractor schema is:
//
//   [[extractors]]
//   id = "unique-id"
//   command = "executable"
//   arguments = ["optional", "arguments"]
//   patterns = ["**/*.py", "!generated/**"]
//
//   `id`, `command`, and a nonempty `patterns` are required; `arguments`
//   defaults to an empty array. A pattern beginning with `!` excludes matches;
//   a source matches when it matches at least one positive pattern and no
//   exclusion pattern. Reject duplicate IDs, empty exclusion patterns, invalid
//   globs, unknown fields, and empty command values with actionable
//   configuration errors. Implement the parsed Config and ExtractorConfig
//   types in a new src/config.rs module, using the `toml` and `globset` crates.
//   Make the positional directory optional: when it is omitted,
//   require sparqld.toml in the current directory and serve that directory;
//   when it is supplied, preserve current directory-serving behavior and load
//   a sparqld.toml only from that directory when it exists.
// acceptance:
//   - Unit-test valid parsing, defaults, every validation failure, and glob
//     matching against relative paths.
//   - Test CLI behavior with and without a configuration file and directory.
//   - Keep serving a directory without sparqld.toml backward compatible.
#[derive(Debug, Parser)]
#[command(version, about = "Expose a directory via SPARQL.")]
struct Cli {
    /// Directory to serve
    #[arg(value_hint = ValueHint::DirPath)]
    directory: PathBuf,

    /// Host address on which to serve
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port on which to serve
    #[arg(long, default_value_t = 7737)]
    port: u16,

    /// Load the directory once instead of watching it for changes
    #[arg(long)]
    no_watch: bool,

    /// Include or exclude source paths with a glob; prefix exclusions with !
    #[arg(
        long = "pattern",
        value_name = "GLOB",
        value_parser = sparqld::patterns::validate_pattern
    )]
    patterns: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Cli {
        directory,
        host,
        port,
        no_watch,
        patterns,
    } = Cli::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp_secs()
        .format_level(false)
        .init();

    sparqld::serve_at_with_options(
        directory,
        &host,
        port,
        sparqld::ServeOptions {
            watch: !no_watch,
            patterns,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn parses_defaults() {
        let cli = Cli::try_parse_from(["sparqld", "fixtures/data"]).unwrap();

        assert_eq!(cli.directory, PathBuf::from("fixtures/data"));
        assert_eq!(cli.host, "127.0.0.1");
        assert_eq!(cli.port, 7737);
        assert!(!cli.no_watch);
        assert!(cli.patterns.is_empty());
    }

    #[test]
    fn parses_options() {
        let cli = Cli::try_parse_from([
            "sparqld",
            "--host",
            "localhost",
            "--port",
            "8080",
            "--no-watch",
            "--pattern",
            "**/*.ttl",
            "--pattern",
            "!archive/**",
            "fixtures/data",
        ])
        .unwrap();

        assert_eq!(cli.directory, PathBuf::from("fixtures/data"));
        assert_eq!(cli.host, "localhost");
        assert_eq!(cli.port, 8080);
        assert!(cli.no_watch);
        assert_eq!(cli.patterns, ["**/*.ttl", "!archive/**"]);
    }

    #[test]
    fn provides_help() {
        let error = Cli::try_parse_from(["sparqld", "--help"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
        let help = error.to_string();
        assert!(help.contains("--host <HOST>"));
        assert!(help.contains("--port <PORT>"));
        assert!(help.contains("--no-watch"));
        assert!(help.contains("--pattern <GLOB>"));
    }

    #[test]
    fn rejects_an_invalid_port() {
        let error =
            Cli::try_parse_from(["sparqld", "--port", "70000", "fixtures/data"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn rejects_an_invalid_pattern() {
        let error =
            Cli::try_parse_from(["sparqld", "--pattern", "!", "fixtures/data"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ValueValidation);
        assert!(
            error
                .to_string()
                .contains("exclusion pattern must follow !")
        );
    }

    #[test]
    fn rejects_an_invalid_glob() {
        let error =
            Cli::try_parse_from(["sparqld", "--pattern", "[", "fixtures/data"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ValueValidation);
    }
}
