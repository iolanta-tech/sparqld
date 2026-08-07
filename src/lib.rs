mod loader;
mod server;
mod watcher;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::io;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use oxigraph::store::{StorageError, Store};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 7737;

type SparqldResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
type SharedDataset = Arc<RwLock<Store>>;

struct DatasetLoad {
    loaded_sources: BTreeSet<PathBuf>,
    load_errors: BTreeMap<PathBuf, String>,
    context_dependents: loader::ContextDependents,
    ignored_files: usize,
    triples: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DatasetStats {
    loaded_files: usize,
    failed_files: usize,
    ignored_files: usize,
    triples: usize,
}

/// Controls how a directory is served.
#[derive(Clone, Copy, Debug)]
pub struct ServeOptions {
    /// Reload the dataset when the directory changes.
    pub watch: bool,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self { watch: true }
    }
}

/// Serves the RDF dataset built from `directory` on the default address.
pub fn serve(directory: impl AsRef<Path>) -> SparqldResult<()> {
    serve_at(directory, DEFAULT_HOST, DEFAULT_PORT)
}

/// Serves the RDF dataset built from `directory` on `host` and `port`.
pub fn serve_at(directory: impl AsRef<Path>, host: &str, port: u16) -> SparqldResult<()> {
    serve_at_with_options(directory, host, port, ServeOptions::default())
}

/// Serves the RDF dataset with explicit runtime options.
pub fn serve_at_with_options(
    directory: impl AsRef<Path>,
    host: &str,
    port: u16,
    options: ServeOptions,
) -> SparqldResult<()> {
    let requested_directory = directory.as_ref();
    let directory = requested_directory.canonicalize()?;
    let dataset = Arc::new(RwLock::new(new_dataset()?));
    let (_directory_watcher, stats) = if options.watch {
        let (watcher, stats) =
            watcher::DirectoryWatcher::start(directory.clone(), Arc::clone(&dataset))?;
        (Some(watcher), stats)
    } else {
        let load = reload_dataset(&directory, &dataset)?;
        (None, load.stats())
    };
    let addresses = (host, port).to_socket_addrs()?.collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "host resolved to no addresses",
        )
        .into());
    }

    let listening_server = server::start(dataset, addresses)?;
    log_serving(requested_directory, host, port);
    log::info!("{}", dataset_summary(stats));
    listening_server.join()?;
    Ok(())
}

impl DatasetLoad {
    fn stats(&self) -> DatasetStats {
        DatasetStats {
            loaded_files: self.loaded_sources.len(),
            failed_files: self.load_errors.len(),
            ignored_files: self.ignored_files,
            triples: self.triples,
        }
    }
}

fn reload_dataset(directory: &Path, dataset: &RwLock<Store>) -> SparqldResult<DatasetLoad> {
    let fresh_dataset = new_dataset()?;
    let load = loader::load_directory_with_stats(directory, &fresh_dataset)?;
    let triples = fresh_dataset.len()?;
    *dataset
        .write()
        .map_err(|error| io::Error::other(error.to_string()))? = fresh_dataset;
    Ok(DatasetLoad {
        loaded_sources: load.loaded_sources,
        load_errors: load.load_errors,
        context_dependents: load.context_dependents,
        ignored_files: load.ignored_files,
        triples,
    })
}

fn new_dataset() -> Result<Store, StorageError> {
    Store::new()
}

fn log_serving(directory: &Path, host: &str, port: u16) {
    log::info!("Serving {} at {host}:{port}", directory.display());
}

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
