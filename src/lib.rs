//! Serve directory-backed RDF datasets through HTTP and stdio MCP.

mod loader;
mod mcp;
pub mod patterns;
mod query;
mod server;
mod watcher;

pub use query::parse_query_timeout;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::io;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use oxigraph::store::{StorageError, Store};

/// Default loopback address for the HTTP endpoint.
const DEFAULT_HOST: &str = "127.0.0.1";
/// Default port for the HTTP endpoint.
const DEFAULT_PORT: u16 = 7737;

/// A service result that can carry errors across worker threads.
type SparqldResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Source tracking and triple counts from the initial directory load.
struct DatasetLoad {
    /// Relative paths of successfully loaded sources.
    loaded_sources: BTreeSet<PathBuf>,
    /// Latest loading error for each failed source path.
    load_errors: BTreeMap<PathBuf, String>,
    /// Number of files excluded by format, patterns, or missing front matter.
    ignored_files: usize,
    /// Number of quads in the dataset, including catalog entries.
    triples: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Counts used to report the current dataset at startup.
pub(crate) struct DatasetStats {
    /// Number of successfully loaded source files.
    loaded_files: usize,
    /// Number of source files with loading errors.
    failed_files: usize,
    /// Number of files skipped during source discovery and loading.
    ignored_files: usize,
    /// Number of quads across source graphs and the catalog.
    triples: usize,
}

/// Dataset and query settings shared by HTTP and MCP.
#[derive(Clone, Debug)]
pub struct SparqldOptions {
    /// Directory containing the dataset sources.
    pub directory: PathBuf,
    /// Glob patterns selecting source files relative to the served directory.
    pub patterns: Vec<String>,
    /// Stop a query that runs longer than this budget. `None` is unlimited.
    pub query_timeout: Option<Duration>,
}

/// Settings specific to the HTTP service.
#[derive(Clone, Debug)]
pub struct HttpOptions {
    /// Address on which to listen.
    pub host: String,
    /// Port on which to listen.
    pub port: u16,
    /// Reload the dataset when the directory changes.
    pub watch: bool,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.into(),
            port: DEFAULT_PORT,
            watch: true,
        }
    }
}

/// Serves the RDF dataset through an HTTP SPARQL endpoint.
pub fn serve_http(options: SparqldOptions, http: HttpOptions) -> SparqldResult<()> {
    let requested_directory = options.directory;
    let directory = requested_directory.canonicalize()?;
    let source_patterns =
        patterns::SourcePatterns::compile(&options.patterns).map_err(io::Error::other)?;
    let addresses = (http.host.as_str(), http.port)
        .to_socket_addrs()?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "host resolved to no addresses",
        )
        .into());
    }

    let dataset = new_dataset()?;
    let startup = server::StartupGate::new();
    let policy = query::QueryPolicy::new(options.query_timeout);
    let listening_server = server::start(dataset.clone(), startup.clone(), policy, addresses)?;
    start_initialization(
        directory,
        requested_directory,
        http,
        dataset,
        source_patterns,
        startup,
    )?;
    listening_server.join()?;
    Ok(())
}

/// Serves the RDF dataset through a local stdio MCP server.
pub fn serve_mcp(options: SparqldOptions) -> SparqldResult<()> {
    let directory = options.directory.canonicalize()?;
    let source_patterns =
        patterns::SourcePatterns::compile(&options.patterns).map_err(io::Error::other)?;
    let dataset = new_dataset()?;
    mcp::serve(
        dataset,
        directory,
        source_patterns,
        server::StartupGate::new(),
        query::QueryPolicy::new(options.query_timeout),
    )
}

