use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueHint};

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
#[command(
    version,
    about = "Expose a directory via SPARQL.",
    subcommand_negates_reqs = true,
    args_conflicts_with_subcommands = true
)]
struct Cli {
    /// Directory to serve
    #[arg(value_hint = ValueHint::DirPath, required = true)]
    directory: Option<PathBuf>,

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

    /// Stop queries that run longer than this many seconds
    #[arg(
        long = "query-timeout",
        value_name = "SECONDS",
        allow_hyphen_values = true,
        value_parser = sparqld::parse_query_timeout
    )]
    query_timeout: Option<Duration>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Expose a directory to MCP clients over standard input and output
    Mcp(McpCli),
}

#[derive(Args, Debug)]
#[command(about = "Expose a directory to MCP clients over standard input and output")]
struct McpCli {
    /// Directory to watch and expose
    #[arg(value_hint = ValueHint::DirPath)]
    directory: PathBuf,

    /// Include or exclude source paths with a glob; prefix exclusions with !
    #[arg(
        long = "pattern",
        value_name = "GLOB",
        value_parser = sparqld::patterns::validate_pattern
    )]
    patterns: Vec<String>,

    /// Stop queries that run longer than this many seconds
    #[arg(
        long = "query-timeout",
        value_name = "SECONDS",
        allow_hyphen_values = true,
        value_parser = sparqld::parse_query_timeout
    )]
    query_timeout: Option<Duration>,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp_secs()
        .format_level(false)
        .init();

    let Cli {
        directory,
        host,
        port,
        no_watch,
        patterns,
        query_timeout,
        command,
    } = Cli::parse();
    match command {
        Some(Command::Mcp(McpCli {
            directory,
            patterns,
            query_timeout,
        })) => sparqld::serve_mcp(directory, patterns, query_timeout),
        None => sparqld::serve_at_with_options(
            directory.expect("Clap requires a directory when no subcommand is given"),
            &host,
            port,
            sparqld::ServeOptions {
                watch: !no_watch,
                patterns,
                query_timeout,
            },
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn parses_defaults() {
        let cli = Cli::try_parse_from(["sparqld", "fixtures/data"]).unwrap();

        assert_eq!(cli.directory, Some(PathBuf::from("fixtures/data")));
        assert_eq!(cli.host, "127.0.0.1");
        assert_eq!(cli.port, 7737);
        assert!(!cli.no_watch);
        assert!(cli.patterns.is_empty());
        assert!(cli.query_timeout.is_none());
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

        assert_eq!(cli.directory, Some(PathBuf::from("fixtures/data")));
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
        assert!(help.contains("--query-timeout <SECONDS>"));
        assert!(help.contains("mcp"));
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

    #[test]
    fn parses_mcp_options() {
        let Cli {
            directory, command, ..
        } = Cli::try_parse_from(["sparqld", "mcp", "--pattern", "**/*.ttl", "fixtures/data"])
            .unwrap();

        assert!(directory.is_none());
        assert!(matches!(
            command,
            Some(Command::Mcp(McpCli {
                directory,
                patterns,
                query_timeout: None,
            })) if directory.as_path() == std::path::Path::new("fixtures/data") && patterns == ["**/*.ttl"]
        ));
    }

    #[test]
    fn parses_a_query_timeout() {
        let cli =
            Cli::try_parse_from(["sparqld", "--query-timeout", "20", "fixtures/data"]).unwrap();
        assert_eq!(cli.query_timeout, Some(Duration::from_secs(20)));

        let Cli { command, .. } =
            Cli::try_parse_from(["sparqld", "mcp", "--query-timeout", "20", "fixtures/data"])
                .unwrap();
        assert!(matches!(
            command,
            Some(Command::Mcp(McpCli {
                query_timeout: Some(timeout),
                ..
            })) if timeout == Duration::from_secs(20)
        ));
    }

    #[test]
    fn rejects_invalid_query_timeouts() {
        for value in ["0", "-1", "abc", "1.5", "20s", &u64::MAX.to_string()] {
            let error = Cli::try_parse_from(["sparqld", "--query-timeout", value, "fixtures/data"])
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::ValueValidation, "{value}");

            let error =
                Cli::try_parse_from(["sparqld", "mcp", "--query-timeout", value, "fixtures/data"])
                    .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::ValueValidation, "mcp {value}");
        }
    }

    #[test]
    fn mcp_help_lists_query_timeout() {
        let error = Cli::try_parse_from(["sparqld", "mcp", "--help"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
        assert!(error.to_string().contains("--query-timeout <SECONDS>"));
    }

    #[test]
    fn rejects_no_watch_for_mcp() {
        let error =
            Cli::try_parse_from(["sparqld", "mcp", "--no-watch", "fixtures/data"]).unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn rejects_http_options_for_mcp() {
        let error = Cli::try_parse_from(["sparqld", "--host", "localhost", "mcp", "fixtures/data"])
            .unwrap_err();

        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }
}
