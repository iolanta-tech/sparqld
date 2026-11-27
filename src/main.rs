//! Parse command-line options and start the HTTP or stdio MCP service.

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
/// Command-line settings for the HTTP service and its subcommands.
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

    /// Optional stdio MCP service command.
    #[command(subcommand)]
    command: Option<Command>,
}

/// Alternate service interfaces selected from the command line.
#[derive(Debug, Subcommand)]
enum Command {
    /// Expose a directory to MCP clients over standard input and output
    Mcp(McpCli),
}

/// Directory selection and query limits for the stdio MCP service.
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

/// Initialize logging and run the service selected by the command-line arguments.
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
        let cli = Cli::try_parse_from(["sparqld", "http"]).unwrap();

        assert!(cli.directory.is_none());
        assert!(matches!(
            cli.command,
            Command::Http(HttpCli {
                host: None,
                port: None,
                watch: false,
                no_watch: false
            })
        ));
        assert!(cli.patterns.is_empty());
        assert!(cli.query_timeout.is_none());
    }

    #[test]
    fn parses_options() {
        let cli = Cli::try_parse_from([
            "sparqld",
            "--pattern",
            "**/*.ttl",
            "--pattern",
            "!archive/**",
            "--directory",
            "fixtures/data",
            "http",
            "--host",
            "localhost",
            "--port",
            "8080",
            "--no-watch",
        ])
        .unwrap();

        assert_eq!(cli.directory, Some(PathBuf::from("fixtures/data")));
        assert!(matches!(
            cli.command,
            Command::Http(HttpCli { host: Some(host), port: Some(8080), no_watch: true, watch: false })
                if host == "localhost"
        ));
        assert_eq!(cli.patterns, ["**/*.ttl", "!archive/**"]);
    }

    #[test]
    fn provides_help() {
        let error = Cli::try_parse_from(["sparqld", "--help"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
        let help = error.to_string();
        assert!(help.contains("-c, --config <PATH>"));
        assert!(help.contains("-d, --directory <PATH>"));
        assert!(help.contains("--pattern <GLOB>"));
        assert!(help.contains("--query-timeout <SECONDS>"));
        assert!(help.contains("http"));
        assert!(help.contains("mcp"));
        assert!(!help.contains("--host"));
    }

    #[test]
    fn rejects_an_invalid_port() {
        let error = Cli::try_parse_from(["sparqld", "http", "--port", "70000"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn rejects_an_invalid_pattern() {
        let error = Cli::try_parse_from(["sparqld", "--pattern", "!"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ValueValidation);
        assert!(
            error
                .to_string()
                .contains("exclusion pattern must follow !")
        );
    }

    #[test]
    fn rejects_an_invalid_glob() {
        let error = Cli::try_parse_from(["sparqld", "--pattern", "["]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn parses_shared_options_before_mcp() {
        let cli = Cli::try_parse_from([
            "sparqld",
            "--config",
            "custom.toml",
            "--pattern",
            "**/*.ttl",
            "--query-timeout",
            "20",
            "-d",
            "fixtures/data",
            "mcp",
        ])
        .unwrap();
        assert_eq!(cli.directory.as_deref(), Some(Path::new("fixtures/data")));
        assert_eq!(cli.config_path.as_deref(), Some(Path::new("custom.toml")));
        assert_eq!(cli.patterns, ["**/*.ttl"]);
        assert_eq!(cli.query_timeout, Some(Duration::from_secs(20)));
        assert!(matches!(cli.command, Command::Mcp));
    }

    #[test]
    fn parses_a_query_timeout() {
        let cli = Cli::try_parse_from(["sparqld", "--query-timeout", "20", "http"]).unwrap();
        assert_eq!(cli.query_timeout, Some(Duration::from_secs(20)));
    }

    #[test]
    fn rejects_invalid_query_timeouts() {
        for value in ["0", "-1", "abc", "1.5", "20s", &u64::MAX.to_string()] {
            let error = Cli::try_parse_from(["sparqld", "--query-timeout", value]).unwrap_err();
            assert_eq!(error.kind(), ErrorKind::ValueValidation, "{value}");
        }
    }

    #[test]
    fn mcp_help_shows_where_to_place_shared_options() {
        let error = Cli::try_parse_from(["sparqld", "mcp", "--help"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
        assert!(error.to_string().contains("sparqld [OPTIONS] mcp"));
    }

    #[test]
    fn http_help_lists_http_options() {
        let error = Cli::try_parse_from(["sparqld", "http", "--help"]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
        let help = error.to_string();
        assert!(help.contains("sparqld [OPTIONS] http"));
        assert!(help.contains("--host <HOST>"));
        assert!(help.contains("--port <PORT>"));
        assert!(help.contains("--watch"));
        assert!(help.contains("--no-watch"));
    }

    #[test]
    fn requires_a_service_subcommand() {
        for arguments in [&["sparqld"][..], &["sparqld", "-d", "fixtures/data"]] {
            let error = Cli::try_parse_from(arguments).unwrap_err();
            assert!(matches!(
                error.kind(),
                ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            ));
            assert_eq!(error.exit_code(), 2);
        }
    }

    #[test]
    fn rejects_conflicting_http_watch_flags() {
        let error = Cli::try_parse_from(["sparqld", "http", "--watch", "--no-watch"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn rejects_http_options_outside_http() {
        for arguments in [
            &["--host", "localhost"][..],
            &["--port", "7737"],
            &["--watch"],
            &["--no-watch"],
        ] {
            for prefix in [&["sparqld", "mcp"][..], &["sparqld"]] {
                let args = prefix.iter().chain(arguments).copied();
                let error = Cli::try_parse_from(args).unwrap_err();
                assert_eq!(error.kind(), ErrorKind::UnknownArgument);
            }
        }
    }

    #[test]
    fn rejects_shared_options_after_a_subcommand() {
        for option in [
            "--directory",
            "-d",
            "--config",
            "-c",
            "--pattern",
            "--query-timeout",
        ] {
            for command in ["http", "mcp"] {
                let error = Cli::try_parse_from(["sparqld", command, option, "10"]).unwrap_err();
                assert_eq!(error.kind(), ErrorKind::UnknownArgument);
            }
        }
    }

    #[test]
    fn accepts_option_values_named_like_subcommands() {
        let cli = Cli::try_parse_from(["sparqld", "--config", "mcp", "--pattern", "help", "mcp"])
            .unwrap();

        assert_eq!(cli.config_path.as_deref(), Some(Path::new("mcp")));
        assert_eq!(cli.patterns, ["help"]);
        assert!(matches!(cli.command, Command::Mcp));
    }

    #[test]
    fn parses_a_directory_named_mcp() {
        let cli = Cli::try_parse_from(["sparqld", "--directory", "mcp", "http"]).unwrap();

        assert!(matches!(cli.command, Command::Http(_)));
        assert_eq!(cli.directory.as_deref(), Some(Path::new("mcp")));
    }

    #[test]
    fn rejects_positional_directories() {
        for arguments in [
            &["sparqld", "fixtures/data"][..],
            &["sparqld", "http", "fixtures/data"],
            &["sparqld", "mcp", "fixtures/data"],
        ] {
            let error = Cli::try_parse_from(arguments).unwrap_err();
            assert!(matches!(
                error.kind(),
                ErrorKind::InvalidSubcommand | ErrorKind::UnknownArgument
            ));
        }
    }
}
