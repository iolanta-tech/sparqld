//! Parse command-line options and start the HTTP or stdio MCP service.

mod config;

use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueHint};
use config::{Config, HttpConfig};
use sparqld::HttpOptions;

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
//   served directory option and native `--pattern` behavior.
// acceptance:
//   - Unit-test repeated flags, PATH and explicit-path resolution, descriptor
//     validation, and every declared startup rejection.
// blocked-by:
//   - riddles-cli
/// Shared command-line settings and the selected service.
#[derive(Debug, Parser)]
#[command(version, about = "Expose a directory via SPARQL.")]
struct Cli {
    /// Directory to serve (overrides the configuration file)
    #[arg(short, long, value_name = "PATH", value_hint = ValueHint::DirPath)]
    directory: Option<PathBuf>,

    /// Read startup settings from this file (defaults to ./sparqld.toml)
    #[arg(short = 'c', long = "config", value_name = "PATH", value_hint = ValueHint::FilePath)]
    config_path: Option<PathBuf>,

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

    /// Service to start.
    #[command(subcommand)]
    command: Command,
}

/// Alternate service interfaces selected from the command line.
#[derive(Debug, Subcommand)]
enum Command {
    /// Expose a directory as a SPARQL endpoint over HTTP
    #[command(after_help = "Use sparqld [OPTIONS] http to set shared options. See sparqld --help.")]
    Http(HttpCli),
    /// Expose a directory to MCP clients over standard input and output
    #[command(after_help = "Use sparqld [OPTIONS] mcp to set shared options. See sparqld --help.")]
    Mcp,
}

/// Options that apply to the HTTP service.
#[derive(Args, Debug)]
struct HttpCli {
    /// Host address on which to serve (defaults to 127.0.0.1)
    #[arg(long)]
    host: Option<String>,

    /// Port on which to serve (defaults to 7737)
    #[arg(long)]
    port: Option<u16>,

    /// Watch the directory for changes
    #[arg(long, conflicts_with = "no_watch")]
    watch: bool,

    /// Load the directory once instead of watching it for changes
    #[arg(long)]
    no_watch: bool,
}

impl HttpCli {
    /// Convert the mutually exclusive flags to an optional watch override.
    fn watch_override(&self) -> Option<bool> {
        self.watch
            .then_some(true)
            .or(self.no_watch.then_some(false))
    }

    /// Resolve HTTP settings in order of CLI, file, and runtime defaults.
    fn resolve(self, config: HttpConfig) -> HttpOptions {
        let defaults = HttpOptions::default();
        let watch = self
            .watch_override()
            .or(config.watch)
            .unwrap_or(defaults.watch);
        HttpOptions {
            host: self.host.or(config.host).unwrap_or(defaults.host),
            port: self.port.or(config.port).unwrap_or(defaults.port),
            watch,
        }
    }
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
        config_path,
        patterns,
        query_timeout,
        command,
    } = Cli::parse();
    let config = Config::load(Path::new("."), config_path.as_deref())?;
    let (options, http) = config.sources(directory, patterns, query_timeout)?;
    match command {
        Command::Http(cli) => sparqld::serve_http(options, cli.resolve(http)),
        Command::Mcp => sparqld::serve_mcp(options),
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
        let Command::Http(http) = cli.command else {
            panic!("expected http");
        };
        let http = http.resolve(HttpConfig::default());
        assert_eq!(http.host, "127.0.0.1");
        assert_eq!(http.port, 7737);
        assert!(http.watch);
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
        let Command::Http(http) = cli.command else {
            panic!("expected http");
        };
        let http = http.resolve(HttpConfig {
            watch: Some(true),
            ..HttpConfig::default()
        });
        assert_eq!(http.host, "localhost");
        assert_eq!(http.port, 8080);
        assert!(!http.watch);
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
    fn enables_watching_over_a_disabled_configuration() {
        for (arguments, expected_watch) in [
            (&["sparqld", "http"][..], false),
            (&["sparqld", "http", "--watch"][..], true),
        ] {
            let cli = Cli::try_parse_from(arguments).unwrap();
            let Command::Http(http) = cli.command else {
                panic!("expected http");
            };
            let http = http.resolve(HttpConfig {
                watch: Some(false),
                ..HttpConfig::default()
            });
            assert_eq!(http.watch, expected_watch);
        }
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
