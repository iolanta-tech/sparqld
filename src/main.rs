use std::path::PathBuf;

use clap::{Parser, ValueHint};

// @todo(extractor-registration): Register self-describing extractor executables.
// description: >
//   Add a repeatable `--extractor EXECUTABLE` option. It accepts a PATH-resolved
//   executable name or an explicit executable path. Keep v1 registration to
//   descriptor-driven executable discovery, without manual globs or arguments.
//
//   At startup, invoke `EXECUTABLE describe`, parse its JSON-LD descriptor, and
//   validate its supported protocol version, nonempty stable identity, and at
//   least one positive glob. Reject unavailable executables, failed describe
//   invocations, invalid descriptors, unknown protocol versions, and duplicate
//   extractor identities with actionable startup errors. Preserve the existing
//   positional served directory and native `--pattern` behavior.
// acceptance:
//   - Unit-test repeated flags, PATH and explicit-path resolution, descriptor
//     validation, and every declared startup rejection.
// blocked-by:
//   - riddles-cli
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