/// Load the directory in a worker, release waiting requests, and keep its watcher alive.
fn start_initialization(
    directory: PathBuf,
    requested_directory: PathBuf,
    http: HttpOptions,
    dataset: Store,
    source_patterns: patterns::SourcePatterns,
    startup: server::StartupGate,
) -> SparqldResult<()> {
    thread::Builder::new()
        .name("sparqld-initializer".into())
        .spawn(move || {
            let initialized = if http.watch {
                watcher::DirectoryWatcher::start_with_patterns(directory, dataset, source_patterns)
                    .map(|(watcher, stats)| (Some(watcher), stats))
            } else {
                load_dataset(&directory, &dataset, &source_patterns)
                    .map(|load| (None, load.stats()))
            };

            match initialized {
                Ok((watcher, stats)) => {
                    startup.mark_ready();
                    log_serving(&requested_directory, &http.host, http.port);
                    log::info!("{}", dataset_summary(stats));
                    if let Some(_watcher) = watcher {
                        loop {
                            thread::park();
                        }
                    }
                }
                Err(error) => {
                    log::error!("Could not initialize dataset: {error}");
                    startup.mark_failed();
                    thread::yield_now();
                    std::process::exit(1);
                }
            }
        })?;
    Ok(())
}

impl DatasetLoad {
    /// Summarize the tracked sources and dataset size for startup logging.
    fn stats(&self) -> DatasetStats {
        DatasetStats {
            loaded_files: self.loaded_sources.len(),
            failed_files: self.load_errors.len(),
            ignored_files: self.ignored_files,
            triples: self.triples,
        }
    }
}

/// Load selected directory sources and count the resulting dataset quads.
fn load_dataset(
    directory: &Path,
    dataset: &Store,
    source_patterns: &patterns::SourcePatterns,
) -> SparqldResult<DatasetLoad> {
    let load = loader::load_directory_with_patterns(directory, dataset, source_patterns)?;
    let triples = dataset.len()?;
    Ok(DatasetLoad {
        loaded_sources: load.loaded_sources,
        load_errors: load.load_errors,
        ignored_files: load.ignored_files,
        triples,
    })
}

/// Create an empty in-memory dataset for one server instance.
fn new_dataset() -> Result<Store, StorageError> {
    Store::new()
}

/// Log the served directory and listening address.
fn log_serving(directory: &Path, host: &str, port: u16) {
    log::info!("Serving {} at {host}:{port}", directory.display());
}

/// Format source counts and dataset size for the startup log.
fn dataset_summary(stats: DatasetStats) -> String {
    format!(
        "Dataset: {} source file{} loaded; {} source file{} failed; {} file{} ignored; {} triple{} total",
        stats.loaded_files,
        plural_suffix(stats.loaded_files),
        stats.failed_files,
        plural_suffix(stats.failed_files),
        stats.ignored_files,
        plural_suffix(stats.ignored_files),
        stats.triples,
        plural_suffix(stats.triples),
    )
}

/// Return the plural suffix for a count in a log message.
fn plural_suffix(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{Level, LevelFilter, Log, Metadata, Record};
    use std::sync::Mutex;

    static MESSAGE: Mutex<Option<String>> = Mutex::new(None);
    static LOGGER: TestLogger = TestLogger;

    struct TestLogger;

    impl Log for TestLogger {
        fn enabled(&self, metadata: &Metadata<'_>) -> bool {
            metadata.level() <= Level::Info
        }

        fn log(&self, record: &Record<'_>) {
            if self.enabled(record.metadata()) {
                *MESSAGE.lock().unwrap() = Some(record.args().to_string());
            }
        }

        fn flush(&self) {}
    }

    #[test]
    fn logs_the_directory_and_address() {
        log::set_logger(&LOGGER).unwrap();
        log::set_max_level(LevelFilter::Info);

        log_serving(Path::new("docs"), "localhost", 8080);

        assert_eq!(
            MESSAGE.lock().unwrap().as_deref(),
            Some("Serving docs at localhost:8080")
        );
    }

    #[test]
    fn creates_an_empty_dataset() {
        let dataset = new_dataset().unwrap();

        assert!(dataset.is_empty().unwrap());
    }

    #[test]
    fn formats_dataset_statistics() {
        assert_eq!(
            dataset_summary(DatasetStats {
                loaded_files: 3,
                failed_files: 2,
                ignored_files: 6,
                triples: 24,
            }),
            "Dataset: 3 source files loaded; 2 source files failed; 6 files ignored; 24 triples total"
        );
        assert_eq!(
            dataset_summary(DatasetStats {
                loaded_files: 1,
                failed_files: 1,
                ignored_files: 1,
                triples: 1,
            }),
            "Dataset: 1 source file loaded; 1 source file failed; 1 file ignored; 1 triple total"
        );
    }
}
