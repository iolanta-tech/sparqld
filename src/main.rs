use std::path::PathBuf;

use clap::{Parser, ValueHint};

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
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let Cli {
        directory,
        host,
        port,
        no_watch,
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
        sparqld::ServeOptions { watch: !no_watch },
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
            "fixtures/data",
        ])
        .unwrap();

        assert_eq!(cli.directory, PathBuf::from("fixtures/data"));
        assert_eq!(cli.host, "localhost");
        assert_eq!(cli.port, 8080);
        assert!(cli.no_watch);
    }

    #[test]
    fn provides_help() {
        let error = Cli::try_parse_from(["sparqld", "--help"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::DisplayHelp);
        let help = error.to_string();
        assert!(help.contains("--host <HOST>"));
        assert!(help.contains("--port <PORT>"));
        assert!(help.contains("--no-watch"));
    }

    #[test]
    fn rejects_an_invalid_port() {
        let error =
            Cli::try_parse_from(["sparqld", "--port", "70000", "fixtures/data"]).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::ValueValidation);
    }
}
