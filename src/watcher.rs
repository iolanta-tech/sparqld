use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use notify::event::{CreateKind, RemoveKind};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{DatasetStats, SharedDataset, SparqldResult, loader, reload_dataset};

const RELOAD_DEBOUNCE: Duration = Duration::from_millis(500);

enum Message {
    Event(notify::Result<Event>),
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FileChange {
    Created,
    Changed,
    Deleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EntryKind {
    File,
    Directory,
    Unknown,
}

impl EntryKind {
    fn noun(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Directory => "Directory",
            Self::Unknown => "Path",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PathChange {
    change: FileChange,
    entry_kind: EntryKind,
}

impl PathChange {
    fn merge(self, next: Self) -> Self {
        Self {
            change: self.change.merge(next.change),
            entry_kind: match next.entry_kind {
                EntryKind::Unknown => self.entry_kind,
                entry_kind => entry_kind,
            },
        }
    }
}

impl FileChange {
    fn merge(self, next: Self) -> Self {
        match (self, next) {
            (Self::Created, Self::Changed) => Self::Created,
            (Self::Deleted, Self::Created) => Self::Changed,
            (_, next) => next,
        }
    }

    fn verb(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Changed => "changed",
            Self::Deleted => "deleted",
        }
    }
}

pub(crate) struct DirectoryWatcher {
    watcher: Option<RecommendedWatcher>,
    control: Sender<Message>,
    worker: Option<JoinHandle<()>>,
}

impl DirectoryWatcher {
    pub(crate) fn start(
        directory: PathBuf,
        dataset: SharedDataset,
    ) -> SparqldResult<(Self, DatasetStats)> {
        let (control, events) = mpsc::channel();
        let event_control = control.clone();
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = event_control.send(Message::Event(event));
        })?;
        watcher.watch(&directory, RecursiveMode::Recursive)?;

        // The watch is active before the initial load, so changes made while it
        // is loading remain queued for the worker and cannot be missed.
        let load = reload_dataset(&directory, &dataset)?;
        let stats = load.stats();
        let loaded_sources = load.loaded_sources;
        let load_errors = load.load_errors;
        let worker = thread::Builder::new()
            .name("sparqld-watcher".into())
            .spawn(move || watch(events, directory, dataset, loaded_sources, load_errors))?;

        Ok((
            Self {
                watcher: Some(watcher),
                control,
                worker: Some(worker),
            },
            stats,
        ))
    }
}

impl Drop for DirectoryWatcher {
    fn drop(&mut self) {
        self.watcher.take();
        let _ = self.control.send(Message::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn watch(
    events: Receiver<Message>,
    directory: PathBuf,
    dataset: SharedDataset,
    mut loaded_sources: BTreeSet<PathBuf>,
    mut load_errors: BTreeMap<PathBuf, String>,
) {
    let mut reload_at: Option<Instant> = None;
    let mut changed_paths = BTreeMap::new();

    loop {
        let message = if let Some(deadline) = reload_at {
            match events.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok(message) => message,
                Err(RecvTimeoutError::Timeout) => {
                    reload(
                        &directory,
                        &changed_paths,
                        &dataset,
                        &mut loaded_sources,
                        &mut load_errors,
                    );
                    changed_paths.clear();
                    reload_at = None;
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => return,
            }
        } else {
            match events.recv() {
                Ok(message) => message,
                Err(_) => return,
            }
        };

        match message {
            Message::Event(Ok(event)) => {
                let Some(change) = file_change(&event) else {
                    continue;
                };
                let event_entry_kind = explicit_entry_kind(&event).or_else(|| {
                    event
                        .paths
                        .iter()
                        .find_map(|path| existing_entry_kind(path))
                });
                for path in event.paths {
                    let entry_kind = explicit_entry_kind_from_kind(event.kind)
                        .or_else(|| existing_entry_kind(&path))
                        .or(event_entry_kind)
                        .unwrap_or(EntryKind::Unknown);
                    let change = PathChange { change, entry_kind };
                    changed_paths
                        .entry(path)
                        .and_modify(|previous| *previous = previous.merge(change))
                        .or_insert(change);
                }
                reload_at = Some(Instant::now() + RELOAD_DEBOUNCE);
            }
            Message::Event(Err(error)) => log::error!("Filesystem watch error: {error}"),
            Message::Stop => return,
        }
    }
}

fn format_changed_paths(directory: &Path, paths: &BTreeSet<PathBuf>) -> String {
    if paths.is_empty() {
        return "filesystem changes".to_owned();
    }
    paths
        .iter()
        .map(|path| {
            let relative = path.strip_prefix(directory).unwrap_or(path);
            if relative.as_os_str().is_empty() {
                ".".to_owned()
            } else {
                relative.display().to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_changed_path(directory: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(directory).unwrap_or(path);
    if relative.as_os_str().is_empty() {
        ".".to_owned()
    } else {
        relative.display().to_string()
    }
}

fn file_change(event: &Event) -> Option<FileChange> {
    match event.kind {
        EventKind::Any | EventKind::Modify(_) => Some(FileChange::Changed),
        EventKind::Create(_) => Some(FileChange::Created),
        EventKind::Remove(_) => Some(FileChange::Deleted),
        _ => None,
    }
}

fn explicit_entry_kind(event: &Event) -> Option<EntryKind> {
    explicit_entry_kind_from_kind(event.kind)
}

fn explicit_entry_kind_from_kind(kind: EventKind) -> Option<EntryKind> {
    match kind {
        EventKind::Create(CreateKind::File) | EventKind::Remove(RemoveKind::File) => {
            Some(EntryKind::File)
        }
        EventKind::Create(CreateKind::Folder) | EventKind::Remove(RemoveKind::Folder) => {
            Some(EntryKind::Directory)
        }
        _ => None,
    }
}

fn existing_entry_kind(path: &Path) -> Option<EntryKind> {
    if path.is_dir() {
        Some(EntryKind::Directory)
    } else if path.is_file() {
        Some(EntryKind::File)
    } else {
        None
    }
}

fn reload(
    directory: &Path,
    changed_paths: &BTreeMap<PathBuf, PathChange>,
    dataset: &SharedDataset,
    loaded_sources: &mut BTreeSet<PathBuf>,
    load_errors: &mut BTreeMap<PathBuf, String>,
) {
    let path_set = changed_paths.keys().cloned().collect::<BTreeSet<_>>();
    let paths = format_changed_paths(directory, &path_set);
    let dataset = match dataset.read() {
        Ok(dataset) => dataset,
        Err(error) => {
            log::error!("Could not reload {paths}: failed to lock the dataset: {error}");
            return;
        }
    };
    match loader::reload_changed(directory, &path_set, loaded_sources, load_errors, &dataset) {
        Ok(report) => {
            for message in reload_messages(directory, changed_paths, &report) {
                log::info!("{message}");
            }
            *loaded_sources = report.loaded_sources;
            *load_errors = report.load_errors;
        }
        Err(error) => log::error!(
            "Could not reload {paths}: {error}; continuing to serve the previous dataset"
        ),
    }
}

fn reload_messages(
    directory: &Path,
    changed_paths: &BTreeMap<PathBuf, PathChange>,
    report: &loader::ReloadReport,
) -> Vec<String> {
    changed_paths
        .iter()
        .map(|(path, observed)| {
            let relative = path.strip_prefix(directory).unwrap_or(path);
            let direct = report.updates.iter().find(|update| update.path == relative);
            let direct_failure = report
                .failures
                .iter()
                .find(|failure| failure.path == relative);
            let change = if path.exists() {
                observed.change
            } else {
                FileChange::Deleted
            };
            let entry_kind = match observed.entry_kind {
                EntryKind::Unknown if direct.is_some() || direct_failure.is_some() => {
                    EntryKind::File
                }
                entry_kind => entry_kind,
            };
            let prefix = format!(
                "{} {}: {}",
                entry_kind.noun(),
                change.verb(),
                format_changed_path(directory, path)
            );

            if entry_kind == EntryKind::Directory {
                return format!("{prefix}; ignored");
            }

            if let Some(failure) = direct_failure {
                return format!("{prefix}; {}", failure_message(failure));
            }
            if let Some(update) = direct {
                return format!("{prefix}; {}", direct_update_message(update));
            }

            let dependent_updates = report
                .impacts
                .get(relative)
                .into_iter()
                .flatten()
                .filter_map(|source| report.updates.iter().find(|update| &update.path == source))
                .collect::<Vec<_>>();
            let dependent_failures = report
                .impacts
                .get(relative)
                .into_iter()
                .flatten()
                .filter_map(|source| {
                    report
                        .failures
                        .iter()
                        .find(|failure| &failure.path == source)
                })
                .collect::<Vec<_>>();
            if !dependent_failures.is_empty() {
                return match dependent_failures.as_slice() {
                    [failure] => format!("{prefix}; {}", failure_message(failure)),
                    failures => format!(
                        "{prefix}; failed to load dependent source files: {}",
                        failures
                            .iter()
                            .map(|failure| failure.path.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                };
            }
            match dependent_updates.as_slice() {
                [] => format!("{prefix}; ignored"),
                [update] => format!("{prefix}; {}", dependent_update_message(update)),
                updates => format!("{prefix}; updated {} dependent source files", updates.len()),
            }
        })
        .collect()
}

fn failure_message(failure: &loader::SourceFailure) -> String {
    format!(
        "failed to load as {} into {}: {}",
        failure.format, failure.graph, failure.error
    )
}

fn direct_update_message(update: &loader::SourceUpdate) -> String {
    let action = match update.kind {
        loader::SourceUpdateKind::Loaded => "loaded as",
        loader::SourceUpdateKind::Reloaded => "reloaded as",
        loader::SourceUpdateKind::Removed => "removed",
    };
    format!(
        "{action} {} ({})",
        update.graph,
        triple_count(update.triples)
    )
}

fn dependent_update_message(update: &loader::SourceUpdate) -> String {
    let action = match update.kind {
        loader::SourceUpdateKind::Loaded => "loaded",
        loader::SourceUpdateKind::Reloaded => "reloaded",
        loader::SourceUpdateKind::Removed => "removed",
    };
    format!(
        "{action} {} ({})",
        update.graph,
        triple_count(update.triples)
    )
}

fn triple_count(count: usize) -> String {
    let suffix = if count == 1 { "" } else { "s" };
    format!("{count} triple{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;

    use oxigraph::model::{NamedNodeRef, QuadRef};
    use oxigraph::store::Store;
    use tempfile::tempdir;

    #[test]
    fn reloads_created_modified_and_deleted_files() {
        let directory = tempdir().unwrap();
        let dataset = Arc::new(RwLock::new(Store::new().unwrap()));
        let (watcher, stats) =
            DirectoryWatcher::start(directory.path().to_owned(), Arc::clone(&dataset)).unwrap();
        assert_eq!(stats.loaded_files, 0);

        let source = directory.path().join("source.ttl");
        fs::write(&source, "<urn:subject> <urn:predicate> <urn:first> .").unwrap();
        wait_until(|| contains_object(&dataset, "urn:first"));

        fs::write(&source, "<urn:subject> <urn:predicate> <urn:second> .").unwrap();
        wait_until(|| {
            contains_object(&dataset, "urn:second") && !contains_object(&dataset, "urn:first")
        });

        fs::remove_file(source).unwrap();
        wait_until(|| !contains_object(&dataset, "urn:second"));
        drop(watcher);
    }

    #[test]
    fn formats_changed_paths_relative_to_the_served_directory() {
        let directory = Path::new("/srv/documents");
        let paths = BTreeSet::from([
            directory.join("zeta.ttl"),
            directory.join("nested/alpha.md"),
        ]);

        assert_eq!(
            format_changed_paths(directory, &paths),
            "nested/alpha.md, zeta.ttl"
        );
    }

    #[test]
    fn describes_each_files_loader_outcome() {
        let directory = tempdir().unwrap();
        for relative in [
            "stylesheets/extra.css",
            "index.md",
            "index.js",
            "foo.yamlld",
        ] {
            let path = directory.path().join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "source").unwrap();
        }
        let changed_paths = BTreeMap::from([
            (
                directory.path().join("stylesheets/extra.css"),
                PathChange {
                    change: FileChange::Changed,
                    entry_kind: EntryKind::File,
                },
            ),
            (
                directory.path().join("index.md"),
                PathChange {
                    change: FileChange::Changed,
                    entry_kind: EntryKind::File,
                },
            ),
            (
                directory.path().join("index.js"),
                PathChange {
                    change: FileChange::Created,
                    entry_kind: EntryKind::File,
                },
            ),
            (
                directory.path().join("foo.yamlld"),
                PathChange {
                    change: FileChange::Created,
                    entry_kind: EntryKind::File,
                },
            ),
        ]);
        let report = loader::ReloadReport {
            loaded_sources: BTreeSet::from([
                PathBuf::from("foo.yamlld"),
                PathBuf::from("index.md"),
            ]),
            load_errors: BTreeMap::new(),
            updates: vec![
                loader::SourceUpdate {
                    path: PathBuf::from("foo.yamlld"),
                    graph: "sparqld:foo.yamlld".to_owned(),
                    triples: 8,
                    kind: loader::SourceUpdateKind::Loaded,
                },
                loader::SourceUpdate {
                    path: PathBuf::from("index.md"),
                    graph: "sparqld:index.md".to_owned(),
                    triples: 5,
                    kind: loader::SourceUpdateKind::Reloaded,
                },
            ],
            failures: Vec::new(),
            impacts: BTreeMap::from([
                (
                    PathBuf::from("foo.yamlld"),
                    BTreeSet::from([PathBuf::from("foo.yamlld")]),
                ),
                (
                    PathBuf::from("index.md"),
                    BTreeSet::from([PathBuf::from("index.md")]),
                ),
                (PathBuf::from("index.js"), BTreeSet::new()),
                (PathBuf::from("stylesheets/extra.css"), BTreeSet::new()),
            ]),
        };

        assert_eq!(
            reload_messages(directory.path(), &changed_paths, &report),
            [
                "File created: foo.yamlld; loaded as sparqld:foo.yamlld (8 triples)",
                "File created: index.js; ignored",
                "File changed: index.md; reloaded as sparqld:index.md (5 triples)",
                "File changed: stylesheets/extra.css; ignored",
            ]
        );
    }

    #[test]
    fn describes_a_failed_source_with_its_path_and_graph() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("broken.xml");
        fs::write(&source, "<rdf:RDF></rdf:RDF>").unwrap();
        let changed_paths = BTreeMap::from([(
            source,
            PathChange {
                change: FileChange::Created,
                entry_kind: EntryKind::File,
            },
        )]);
        let error = "XML namespaces are required in RDF/XML";
        let report = loader::ReloadReport {
            loaded_sources: BTreeSet::new(),
            load_errors: BTreeMap::from([(PathBuf::from("broken.xml"), error.to_owned())]),
            updates: vec![loader::SourceUpdate {
                path: PathBuf::from("broken.xml"),
                graph: "sparqld:broken.xml".to_owned(),
                triples: 3,
                kind: loader::SourceUpdateKind::Removed,
            }],
            failures: vec![loader::SourceFailure {
                path: PathBuf::from("broken.xml"),
                graph: "sparqld:broken.xml".to_owned(),
                format: "RDF/XML",
                error: error.to_owned(),
            }],
            impacts: BTreeMap::from([(
                PathBuf::from("broken.xml"),
                BTreeSet::from([PathBuf::from("broken.xml")]),
            )]),
        };

        assert_eq!(
            reload_messages(directory.path(), &changed_paths, &report),
            [
                "File created: broken.xml; failed to load as RDF/XML into sparqld:broken.xml: XML namespaces are required in RDF/XML"
            ]
        );
    }

    #[test]
    fn describes_a_deleted_directory_as_a_directory() {
        let directory = tempdir().unwrap();
        let deleted = directory.path().join("examples/context-files/people");
        let changed_paths = BTreeMap::from([(
            deleted,
            PathChange {
                change: FileChange::Deleted,
                entry_kind: EntryKind::Directory,
            },
        )]);
        let report = loader::ReloadReport {
            loaded_sources: BTreeSet::new(),
            load_errors: BTreeMap::new(),
            updates: Vec::new(),
            failures: Vec::new(),
            impacts: BTreeMap::from([(
                PathBuf::from("examples/context-files/people"),
                BTreeSet::new(),
            )]),
        };

        assert_eq!(
            reload_messages(directory.path(), &changed_paths, &report),
            ["Directory deleted: examples/context-files/people; ignored"]
        );
    }

    #[test]
    fn does_not_attribute_a_source_failure_to_its_created_directory() {
        let directory = tempdir().unwrap();
        let created = directory.path().join("conversations");
        fs::create_dir(&created).unwrap();
        let source = created.join("user.txt");
        fs::write(&source, "Use the SPARQL endpoint").unwrap();
        let changed_paths = BTreeMap::from([
            (
                created,
                PathChange {
                    change: FileChange::Created,
                    entry_kind: EntryKind::Directory,
                },
            ),
            (
                source,
                PathChange {
                    change: FileChange::Created,
                    entry_kind: EntryKind::File,
                },
            ),
        ]);
        let error = "The subject of a triple must be an IRI or a blank node";
        let report = loader::ReloadReport {
            loaded_sources: BTreeSet::new(),
            load_errors: BTreeMap::from([(
                PathBuf::from("conversations/user.txt"),
                error.to_owned(),
            )]),
            updates: Vec::new(),
            failures: vec![loader::SourceFailure {
                path: PathBuf::from("conversations/user.txt"),
                graph: "sparqld:conversations/user.txt".to_owned(),
                format: "N-Triples",
                error: error.to_owned(),
            }],
            impacts: BTreeMap::from([
                (
                    PathBuf::from("conversations"),
                    BTreeSet::from([PathBuf::from("conversations/user.txt")]),
                ),
                (
                    PathBuf::from("conversations/user.txt"),
                    BTreeSet::from([PathBuf::from("conversations/user.txt")]),
                ),
            ]),
        };

        assert_eq!(
            reload_messages(directory.path(), &changed_paths, &report),
            [
                "Directory created: conversations; ignored",
                "File created: conversations/user.txt; failed to load as N-Triples into sparqld:conversations/user.txt: The subject of a triple must be an IRI or a blank node",
            ]
        );
    }

    #[test]
    fn takes_the_entry_kind_from_remove_events() {
        let directory = tempdir().unwrap();
        let event = Event::new(EventKind::Remove(RemoveKind::Folder))
            .add_path(directory.path().join("deleted"));

        assert_eq!(explicit_entry_kind(&event), Some(EntryKind::Directory));
    }

    fn contains_object(dataset: &SharedDataset, object: &str) -> bool {
        dataset
            .read()
            .unwrap()
            .contains(QuadRef::new(
                NamedNodeRef::new("urn:subject").unwrap(),
                NamedNodeRef::new("urn:predicate").unwrap(),
                NamedNodeRef::new(object).unwrap(),
                NamedNodeRef::new("sparqld:source.ttl").unwrap(),
            ))
            .unwrap()
    }

    fn wait_until(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if condition() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("timed out waiting for the dataset to reload");
    }
}
