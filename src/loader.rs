//! Load source graphs and maintain file, context, and error metadata.

pub(crate) mod retry;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use oxigraph::io::{JsonLdProfile, JsonLdProfileSet, LoadedDocument, RdfFormat, RdfParser};
use oxigraph::model::{
    BlankNode, GraphName, LiteralRef, NamedNode, NamedNodeRef, NamedOrBlankNode, Quad, QuadRef,
    Term, vocab::rdf,
};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde_json::Value;

use crate::patterns::SourcePatterns;

// @todo(extractor-contributions): Load self-describing extractor output as JSON-LD.
// description: >
//   Match served relative paths against each registered extractor descriptor's
//   advertised globs; do not repeat extractor globs in configuration. For each
//   match, launch one `extract` process and write a JSON request containing the
//   canonical absolute path and root-relative sourceUrl to stdin. Never invoke
//   a shell. Parse successful stdout through the existing JSON-LD loader,
//   including local-context restrictions and named graph rewriting.
// discovery:
//   - Enumerate every regular file once. A file is a source when native RDF
//     format is recognized, at least one extractor matches it, or both do.
//   - Count every other regular file as ignored and retain native behavior for
//     recognized RDF files that no extractor matches.
// ownership:
//   - Track each result by (relative source path, producer), where producer is
//     native input or the stable extractor ID.
//   - Keep native data in sparqld:<path>.
//   - Assign extractor default-graph data to
//     sparqld:<path>@<percent-encoded-id>; namespace embedded graphs below that
//     host-owned contribution graph with the current scoping mechanism.
// failures:
//   - Treat a missing executable, I/O error, invalid JSON request or result,
//     nonzero exit, or invalid stdout JSON-LD as an extractor failure. Remove
//     its prior contribution and record its diagnostic in the rlog catalog.
//   - Detect duplicate urn:riddle:<id> identities across source contributions;
//     catalogue the collision instead of silently merging two riddles.
// acceptance:
//   - Test descriptor glob selection, cwd/argv/stdin, ignored files, JSON-LD
//     triples visible by SPARQL, host graph ownership, malformed stdout,
//     process failure, two extractors for one source, graph rewriting, and
//     duplicate IDs across files.
// blocked-by:
//   - extractor-registration

// @todo(extractor-reload): Replace every graph owned by a changed extractor.
// description: >
//   Extend the current staged reload transaction to replace the complete
//   contribution identified by (relative source path, extractor identity). Run the
//   extractor again, stage all new quads, remove its contribution graph and
//   every graph scoped beneath it, then insert the staged quads atomically.
//   Preserve unrelated native and extractor contributions for the same file.
// failures:
//   - On deletion, remove its contribution, nested graphs, catalog entries, and
//     riddle-ID ownership.
//   - On failed re-extraction, remove stale prior data and retain the new
//     catalogued failure.
// acceptance:
//   - Regression-test an extractor result changing from named graphs A and B
//     to B and C: A is absent and new B/C are present after reload.
//   - Test deletion, failure after success, recovery, duplicate-ID collision
//     introduction/removal, and unaffected native and parallel extractors.
// blocked-by:
//   - extractor-contributions

/// A loading result whose errors can cross service thread boundaries.
type LoadResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Characters escaped when encoding relative file paths as graph IRIs.
const GRAPH_PATH_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

/// Reserved graph containing source, context, and error metadata.
const FILE_CATALOG_GRAPH: NamedNodeRef<'_> = NamedNodeRef::new_unchecked("sparqld:");
/// Predicate linking a source to each context requested during parsing.
const JSONLD_CONTEXT: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("http://www.w3.org/ns/json-ld#context");
/// Class identifying registered JSON-LD contexts.
const JSONLD_CONTEXT_CLASS: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("http://www.w3.org/ns/json-ld#Context");
/// Diagnostic text identifying conflicting context body declarations.
const AMBIGUOUS_TARGETS: &str = "has multiple contentUrl targets";

/// File and directory vocabulary terms used in the catalog.
mod nfo {
    use oxigraph::model::NamedNodeRef;

    /// Construct a constant IRI in the file ontology namespace.
    macro_rules! term {
        ($local_name:literal) => {
            NamedNodeRef::new_unchecked(concat!(
                "http://www.semanticdesktop.org/ontologies/2007/03/22/nfo#",
                $local_name
            ))
        };
    }

    /// Predicate linking a file or directory to its parent directory.
    pub const BELONGS_TO_CONTAINER: NamedNodeRef<'_> = term!("belongsToContainer");
    /// Class identifying a file-backed resource in the catalog.
    pub const FILE_DATA_OBJECT: NamedNodeRef<'_> = term!("FileDataObject");
    /// Predicate recording a file or directory basename.
    pub const FILE_NAME: NamedNodeRef<'_> = term!("fileName");
    /// Class identifying a catalog directory.
    pub const FOLDER: NamedNodeRef<'_> = term!("Folder");
}

/// Logging vocabulary terms used to describe source loading failures.
mod rlog {
    use oxigraph::model::NamedNodeRef;

    /// Construct a constant IRI in the file ontology namespace.
    macro_rules! term {
        ($local_name:literal) => {
            NamedNodeRef::new_unchecked(concat!(
                "http://persistence.uni-leipzig.org/nlp2rdf/ontologies/rlog#",
                $local_name
            ))
        };
    }

    /// Class identifying a recorded loading diagnostic.
    pub const ENTRY: NamedNodeRef<'_> = term!("Entry");
    /// Severity assigned to source loading failures.
    pub const ERROR: NamedNodeRef<'_> = term!("ERROR");
    /// Predicate recording a diagnostic's severity.
    pub const LEVEL: NamedNodeRef<'_> = term!("level");
    /// Predicate recording a diagnostic's human-readable message.
    pub const MESSAGE: NamedNodeRef<'_> = term!("message");
    /// Predicate linking a diagnostic to its source resource.
    pub const RESOURCE: NamedNodeRef<'_> = term!("resource");
}

/// SPARQL service description terms for embedded named graphs.
mod sd {
    use oxigraph::model::NamedNodeRef;

    /// Construct a constant IRI in the file ontology namespace.
    macro_rules! term {
        ($local_name:literal) => {
            NamedNodeRef::new_unchecked(concat!(
                "http://www.w3.org/ns/sparql-service-description#",
                $local_name
            ))
        };
    }

    /// Class assigned to a source containing embedded named graphs.
    pub const DATASET: NamedNodeRef<'_> = term!("Dataset");
    /// Predicate recording an embedded graph's scoped IRI.
    pub const NAME: NamedNodeRef<'_> = term!("name");
    /// Predicate linking a source dataset to an embedded graph description.
    pub const NAMED_GRAPH: NamedNodeRef<'_> = term!("namedGraph");
    /// Class identifying an embedded named graph description.
    pub const NAMED_GRAPH_CLASS: NamedNodeRef<'_> = term!("NamedGraph");
}

#[derive(Clone, Copy)]
/// Parsing strategy selected from a source file's extension.
enum SourceFormat {
    /// An RDF syntax handled directly by Oxigraph.
    Rdf(RdfFormat),
    /// A JSON document parsed as JSON-LD.
    JsonLd,
    /// A YAML document converted to JSON before JSON-LD parsing.
    YamlLd,
    /// A Markdown document whose YAML front matter supplies linked data.
    Markdown,
}

/// Writer for file metadata and observed context dependencies in the catalog graph.
struct FileCatalog<'a> {
    /// Store receiving catalog quads.
    dataset: &'a Store,
}

/// Source state and per-path outcomes after an atomic reload.
pub(crate) struct ReloadReport {
    /// Successfully loaded sources after the reload.
    pub(crate) loaded_sources: BTreeSet<PathBuf>,
    /// Latest errors for sources that failed to load.
    pub(crate) load_errors: BTreeMap<PathBuf, String>,
    /// Successful additions, replacements, and removals.
    pub(crate) updates: Vec<SourceUpdate>,
    /// Sources that failed during this reload attempt.
    pub(crate) failures: Vec<SourceFailure>,
    /// Affected sources grouped by the changed path that triggered them.
    pub(crate) impacts: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
}

/// Source state and ignored-file count from a batch load.
pub(crate) struct DirectoryLoad {
    /// Successfully loaded sources after the reload.
    pub(crate) loaded_sources: BTreeSet<PathBuf>,
    /// Latest errors for sources that failed to load.
    pub(crate) load_errors: BTreeMap<PathBuf, String>,
    /// Number of files skipped by selection or missing Markdown front matter.
    pub(crate) ignored_files: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// How a successful reload operation changed a source graph.
pub(crate) enum SourceUpdateKind {
    /// A previously unloaded source now contributes data.
    Loaded,
    /// An existing source's data was replaced.
    Reloaded,
    /// A source's previous data was removed.
    Removed,
}

#[derive(Debug)]
/// Graph identity and size for one successful source update.
pub(crate) struct SourceUpdate {
    /// Source path relative to the served directory.
    pub(crate) path: PathBuf,
    /// IRI of the source's default contribution graph.
    pub(crate) graph: String,
    /// Number of quads contributed by the source, including embedded graphs.
    pub(crate) triples: usize,
    /// Whether the source was added, replaced, or removed.
    pub(crate) kind: SourceUpdateKind,
}

#[derive(Debug)]
/// Diagnostic for a source that failed during a reload.
pub(crate) struct SourceFailure {
    /// Source path relative to the served directory.
    pub(crate) path: PathBuf,
    /// IRI of the source's default contribution graph.
    pub(crate) graph: String,
    /// Human-readable name of the attempted source format.
    pub(crate) format: &'static str,
    /// Human-readable loading error.
    pub(crate) error: String,
}

impl<'a> FileCatalog<'a> {
    /// Initialize the root directory's metadata in the reserved catalog graph.
    fn new(dataset: &'a Store, root: &Path) -> LoadResult<Self> {
        let catalog = Self { dataset };
        catalog.insert_type(FILE_CATALOG_GRAPH, nfo::FILE_DATA_OBJECT)?;
        catalog.insert_type(FILE_CATALOG_GRAPH, nfo::FOLDER)?;
        if let Some(name) = root.file_name().and_then(|name| name.to_str()) {
            catalog.insert_file_name(FILE_CATALOG_GRAPH, name)?;
        }
        Ok(catalog)
    }

    /// Describe a source, its parent directories, and its embedded named graphs.
    fn describe_source(
        &self,
        relative: &Path,
        embedded_graphs: &BTreeSet<NamedNode>,
    ) -> LoadResult<()> {
        let graph = graph_name(relative)?;
        let container =
            self.describe_directories(relative.parent().unwrap_or_else(|| Path::new("")))?;
        let file_name = relative
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("file name is not valid UTF-8: {}", relative.display()),
                )
            })?;

        self.insert_type(graph.as_ref(), nfo::FILE_DATA_OBJECT)?;
        self.insert_file_name(graph.as_ref(), file_name)?;
        self.insert_container(graph.as_ref(), container.as_ref())?;
        if !embedded_graphs.is_empty() {
            self.insert_type(graph.as_ref(), sd::DATASET)?;
        }
        for embedded_graph in embedded_graphs {
            let description = BlankNode::default();
            self.dataset.insert(QuadRef::new(
                graph.as_ref(),
                sd::NAMED_GRAPH,
                description.as_ref(),
                FILE_CATALOG_GRAPH,
            ))?;
            self.dataset.insert(QuadRef::new(
                description.as_ref(),
                rdf::TYPE,
                sd::NAMED_GRAPH_CLASS,
                FILE_CATALOG_GRAPH,
            ))?;
            self.dataset.insert(QuadRef::new(
                description.as_ref(),
                sd::NAME,
                embedded_graph.as_ref(),
                FILE_CATALOG_GRAPH,
            ))?;
        }
        Ok(())
    }

    /// Record the source's requested contexts as catalog edges, including failed requests.
    fn describe_contexts(&self, relative: &Path, contexts: &BTreeSet<NamedNode>) -> LoadResult<()> {
        let source = graph_name(relative)?;
        for context in contexts {
            self.dataset.insert(QuadRef::new(
                source.as_ref(),
                JSONLD_CONTEXT,
                context.as_ref(),
                FILE_CATALOG_GRAPH,
            ))?;
        }
        Ok(())
    }

    /// Attach a loading diagnostic to the source through RDF logging terms.
    fn describe_load_error(&self, relative: &Path, error: &str) -> LoadResult<()> {
        let resource = graph_name(relative)?;
        let entry = load_error_name(relative)?;

        self.insert_type(entry.as_ref(), rlog::ENTRY)?;
        self.dataset.insert(QuadRef::new(
            entry.as_ref(),
            rlog::LEVEL,
            rlog::ERROR,
            FILE_CATALOG_GRAPH,
        ))?;
        self.dataset.insert(QuadRef::new(
            entry.as_ref(),
            rlog::RESOURCE,
            resource.as_ref(),
            FILE_CATALOG_GRAPH,
        ))?;
        self.dataset.insert(QuadRef::new(
            entry.as_ref(),
            rlog::MESSAGE,
            LiteralRef::new_simple_literal(error),
            FILE_CATALOG_GRAPH,
        ))?;
        Ok(())
    }

    /// Describe each ancestor directory and return the immediate container's IRI.
    fn describe_directories(&self, relative: &Path) -> LoadResult<NamedNode> {
        let mut current = PathBuf::new();
        let mut container = FILE_CATALOG_GRAPH.into_owned();
        for component in relative.components() {
            let Component::Normal(name) = component else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("directory path is not relative: {}", relative.display()),
                )
                .into());
            };
            current.push(name);
            let directory = NamedNode::new(path_iri(&current, true)?)?;
            let name = name.to_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("directory name is not valid UTF-8: {}", current.display()),
                )
            })?;

            self.insert_type(directory.as_ref(), nfo::FILE_DATA_OBJECT)?;
            self.insert_type(directory.as_ref(), nfo::FOLDER)?;
            self.insert_file_name(directory.as_ref(), name)?;
            self.insert_container(directory.as_ref(), container.as_ref())?;
            container = directory;
        }
        Ok(container)
    }

    /// Insert a resource's RDF class into the file catalog.
    fn insert_type(&self, resource: NamedNodeRef<'_>, class: NamedNodeRef<'_>) -> LoadResult<()> {
        self.dataset
            .insert(QuadRef::new(resource, rdf::TYPE, class, FILE_CATALOG_GRAPH))?;
        Ok(())
    }

    /// Record a resource's basename in the file catalog.
    fn insert_file_name(&self, resource: NamedNodeRef<'_>, name: &str) -> LoadResult<()> {
        self.dataset.insert(QuadRef::new(
            resource,
            nfo::FILE_NAME,
            LiteralRef::new_simple_literal(name),
            FILE_CATALOG_GRAPH,
        ))?;
        Ok(())
    }

    /// Link a file or directory to its parent in the file catalog.
    fn insert_container(
        &self,
        resource: NamedNodeRef<'_>,
        container: NamedNodeRef<'_>,
    ) -> LoadResult<()> {
        self.dataset.insert(QuadRef::new(
            resource,
            nfo::BELONGS_TO_CONTAINER,
            container,
            FILE_CATALOG_GRAPH,
        ))?;
        Ok(())
    }
}

#[cfg(test)]
fn load_directory(directory: &Path, dataset: &Store) -> LoadResult<BTreeSet<PathBuf>> {
    Ok(load_directory_with_stats(directory, dataset)?.loaded_sources)
}

#[cfg(test)]
pub(crate) fn load_directory_with_stats(
    directory: &Path,
    dataset: &Store,
) -> LoadResult<DirectoryLoad> {
    load_directory_with_patterns(directory, dataset, &SourcePatterns::all())
}

/// Discover selected sources, load their data and catalog entries, and log failures.
pub(crate) fn load_directory_with_patterns(
    directory: &Path,
    dataset: &Store,
    source_patterns: &SourcePatterns,
) -> LoadResult<DirectoryLoad> {
    let root = directory.canonicalize()?;
    FileCatalog::new(dataset, &root)?;
    let files = directory_files(&root)?;
    let sources = source_paths(&root, source_patterns)?;
    let report = retry::reload_with_retries(
        &root,
        &sources,
        &BTreeSet::new(),
        &BTreeMap::new(),
        dataset,
        source_patterns,
    )?;
    Ok(DirectoryLoad {
        ignored_files: files.len() - report.loaded_sources.len() - report.load_errors.len(),
        loaded_sources: report.loaded_sources,
        load_errors: report.load_errors,
    })
}

#[cfg(test)]
pub(crate) fn reload_changed(
    directory: &Path,
    changed_paths: &BTreeSet<PathBuf>,
    loaded_sources: &BTreeSet<PathBuf>,
    load_errors: &BTreeMap<PathBuf, String>,
    dataset: &Store,
) -> LoadResult<ReloadReport> {
    reload_changed_with_patterns(
        directory,
        changed_paths,
        loaded_sources,
        load_errors,
        dataset,
        &SourcePatterns::all(),
    )
}

/// Build a source diagnostic from a relative path and load error.
fn source_failure(path: &Path, error: &str) -> LoadResult<SourceFailure> {
    let format = source_format(path)
        .ok_or_else(|| io::Error::other("failed source has no recognized format"))?;
    Ok(SourceFailure {
        graph: graph_name(path)?.as_str().to_owned(),
        path: path.to_owned(),
        format: format.name(),
        error: error.to_owned(),
    })
}

/// Reload changed files and their context dependents in one transaction.
pub(crate) fn reload_changed_with_patterns(
    directory: &Path,
    changed_paths: &BTreeSet<PathBuf>,
    loaded_sources: &BTreeSet<PathBuf>,
    load_errors: &BTreeMap<PathBuf, String>,
    dataset: &Store,
    source_patterns: &SourcePatterns,
) -> LoadResult<ReloadReport> {
    let root = directory.canonicalize()?;
    let current_sources = source_paths(&root, source_patterns)?;
    let tracked_sources = loaded_sources
        .iter()
        .chain(load_errors.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut impacts = source_impacts(
        &root,
        changed_paths,
        &tracked_sources,
        &current_sources,
        dataset,
    )?;
    let mut affected = impacts.values().flatten().cloned().collect::<BTreeSet<_>>();

    if affected.is_empty() {
        return Ok(ReloadReport {
            loaded_sources: loaded_sources.clone(),
            load_errors: load_errors.clone(),
            updates: Vec::new(),
            failures: Vec::new(),
            impacts,
        });
    }

    let staging = Store::new()?;
    let mut staged_sources = BTreeSet::new();
    let mut failed_sources = BTreeMap::new();
    let mut excluded_graphs = BTreeSet::new();
    for relative in &affected {
        excluded_graphs.insert(graph_name(relative)?);
        excluded_graphs.extend(scoped_graphs_for_source(dataset, relative)?);
    }
    let registrations = context_bodies(&lookup_store(dataset, &staging, &excluded_graphs)?)?;
    let staging_catalog = FileCatalog { dataset: &staging };
    let yaml_json = Arc::new(Mutex::new(BTreeMap::new()));
    for relative in affected.intersection(&current_sources) {
        let mut state = ContextLoadState {
            dependencies: BTreeSet::new(),
            yaml_json: Arc::clone(&yaml_json),
        };
        match load_file(
            &root,
            &root.join(relative),
            &staging,
            &registrations,
            &mut state,
        ) {
            Ok(Some(_)) => {
                staged_sources.insert(relative.clone());
            }
            Ok(None) => {}
            Err(error) => {
                staging.remove_named_graph(graph_name(relative)?.as_ref())?;
                failed_sources.insert(relative.clone(), error.to_string());
            }
        }
        staging_catalog.describe_contexts(relative, &state.dependencies)?;
    }
    let visible = lookup_store(dataset, &staging, &excluded_graphs)?;
    for (declaration, error) in ambiguous_declarations(&visible, &current_sources)? {
        let invalidated = source_impacts(
            &root,
            &BTreeSet::from([declaration]),
            &tracked_sources,
            &current_sources,
            &visible,
        )?;
        for (changed, sources) in invalidated {
            for source in &sources {
                let graph = graph_name(source)?;
                if affected.insert(source.clone()) {
                    for dependency in dataset.quads_for_pattern(
                        Some(graph.as_ref().into()),
                        Some(JSONLD_CONTEXT),
                        None,
                        Some(FILE_CATALOG_GRAPH.into()),
                    ) {
                        staging.insert(dependency?.as_ref())?;
                    }
                }
                staging.remove_named_graph(graph.as_ref())?;
                for graph in scoped_graphs_for_source(&staging, source)? {
                    staging.remove_named_graph(graph.as_ref())?;
                }
                staged_sources.remove(source);
                failed_sources.insert(source.clone(), error.clone());
            }
            impacts.entry(changed).or_default().extend(sources);
        }
    }

    let mut next_sources = loaded_sources.clone();
    let mut next_errors = load_errors.clone();
    for relative in &affected {
        next_sources.remove(relative);
        next_errors.remove(relative);
        if failed_sources.contains_key(relative) {
            next_errors.insert(relative.clone(), failed_sources[relative].clone());
        } else if staged_sources.contains(relative) {
            next_sources.insert(relative.clone());
        }
    }

    let catalog = reload_catalog(
        dataset,
        &staging,
        &root,
        &next_sources,
        &next_errors,
        &affected,
    )?;
    let staged_quads = collect_quads(&staging)?
        .into_iter()
        .filter(|quad| quad.graph_name != FILE_CATALOG_GRAPH.into())
        .collect::<Vec<_>>();
    let catalog_quads = collect_quads(&catalog)?;
    let removed_triples = affected
        .iter()
        .filter(|relative| loaded_sources.contains(*relative) && !next_sources.contains(*relative))
        .map(|relative| Ok((relative.clone(), source_triple_count(dataset, relative)?)))
        .collect::<LoadResult<HashMap<_, _>>>()?;
    let mut transaction = dataset.start_transaction()?;
    for relative in &affected {
        transaction.remove_named_graph(graph_name(relative)?.as_ref())?;
        for graph in scoped_graphs_for_source(dataset, relative)? {
            transaction.remove_named_graph(graph.as_ref())?;
        }
    }
    transaction.remove_named_graph(FILE_CATALOG_GRAPH)?;
    for quad in staged_quads.iter().chain(&catalog_quads) {
        transaction.insert(quad.as_ref());
    }
    transaction.commit()?;

    let mut updates = Vec::new();
    for relative in &affected {
        let graph = graph_name(relative)?;
        if staged_sources.contains(relative) {
            updates.push(SourceUpdate {
                path: relative.clone(),
                graph: graph.as_str().to_owned(),
                triples: source_triple_count(&staging, relative)?,
                kind: if loaded_sources.contains(relative) {
                    SourceUpdateKind::Reloaded
                } else {
                    SourceUpdateKind::Loaded
                },
            });
        } else if loaded_sources.contains(relative) && !next_sources.contains(relative) {
            updates.push(SourceUpdate {
                path: relative.clone(),
                graph: graph.as_str().to_owned(),
                triples: removed_triples[relative],
                kind: SourceUpdateKind::Removed,
            });
        }
    }

    let failures = failed_sources
        .iter()
        .map(|(path, error)| source_failure(path, error))
        .collect::<LoadResult<Vec<_>>>()?;

    Ok(ReloadReport {
        loaded_sources: next_sources,
        load_errors: next_errors,
        updates,
        failures,
        impacts,
    })
}

/// Count the quads in one named graph.
fn graph_triple_count(dataset: &Store, graph: NamedNodeRef<'_>) -> LoadResult<usize> {
    dataset
        .quads_for_pattern(None, None, None, Some(graph.into()))
        .try_fold(0, |count, quad| {
            quad?;
            Ok(count + 1)
        })
}

/// Count a source's quads across its default and scoped embedded graphs.
fn source_triple_count(dataset: &Store, relative: &Path) -> LoadResult<usize> {
    let graph = graph_name(relative)?;
    let embedded_graphs = scoped_graphs_for_source(dataset, relative)?;
    embedded_graphs.iter().try_fold(
        graph_triple_count(dataset, graph.as_ref())?,
        |count, graph| Ok(count + graph_triple_count(dataset, graph.as_ref())?),
    )
}

/// Collect selected source paths relative to the served root.
fn source_paths(root: &Path, source_patterns: &SourcePatterns) -> LoadResult<BTreeSet<PathBuf>> {
    source_files(root, source_patterns)?
        .into_iter()
        .map(|file| {
            file.strip_prefix(root)
                .map(Path::to_owned)
                .map_err(|error| io::Error::other(error).into())
        })
        .collect()
}

/// Map filesystem changes to tracked sources using the catalog's context dependencies.
fn source_impacts(
    root: &Path,
    changed_paths: &BTreeSet<PathBuf>,
    loaded_sources: &BTreeSet<PathBuf>,
    current_sources: &BTreeSet<PathBuf>,
    dataset: &Store,
) -> LoadResult<BTreeMap<PathBuf, BTreeSet<PathBuf>>> {
    let request_sources = context_request_sources(dataset)?;
    let bodies = context_bodies(dataset)?;
    let mut impacts = BTreeMap::new();
    for changed in changed_paths {
        let changed = if changed.is_absolute() {
            let Ok(relative) = changed.strip_prefix(root) else {
                continue;
            };
            relative
        } else {
            changed.as_path()
        };

        let mut affected = BTreeSet::new();
        for dependency in dataset.quads_for_pattern(
            None,
            Some(JSONLD_CONTEXT),
            None,
            Some(FILE_CATALOG_GRAPH.into()),
        ) {
            let dependency = dependency?;
            let (NamedOrBlankNode::NamedNode(source), Term::NamedNode(context)) =
                (dependency.subject, dependency.object)
            else {
                continue;
            };
            let (Ok(source), Ok(context)) = (
                local_context_relative(source.as_str()),
                local_context_relative(context.as_str()),
            ) else {
                continue;
            };
            if context.starts_with(changed) && loaded_sources.contains(&source) {
                affected.insert(source);
            }
        }
        for (context, body_iris) in &bodies {
            for body_iri in body_iris {
                let Ok(body_path) = local_context_relative(body_iri) else {
                    continue;
                };
                if !body_path.starts_with(changed) {
                    continue;
                }
                if let Some(sources) = request_sources.get(context) {
                    affected.extend(
                        sources
                            .iter()
                            .filter(|source| loaded_sources.contains(*source))
                            .cloned(),
                    );
                }
            }
        }
        if loaded_sources.contains(changed) || current_sources.contains(changed) {
            affected.insert(changed.to_owned());
        } else if root.join(changed).is_dir()
            || loaded_sources
                .iter()
                .any(|source| source.starts_with(changed))
            || current_sources
                .iter()
                .any(|source| source.starts_with(changed))
        {
            affected.extend(
                loaded_sources
                    .union(current_sources)
                    .filter(|source| source.starts_with(changed))
                    .cloned(),
            );
        }
        loop {
            let mut extra = BTreeSet::new();
            for source in &affected {
                for context in declared_context_iris(dataset, source)? {
                    if let Some(sources) = request_sources.get(&context) {
                        extra.extend(
                            sources
                                .iter()
                                .filter(|dependent| loaded_sources.contains(*dependent))
                                .cloned(),
                        );
                    }
                }
            }
            let new: BTreeSet<_> = extra.difference(&affected).cloned().collect();
            if new.is_empty() {
                break;
            }
            affected.extend(new);
        }
        impacts.insert(changed.to_owned(), affected);
    }
    Ok(impacts)
}

/// Index catalog context requests by context IRI.
fn context_request_sources(dataset: &Store) -> LoadResult<BTreeMap<String, BTreeSet<PathBuf>>> {
    let mut requests: BTreeMap<String, BTreeSet<PathBuf>> = BTreeMap::new();
    for dependency in dataset.quads_for_pattern(
        None,
        Some(JSONLD_CONTEXT),
        None,
        Some(FILE_CATALOG_GRAPH.into()),
    ) {
        let dependency = dependency?;
        let (NamedOrBlankNode::NamedNode(source), Term::NamedNode(context)) =
            (dependency.subject, dependency.object)
        else {
            continue;
        };
        let Ok(source) = local_context_relative(source.as_str()) else {
            continue;
        };
        requests
            .entry(context.as_str().to_owned())
            .or_default()
            .insert(source);
    }
    Ok(requests)
}

/// Find context declarations contributed by a source and its embedded graphs.
fn declared_context_iris(dataset: &Store, source: &Path) -> LoadResult<BTreeSet<String>> {
    let mut graphs = scoped_graphs_for_source(dataset, source)?;
    graphs.insert(graph_name(source)?);
    let content_url = NamedNodeRef::new_unchecked("https://schema.org/contentUrl");
    let mut iris = BTreeSet::new();
    for graph in graphs {
        for quad in dataset.quads_for_pattern(None, None, None, Some(graph.as_ref().into())) {
            let quad = quad?;
            if ((quad.predicate == rdf::TYPE && quad.object == JSONLD_CONTEXT_CLASS.into())
                || quad.predicate == content_url)
                && let NamedOrBlankNode::NamedNode(subject) = quad.subject
            {
                iris.insert(subject.as_str().to_owned());
            }
        }
    }
    Ok(iris)
}

/// Build replacement metadata from staged and unaffected source contributions.
fn reload_catalog(
    published: &Store,
    staging: &Store,
    root: &Path,
    sources: &BTreeSet<PathBuf>,
    load_errors: &BTreeMap<PathBuf, String>,
    affected: &BTreeSet<PathBuf>,
) -> LoadResult<Store> {
    let catalog = Store::new()?;
    let file_catalog = FileCatalog::new(&catalog, root)?;
    for source in sources.iter().chain(load_errors.keys()) {
        let embedded = if affected.contains(source) {
            scoped_graphs_for_source(staging, source)?
        } else {
            scoped_graphs_for_source(published, source)?
        };
        file_catalog.describe_source(source, &embedded)?;
    }
    for (source, error) in load_errors {
        file_catalog.describe_load_error(source, error)?;
    }
    let affected_graphs = affected
        .iter()
        .map(|source| graph_name(source))
        .collect::<LoadResult<BTreeSet<_>>>()?;
    for previous in published.quads_for_pattern(
        None,
        Some(JSONLD_CONTEXT),
        None,
        Some(FILE_CATALOG_GRAPH.into()),
    ) {
        let previous = previous?;
        let replaced = match &previous.subject {
            NamedOrBlankNode::NamedNode(source) => affected_graphs.contains(source),
            NamedOrBlankNode::BlankNode(_) => false,
        };
        if !replaced {
            catalog.insert(previous.as_ref())?;
        }
    }
    for dependency in staging.quads_for_pattern(
        None,
        Some(JSONLD_CONTEXT),
        None,
        Some(FILE_CATALOG_GRAPH.into()),
    ) {
        catalog.insert(dependency?.as_ref())?;
    }
    record_requested_context_types(&catalog)?;
    Ok(catalog)
}

/// Describe requested remote contexts in the catalog.
fn record_requested_context_types(catalog: &Store) -> LoadResult<()> {
    let edges = catalog
        .quads_for_pattern(
            None,
            Some(JSONLD_CONTEXT),
            None,
            Some(FILE_CATALOG_GRAPH.into()),
        )
        .collect::<Result<Vec<_>, _>>()?;
    for edge in edges {
        let Term::NamedNode(url) = &edge.object else {
            continue;
        };
        if local_context_relative(url.as_str()).is_ok() {
            continue;
        }
        catalog.insert(QuadRef::new(
            url.as_ref(),
            rdf::TYPE,
            JSONLD_CONTEXT_CLASS,
            FILE_CATALOG_GRAPH,
        ))?;
    }
    Ok(())
}

/// Find named graphs whose IRIs are scoped beneath the source graph.
fn scoped_graphs_for_source(dataset: &Store, relative: &Path) -> LoadResult<BTreeSet<NamedNode>> {
    let prefix = format!("{}#", graph_name(relative)?.as_str());
    dataset
        .named_graphs()
        .filter_map(|graph| match graph {
            Ok(NamedOrBlankNode::NamedNode(graph)) if graph.as_str().starts_with(&prefix) => {
                Some(Ok(graph))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}

/// Read every quad from a store, propagating iteration errors.
fn collect_quads(dataset: &Store) -> LoadResult<Vec<Quad>> {
    dataset
        .into_iter()
        .collect::<Result<_, _>>()
        .map_err(Into::into)
}

/// Find files with supported formats that match the source selection patterns.
fn source_files(directory: &Path, source_patterns: &SourcePatterns) -> LoadResult<Vec<PathBuf>> {
    Ok(directory_files(directory)?
        .into_iter()
        .filter(|path| {
            source_format(path).is_some()
                && path
                    .strip_prefix(directory)
                    .is_ok_and(|relative| source_patterns.matches(relative))
        })
        .collect())
}

/// Collect regular files recursively beneath the directory.
fn directory_files(directory: &Path) -> LoadResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_directory_files(directory, &mut files)?;
    Ok(files)
}

/// Append regular files from a directory and its child directories.
fn collect_directory_files(directory: &Path, files: &mut Vec<PathBuf>) -> LoadResult<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_directory_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

/// Context dependencies and shared YAML conversion cache for a source load.
struct ContextLoadState {
    /// Context IRIs requested while parsing the source.
    dependencies: BTreeSet<NamedNode>,
    /// Converted YAML context bodies shared within one reload.
    yaml_json: Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
}

/// Read and parse a source, returning its embedded graphs or absent Markdown front matter.
fn load_file(
    root: &Path,
    file: &Path,
    dataset: &Store,
    registrations: &BTreeMap<String, BTreeSet<String>>,
    state: &mut ContextLoadState,
) -> LoadResult<Option<BTreeSet<NamedNode>>> {
    let format = source_format(file).ok_or_else(|| io::Error::other("unknown RDF format"))?;
    let relative = file.strip_prefix(root).map_err(io::Error::other)?;
    let graph = graph_name(relative)?;
    let base_iri = directory_iri(relative)?;
    let source = fs::read(file)?;
    let embedded_graphs = match format {
        SourceFormat::Rdf(format) => {
            let parser = RdfParser::from_format(format).with_base_iri(base_iri)?;
            load_scoped_quads(
                dataset,
                parser.for_slice(&source).collect::<Result<Vec<_>, _>>()?,
                graph,
            )?
        }
        SourceFormat::JsonLd => load_json_ld(
            root,
            &source,
            &base_iri,
            graph,
            dataset,
            registrations,
            state,
        )?,
        SourceFormat::YamlLd => load_json_ld(
            root,
            &cached_yaml_json(file, &state.yaml_json)?,
            &base_iri,
            graph,
            dataset,
            registrations,
            state,
        )?,
        SourceFormat::Markdown => {
            let Some(front_matter) = markdown_front_matter(&source)? else {
                return Ok(None);
            };
            load_json_ld(
                root,
                &serde_json::to_vec(&parse_yaml_ld(front_matter)?)?,
                &base_iri,
                graph,
                dataset,
                registrations,
                state,
            )?
        }
    };
    Ok(Some(embedded_graphs))
}

/// Select a parser from the file extension, accepting JSON and YAML linked-data forms.
fn source_format(path: &Path) -> Option<SourceFormat> {
    let extension = path.extension()?.to_str()?;
    match extension {
        extension
            if extension.eq_ignore_ascii_case("jsonld")
                || extension.eq_ignore_ascii_case("json") =>
        {
            Some(SourceFormat::JsonLd)
        }
        extension if extension.eq_ignore_ascii_case("yamlld") => Some(SourceFormat::YamlLd),
        extension if extension.eq_ignore_ascii_case("md") => Some(SourceFormat::Markdown),
        extension => RdfFormat::from_extension(extension).map(SourceFormat::Rdf),
    }
}

impl SourceFormat {
    /// Return the source format's display name for diagnostics.
    fn name(&self) -> &'static str {
        match self {
            Self::Rdf(format) => format.name(),
            Self::JsonLd => "JSON-LD",
            Self::YamlLd => "YAML-LD",
            Self::Markdown => "Markdown-LD",
        }
    }
}

/// Parse JSON-LD with local contexts and retain requested URLs even when parsing fails.
fn load_json_ld(
    root: &Path,
    source: &[u8],
    base_iri: &str,
    graph: NamedNode,
    dataset: &Store,
    registrations: &BTreeMap<String, BTreeSet<String>>,
    state: &mut ContextLoadState,
) -> LoadResult<BTreeSet<NamedNode>> {
    let root = root.to_owned();
    let registrations = registrations.clone();
    let yaml_json = Arc::clone(&state.yaml_json);
    let loaded_contexts = Arc::new(Mutex::new(BTreeSet::new()));
    let context_dependencies = Arc::clone(&loaded_contexts);
    let parser = RdfParser::from_format(RdfFormat::JsonLd {
        profile: JsonLdProfileSet::empty(),
    })
    .with_base_iri(base_iri)?;
    let result = (|| {
        let quads = parser
            .for_slice(source)
            .with_document_loader(move |url| {
                context_dependencies
                    .lock()
                    .map_err(|error| io::Error::other(error.to_string()))?
                    .insert(NamedNode::new(url)?);
                load_context_document(&root, url, &registrations, &yaml_json)
            })
            .collect::<Result<Vec<_>, _>>()?;
        load_scoped_quads(dataset, quads, graph)
    })();
    state.dependencies.extend(
        loaded_contexts
            .lock()
            .map_err(|error| io::Error::other(error.to_string()))?
            .iter()
            .cloned(),
    );
    note_requested_contexts(dataset, &state.dependencies)?;
    result
}

/// Add catalog types for remote contexts requested during parsing.
fn note_requested_contexts(dataset: &Store, requested: &BTreeSet<NamedNode>) -> LoadResult<()> {
    for url in requested {
        if local_context_relative(url.as_str()).is_ok() {
            continue;
        }
        dataset.insert(QuadRef::new(
            url.as_ref(),
            rdf::TYPE,
            JSONLD_CONTEXT_CLASS,
            FILE_CATALOG_GRAPH,
        ))?;
    }
    Ok(())
}

/// Scope embedded graph identities to the source and insert the rewritten quads.
fn load_scoped_quads(
    dataset: &Store,
    mut quads: Vec<Quad>,
    source_graph: NamedNode,
) -> LoadResult<BTreeSet<NamedNode>> {
    let mut scoped_graphs = HashMap::new();
    for quad in &quads {
        if !quad.graph_name.is_default_graph() {
            let graph_name = quad.graph_name.clone();
            scoped_graphs
                .entry(graph_name.clone())
                .or_insert(scoped_graph_name(&source_graph, &graph_name)?);
        }
    }

    for quad in &mut quads {
        *quad = scope_quad(quad.clone(), &source_graph, &scoped_graphs);
        dataset.insert(quad.as_ref())?;
    }

    Ok(scoped_graphs.into_values().collect())
}

/// Construct a source-owned IRI for an embedded named or blank-node graph.
fn scoped_graph_name(source_graph: &NamedNode, graph_name: &GraphName) -> LoadResult<NamedNode> {
    let embedded_name = match graph_name {
        GraphName::NamedNode(node) => node.as_str().to_owned(),
        GraphName::BlankNode(node) => format!("_:{}", node.as_str()),
        GraphName::DefaultGraph => unreachable!("default graphs are not scoped"),
    };
    Ok(NamedNode::new(format!(
        "{}#{embedded_name}",
        source_graph.as_str()
    ))?)
}

/// Rewrite a quad's graph and references to embedded graphs into the source's namespace.
fn scope_quad(
    quad: Quad,
    source_graph: &NamedNode,
    scoped_graphs: &HashMap<GraphName, NamedNode>,
) -> Quad {
    let map_named_or_blank = |term: NamedOrBlankNode| -> NamedOrBlankNode {
        match term {
            NamedOrBlankNode::NamedNode(node) => scoped_graphs
                .get(&GraphName::NamedNode(node.clone()))
                .cloned()
                .map(NamedOrBlankNode::from)
                .unwrap_or_else(|| NamedOrBlankNode::from(node)),
            NamedOrBlankNode::BlankNode(node) => scoped_graphs
                .get(&GraphName::BlankNode(node.clone()))
                .cloned()
                .map(NamedOrBlankNode::from)
                .unwrap_or_else(|| NamedOrBlankNode::from(node)),
        }
    };
    let map_term = |term: Term| -> Term {
        match term {
            Term::NamedNode(node) => scoped_graphs
                .get(&GraphName::NamedNode(node.clone()))
                .cloned()
                .map(Term::from)
                .unwrap_or_else(|| Term::from(node)),
            Term::BlankNode(node) => scoped_graphs
                .get(&GraphName::BlankNode(node.clone()))
                .cloned()
                .map(Term::from)
                .unwrap_or_else(|| Term::from(node)),
            Term::Literal(literal) => Term::from(literal),
        }
    };
    let predicate = scoped_graphs
        .get(&GraphName::NamedNode(quad.predicate.clone()))
        .cloned()
        .unwrap_or(quad.predicate);
    let graph_name: GraphName = match quad.graph_name {
        GraphName::DefaultGraph => source_graph.clone().into(),
        graph_name => scoped_graphs[&graph_name].clone().into(),
    };
    Quad::new(
        map_named_or_blank(quad.subject),
        predicate,
        map_term(quad.object),
        graph_name,
    )
}

/// Load the bundled convenience context or a JSON-LD or YAML-LD context beneath the root.
fn load_context_document(
    root: &Path,
    url: &str,
    registrations: &BTreeMap<String, BTreeSet<String>>,
    yaml_json: &Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
) -> LoadResult<LoadedDocument> {
    let relative = match local_context_relative(url) {
        Ok(relative) => relative,
        Err(_) => {
            let body = registered_context_body(registrations, url)?;
            match local_context_relative(body.as_str()) {
                Ok(relative) => relative,
                Err(error) if body.as_str().starts_with("sparqld:") => return Err(error),
                Err(_) => {
                    return Err(io::Error::other(format!(
                        "contentUrl is not a local file: {}",
                        body.as_str()
                    ))
                    .into());
                }
            }
        }
    };
    let path = local_context_path(root, &relative, url)?;
    let content = match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension)
            if extension.eq_ignore_ascii_case("jsonld")
                || extension.eq_ignore_ascii_case("json") =>
        {
            fs::read(&path)?
        }
        Some(extension) if extension.eq_ignore_ascii_case("yamlld") => {
            cached_yaml_json(&path, yaml_json)?
        }
        _ => {
            return Err(
                io::Error::other(format!("unsupported local context format: {url}")).into(),
            );
        }
    };
    Ok(LoadedDocument {
        url: url.into(),
        content,
        format: context_format(),
    })
}

/// Convert a YAML context once and reuse its JSON representation.
fn cached_yaml_json(
    path: &Path,
    cache: &Arc<Mutex<BTreeMap<PathBuf, Vec<u8>>>>,
) -> LoadResult<Vec<u8>> {
    let path = path.canonicalize()?;
    let mut cache = cache
        .lock()
        .map_err(|error| io::Error::other(error.to_string()))?;
    if let Some(json) = cache.get(&path) {
        return Ok(json.clone());
    }
    let json = serde_json::to_vec(&parse_yaml_ld(&fs::read(&path)?)?)?;
    cache.insert(path, json.clone());
    Ok(json)
}

/// Select the JSON-LD context profile for document-loader responses.
fn context_format() -> RdfFormat {
    RdfFormat::JsonLd {
        profile: JsonLdProfile::Context.into(),
    }
}

/// Decode a sparqld context IRI into a relative path, rejecting traversal components.
fn local_context_relative(url: &str) -> LoadResult<PathBuf> {
    let Some(relative) = url.strip_prefix("sparqld:") else {
        return Err(io::Error::other(format!("context URL is not supported: {url}")).into());
    };
    let relative = percent_decode_str(relative)
        .decode_utf8()
        .map_err(|_| io::Error::other(format!("context URL is not UTF-8: {url}")))?;
    let relative = Path::new(relative.as_ref());
    if relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(
            io::Error::other(format!("context is outside the served directory: {url}")).into(),
        );
    }
    Ok(relative.to_owned())
}

/// Resolve a context file canonically and reject paths outside the served root.
fn local_context_path(root: &Path, relative: &Path, url: &str) -> LoadResult<PathBuf> {
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(root) {
        return Err(
            io::Error::other(format!("context is outside the served directory: {url}")).into(),
        );
    }
    Ok(path)
}

/// Parse YAML using YAML 1.2 scalar rules into a JSON-compatible value.
fn parse_yaml_ld(source: &[u8]) -> LoadResult<Value> {
    let options = serde_saphyr::options! {
        legacy_octal_numbers: false,
        strict_booleans: true,
    };
    Ok(serde_saphyr::from_slice_with_options(source, options)?)
}

/// Extract delimited YAML front matter and report an unterminated header.
fn markdown_front_matter(source: &[u8]) -> LoadResult<Option<&[u8]>> {
    let source = std::str::from_utf8(source)?;
    let mut lines = source.split_inclusive('\n');
    let Some(opening) = lines.next() else {
        return Ok(None);
    };
    if trim_line_ending(opening) != "---" {
        return Ok(None);
    }

    let start = opening.len();
    let mut end = start;
    for line in lines {
        if matches!(trim_line_ending(line), "---" | "...") {
            return Ok(Some(&source.as_bytes()[start..end]));
        }
        end += line.len();
    }

    Err(io::Error::new(io::ErrorKind::InvalidData, "unterminated YAML front matter").into())
}

/// Remove CR and LF terminators from a line used in delimiter matching.
fn trim_line_ending(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

/// Construct the graph IRI for a relative source path.
fn graph_name(relative: &Path) -> LoadResult<NamedNode> {
    Ok(NamedNode::new(path_iri(relative, false)?)?)
}

/// Construct the base IRI used to resolve references from a source's parent directory.
fn directory_iri(relative: &Path) -> LoadResult<String> {
    path_iri(relative.parent().unwrap_or_else(|| Path::new("")), true)
}

/// Combine unaffected published graphs with staged contributions for context lookup.
fn lookup_store(
    dataset: &Store,
    staging: &Store,
    excluded: &BTreeSet<NamedNode>,
) -> LoadResult<Store> {
    let lookup = Store::new()?;
    for quad in dataset.quads_for_pattern(None, None, None, None) {
        let quad = quad?;
        if let GraphName::NamedNode(graph) = &quad.graph_name
            && excluded.contains(graph)
        {
            continue;
        }
        lookup.insert(quad.as_ref())?;
    }
    for quad in staging.quads_for_pattern(None, None, None, None) {
        lookup.insert(quad?.as_ref())?;
    }
    Ok(lookup)
}

/// Evaluate a SPARQL query over the union of source graphs.
fn evaluate_union<'a>(dataset: &'a Store, query: &str) -> LoadResult<QueryResults<'a>> {
    let mut prepared = SparqlEvaluator::new().parse_query(query)?;
    prepared.dataset_mut().set_default_graph_as_union();
    Ok(prepared.on_store(dataset).execute()?)
}

/// Read context registrations and their body targets from the dataset.
fn context_bodies(dataset: &Store) -> LoadResult<BTreeMap<String, BTreeSet<String>>> {
    let query = "
        PREFIX jsonld: <http://www.w3.org/ns/json-ld#>
        PREFIX schema: <https://schema.org/>
        SELECT DISTINCT ?context ?body WHERE {
          ?context a jsonld:Context ;
                   schema:contentUrl ?body .
        }
    ";
    let mut bodies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let QueryResults::Solutions(solutions) = evaluate_union(dataset, query)? else {
        return Err(io::Error::other("context registration query did not return solutions").into());
    };
    for solution in solutions {
        let solution = solution?;
        let Some(Term::NamedNode(context)) = solution.get("context").cloned() else {
            continue;
        };
        let Some(Term::NamedNode(body)) = solution.get("body").cloned() else {
            continue;
        };
        bodies
            .entry(context.as_str().to_owned())
            .or_default()
            .insert(body.as_str().to_owned());
    }
    Ok(bodies)
}

/// Resolve a context registration with exactly one body target.
fn registered_context_body(
    registrations: &BTreeMap<String, BTreeSet<String>>,
    url: &str,
) -> LoadResult<NamedNode> {
    let bodies = registrations.get(url).cloned().unwrap_or_default();
    match bodies.len() {
        0 => Err(io::Error::other(format!("context is not registered: {url}")).into()),
        1 => Ok(NamedNode::new(
            bodies.into_iter().next().expect("one body"),
        )?),
        _ => Err(io::Error::other(format!("context {url} {AMBIGUOUS_TARGETS}")).into()),
    }
}

/// Find source contributions declaring conflicting context body targets.
fn ambiguous_declarations(
    visible: &Store,
    sources: &BTreeSet<PathBuf>,
) -> LoadResult<BTreeMap<PathBuf, String>> {
    let content_url = NamedNodeRef::new_unchecked("https://schema.org/contentUrl");
    let mut errors = BTreeMap::new();
    for (context, bodies) in context_bodies(visible)? {
        if bodies.len() < 2 {
            continue;
        }
        let context = NamedNode::new(context)?;
        let graphs = visible
            .quads_for_pattern(Some(context.as_ref().into()), Some(content_url), None, None)
            .map(|quad| quad.map(|quad| quad.graph_name))
            .collect::<Result<Vec<_>, _>>()?;
        let message = format!("context {context} {AMBIGUOUS_TARGETS}");
        for relative in sources {
            let graph = graph_name(relative)?;
            let mut owned = scoped_graphs_for_source(visible, relative)?;
            owned.insert(graph.clone());
            if !owned
                .iter()
                .any(|graph| graphs.contains(&graph.as_ref().into()))
            {
                continue;
            }
            errors.insert(relative.clone(), message.clone());
        }
    }
    Ok(errors)
}

/// Construct the catalog IRI for a source load error.
fn load_error_name(relative: &Path) -> LoadResult<NamedNode> {
    Ok(NamedNode::new(format!(
        "{}#load-error",
        graph_name(relative)?.as_str()
    ))?)
}

/// Encode a relative path as a sparqld IRI, adding a trailing slash for directories.
fn path_iri(relative: &Path, directory: bool) -> LoadResult<String> {
    let relative = relative.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("path is not valid UTF-8: {}", relative.display()),
        )
    })?;
    let relative = relative.replace(std::path::MAIN_SEPARATOR, "/");
    let mut iri = format!(
        "sparqld:{}",
        utf8_percent_encode(&relative, GRAPH_PATH_ENCODE_SET)
    );
    if directory && !relative.is_empty() {
        iri.push('/');
    }
    Ok(iri)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxigraph::model::{LiteralRef, NamedNodeRef, QuadRef};
    use oxigraph::sparql::{QueryResults, SparqlEvaluator};

    fn write_context_declaration(directory: &Path, url: &str, body_file: &str, body: &[u8]) {
        fs::write(directory.join(body_file), body).unwrap();
        fs::write(
            directory.join("z-declaration.jsonld"),
            serde_json::to_vec(&serde_json::json!({
                "@context": {
                    "jsonld": "http://www.w3.org/ns/json-ld#",
                    "contentUrl": {
                        "@id": "https://schema.org/contentUrl",
                        "@type": "@id"
                    }
                },
                "@id": url,
                "@type": "jsonld:Context",
                "contentUrl": body_file
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn context_dependencies(dataset: &Store) -> Vec<(String, String)> {
        let mut dependencies = dataset
            .quads_for_pattern(
                None,
                Some(JSONLD_CONTEXT),
                None,
                Some(FILE_CATALOG_GRAPH.into()),
            )
            .map(|quad| {
                let quad = quad.unwrap();
                (quad.subject.to_string(), quad.object.to_string())
            })
            .collect::<Vec<_>>();
        dependencies.sort();
        dependencies
    }

    #[test]
    fn records_context_requests_for_successful_and_failed_sources() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("terms.jsonld"),
            r#"{"@context":{"name":"urn:name"}}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/ctx",
            "body.jsonld",
            br#"{"@context":{"name":"urn:name"}}"#,
        );
        for (filename, content) in [
            (
                "success.jsonld",
                serde_json::json!({"@context": ["terms.jsonld", "terms.jsonld"], "@id": "urn:alice", "name": "Alice"}),
            ),
            (
                "remote.jsonld",
                serde_json::json!({"@context": "did:example:context"}),
            ),
            (
                "ordinary.jsonld",
                serde_json::json!({"@context": ["terms.jsonld", {"bad": {"@id": 42}}]}),
            ),
            (
                "registered.jsonld",
                serde_json::json!({"@context": ["https://example.org/ctx", "https://example.org/ctx"]}),
            ),
            (
                "inline.jsonld",
                serde_json::json!({"@context": {"name": "urn:name"}}),
            ),
        ] {
            fs::write(
                directory.path().join(filename),
                serde_json::to_vec(&content).unwrap(),
            )
            .unwrap();
        }
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert_eq!(load.load_errors.len(), 2);
        assert!(load.load_errors[Path::new("ordinary.jsonld")].contains("@id"));
        assert_eq!(
            context_dependencies(&dataset),
            vec![
                (
                    "<sparqld:ordinary.jsonld>".into(),
                    "<sparqld:terms.jsonld>".into()
                ),
                (
                    "<sparqld:registered.jsonld>".into(),
                    "<https://example.org/ctx>".into()
                ),
                (
                    "<sparqld:remote.jsonld>".into(),
                    "<did:example:context>".into()
                ),
                (
                    "<sparqld:success.jsonld>".into(),
                    "<sparqld:terms.jsonld>".into()
                ),
            ]
        );
    }

    #[test]
    fn reloads_failed_sources_after_excluded_context_directory_changes() {
        let directory = tempfile::tempdir().unwrap();
        let contexts = directory.path().join("hidden contexts");
        let terms = contexts.join("name #.yamlld");
        let source = directory.path().join("alice.jsonld");
        let context = graph_name(Path::new("hidden contexts/name #.yamlld")).unwrap();
        fs::write(
            &source,
            serde_json::to_vec(&serde_json::json!({
                "@context": context.as_str(), "@id": "urn:alice", "name": "Alice"
            }))
            .unwrap(),
        )
        .unwrap();
        let patterns = SourcePatterns::compile(&["alice.jsonld".into()]).unwrap();
        let dataset = Store::new().unwrap();
        let mut load = load_directory_with_patterns(directory.path(), &dataset, &patterns).unwrap();
        assert_eq!(load.load_errors.len(), 1);
        let expected = context_dependencies(&dataset);
        for present in [true, false, true] {
            if present {
                fs::create_dir_all(&contexts).unwrap();
                fs::write(&terms, "'@context':\n  name: urn:name\n").unwrap();
            } else {
                fs::remove_dir_all(&contexts).unwrap();
            }
            let report = reload_changed_with_patterns(
                directory.path(),
                &BTreeSet::from([contexts.clone()]),
                &load.loaded_sources,
                &load.load_errors,
                &dataset,
                &patterns,
            )
            .unwrap();
            assert_eq!(
                report.impacts[Path::new("hidden contexts")],
                BTreeSet::from([PathBuf::from("alice.jsonld")])
            );
            assert_eq!(
                report.loaded_sources.contains(Path::new("alice.jsonld")),
                present
            );
            assert_eq!(report.load_errors.is_empty(), present);
            assert_eq!(context_dependencies(&dataset), expected);
            let fresh = Store::new().unwrap();
            load_directory_with_patterns(directory.path(), &fresh, &patterns).unwrap();
            assert_eq!(context_dependencies(&dataset), context_dependencies(&fresh));
            load.loaded_sources = report.loaded_sources;
            load.load_errors = report.load_errors;
        }
    }

    #[test]
    fn ignores_context_edges_outside_the_reserved_catalog() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("source.ttl"),
            "<urn:s> <urn:p> <urn:o> .",
        )
        .unwrap();
        let patterns = SourcePatterns::compile(&["source.ttl".into()]).unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_patterns(directory.path(), &dataset, &patterns).unwrap();
        dataset
            .insert(QuadRef::new(
                NamedNodeRef::new("sparqld:source.ttl").unwrap(),
                JSONLD_CONTEXT,
                NamedNodeRef::new("sparqld:excluded.jsonld").unwrap(),
                NamedNodeRef::new("urn:user-graph").unwrap(),
            ))
            .unwrap();
        let report = reload_changed_with_patterns(
            directory.path(),
            &BTreeSet::from([PathBuf::from("excluded.jsonld")]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
            &patterns,
        )
        .unwrap();
        assert!(report.updates.is_empty());
        assert!(context_dependencies(&dataset).is_empty());
    }

    #[test]
    fn loads_each_source_file_into_a_relative_path_graph() {
        let dataset = Store::new().unwrap();

        let loaded_sources = load_directory(Path::new("docs"), &dataset).unwrap();

        assert!(loaded_sources.contains(Path::new("examples/alpha-centauri.yamlld")));
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("http://dbpedia.org/resource/Proxima_Centauri_b").unwrap(),
                    NamedNodeRef::new("http://dbpedia.org/property/star").unwrap(),
                    NamedNodeRef::new("http://dbpedia.org/resource/Proxima_Centauri").unwrap(),
                    NamedNodeRef::new("sparqld:examples/proxima-centauri-b.jsonld").unwrap(),
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("http://dbpedia.org/resource/Alpha_Centauri").unwrap(),
                    NamedNodeRef::new("http://dbpedia.org/ontology/constellation").unwrap(),
                    NamedNodeRef::new("http://dbpedia.org/resource/Centaurus").unwrap(),
                    NamedNodeRef::new("sparqld:examples/alpha-centauri.yamlld").unwrap(),
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("http://dbpedia.org/resource/Centaurus").unwrap(),
                    NamedNodeRef::new("https://schema.org/subjectOf").unwrap(),
                    NamedNodeRef::new("sparqld:examples/alpha-centauri.yamlld").unwrap(),
                    NamedNodeRef::new("sparqld:examples/centaurus.md").unwrap(),
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("sparqld:project/decisions/implement-sparqld-in.md").unwrap(),
                    NamedNodeRef::new("https://schema.org/name").unwrap(),
                    LiteralRef::new_simple_literal("Implement sparqld in Rust"),
                    NamedNodeRef::new("sparqld:project/decisions/implement-sparqld-in.md").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn scopes_named_graphs_and_rewrites_their_references() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("dataset.trig"),
            "@prefix ex: <urn:> .\nex:graph { ex:head ex:links ex:graph . }",
        )
        .unwrap();
        fs::write(
            directory.path().join("dataset.nq"),
            "<urn:head> <urn:links> <urn:graph> <urn:graph> .",
        )
        .unwrap();
        fs::write(
            directory.path().join("dataset.jsonld"),
            r#"{
                "@context": {"links": {"@id": "urn:links", "@type": "@id"}},
                "@graph": [{
                    "@id": "urn:graph",
                    "@graph": [{"@id": "urn:head", "links": "urn:graph"}]
                }]
            }"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("nanopublication.yamlld"),
            include_str!("loader/fixtures/nanopublication.yamlld"),
        )
        .unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        for source in ["dataset.trig", "dataset.nq", "dataset.jsonld"] {
            let graph = NamedNode::new(format!("sparqld:{source}#urn:graph")).unwrap();
            let source_graph = NamedNode::new(format!("sparqld:{source}")).unwrap();
            assert!(
                dataset
                    .contains(QuadRef::new(
                        NamedNodeRef::new("urn:head").unwrap(),
                        NamedNodeRef::new("urn:links").unwrap(),
                        graph.as_ref(),
                        graph.as_ref(),
                    ))
                    .unwrap()
            );
            assert!(
                dataset
                    .contains(QuadRef::new(
                        source_graph.as_ref(),
                        rdf::TYPE,
                        sd::DATASET,
                        FILE_CATALOG_GRAPH,
                    ))
                    .unwrap()
            );
            let descriptions = dataset
                .quads_for_pattern(
                    Some(source_graph.as_ref().into()),
                    Some(sd::NAMED_GRAPH),
                    None,
                    Some(FILE_CATALOG_GRAPH.into()),
                )
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(descriptions.len(), 1);
            let description = match &descriptions[0].object {
                Term::NamedNode(node) => NamedOrBlankNode::from(node.clone()),
                Term::BlankNode(node) => NamedOrBlankNode::from(node.clone()),
                Term::Literal(_) => panic!("named graph description is a literal"),
            };
            assert!(
                dataset
                    .contains(QuadRef::new(
                        description.as_ref(),
                        rdf::TYPE,
                        sd::NAMED_GRAPH_CLASS,
                        FILE_CATALOG_GRAPH,
                    ))
                    .unwrap()
            );
            assert!(
                dataset
                    .contains(QuadRef::new(
                        description.as_ref(),
                        sd::NAME,
                        graph.as_ref(),
                        FILE_CATALOG_GRAPH,
                    ))
                    .unwrap()
            );
        }

        let nanopublication = NamedNode::new("http://purl.org/nanopub/temp/np/").unwrap();
        let assertion = NamedNode::new(
            "sparqld:nanopublication.yamlld#http://purl.org/nanopub/temp/np/assertion",
        )
        .unwrap();
        let head =
            NamedNode::new("sparqld:nanopublication.yamlld#http://purl.org/nanopub/temp/np/Head")
                .unwrap();
        assert!(
            dataset
                .contains(QuadRef::new(
                    nanopublication.as_ref(),
                    NamedNodeRef::new("http://www.nanopub.org/nschema#hasAssertion").unwrap(),
                    assertion.as_ref(),
                    head.as_ref(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn removes_scoped_graphs_when_a_source_is_deleted() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("dataset.nq");
        fs::write(&source, "<urn:head> <urn:links> <urn:graph> <urn:graph> .").unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        let scoped = NamedNodeRef::new("sparqld:dataset.nq#urn:graph").unwrap();
        assert!(dataset.contains_named_graph(scoped).unwrap());

        fs::remove_file(&source).unwrap();
        reload_changed(
            directory.path(),
            &BTreeSet::from([source]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
        )
        .unwrap();

        assert!(!dataset.contains_named_graph(scoped).unwrap());
    }

    #[test]
    fn replaces_scoped_graphs_when_a_source_changes() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("dataset.nq");
        fs::write(
            &source,
            "<urn:head> <urn:links> <urn:old-graph> <urn:old-graph> .",
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        let old_graph = NamedNodeRef::new("sparqld:dataset.nq#urn:old-graph").unwrap();

        fs::write(
            &source,
            "<urn:head> <urn:links> <urn:new-graph> <urn:new-graph> .",
        )
        .unwrap();
        reload_changed(
            directory.path(),
            &BTreeSet::from([source]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
        )
        .unwrap();

        assert!(!dataset.contains_named_graph(old_graph).unwrap());
        assert!(
            dataset
                .contains_named_graph(
                    NamedNodeRef::new("sparqld:dataset.nq#urn:new-graph").unwrap(),
                )
                .unwrap()
        );
    }

    #[test]
    fn counts_ignored_files_and_loads_context_documents_as_sources() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("context.yamlld"),
            "'@context':\n  value: urn:value\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("source.yamlld"),
            "'@context': context.yamlld\n'@id': urn:subject\nvalue: object\n",
        )
        .unwrap();
        fs::write(directory.path().join("notes.md"), "# No front matter\n").unwrap();
        fs::write(directory.path().join("styles.css"), "body {}\n").unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        assert_eq!(load.loaded_sources.len(), 2);
        assert_eq!(load.ignored_files, 2);
    }

    #[test]
    fn records_a_source_error_and_continues_initial_loading() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("good.ttl"),
            "<urn:subject> <urn:value> <urn:object> .",
        )
        .unwrap();
        fs::write(directory.path().join("broken.xml"), "<RDF></RDF>").unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        let broken = Path::new("broken.xml");
        let error = &load.load_errors[broken];
        let resource = NamedNodeRef::new("sparqld:broken.xml").unwrap();
        let entry = NamedNodeRef::new("sparqld:broken.xml#load-error").unwrap();
        assert_eq!(
            load.loaded_sources,
            BTreeSet::from([PathBuf::from("good.ttl")])
        );
        assert!(
            error.contains("XML namespaces are required in RDF/XML"),
            "unexpected parser error: {error}"
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    entry,
                    rdf::TYPE,
                    rlog::ENTRY,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    entry,
                    rlog::LEVEL,
                    rlog::ERROR,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    entry,
                    rlog::RESOURCE,
                    resource,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    entry,
                    rlog::MESSAGE,
                    LiteralRef::new_simple_literal(error),
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(!dataset.contains_named_graph(resource).unwrap());
    }

    #[test]
    fn resolves_a_remote_context_from_a_later_declaration() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("body.jsonld"),
            r#"{"@context":{"name":"urn:name"}}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("z-declaration.jsonld"),
            serde_json::to_vec(&serde_json::json!({
                "@context": {
                    "jsonld": "http://www.w3.org/ns/json-ld#",
                    "contentUrl": {
                        "@id": "https://schema.org/contentUrl",
                        "@type": "@id"
                    }
                },
                "@id": "https://example.org/ctx",
                "@type": "jsonld:Context",
                "contentUrl": "body.jsonld"
            }))
            .unwrap(),
        )
        .unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(load.loaded_sources.contains(Path::new("alice.jsonld")));
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn resolves_a_yaml_ld_context_body() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/ctx",
            "body.yamlld",
            b"'@context':\n  name: urn:name\n",
        );
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn resolves_a_body_excluded_from_source_patterns() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/ctx",
            "body.jsonld",
            br#"{"@context":{"name":"urn:name"}}"#,
        );
        let patterns =
            SourcePatterns::compile(&["*.jsonld".into(), "!body.jsonld".into()]).unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_patterns(directory.path(), &dataset, &patterns).unwrap();
        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(!load.loaded_sources.contains(Path::new("body.jsonld")));
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn rejects_ambiguous_content_url_targets() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("keep.ttl"),
            "<urn:keep> <urn:value> \"kept\" .",
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/ctx",
            "one.jsonld",
            br#"{"@context":{"name":"urn:name"}}"#,
        );
        fs::write(
            directory.path().join("two.jsonld"),
            br#"{"@context":{"name":"urn:other"}}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("a-declaration.jsonld"),
            serde_json::to_vec(&serde_json::json!({
                "@context": {
                    "jsonld": "http://www.w3.org/ns/json-ld#",
                    "contentUrl": {
                        "@id": "https://schema.org/contentUrl",
                        "@type": "@id"
                    }
                },
                "@id": "https://example.org/ctx",
                "@type": "jsonld:Context",
                "contentUrl": "two.jsonld"
            }))
            .unwrap(),
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        for declaration in ["a-declaration.jsonld", "z-declaration.jsonld"] {
            let error = &load.load_errors[Path::new(declaration)];
            assert!(
                error.contains("multiple contentUrl targets"),
                "{declaration}: {error}"
            );
            assert!(!load.loaded_sources.contains(Path::new(declaration)));
        }
        assert!(load.loaded_sources.contains(Path::new("keep.ttl")));
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:keep").unwrap(),
                    NamedNodeRef::new("urn:value").unwrap(),
                    LiteralRef::new_simple_literal("kept"),
                    NamedNodeRef::new("sparqld:keep.ttl").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn records_a_requested_context_url_without_a_registration() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("a.jsonld"),
            r#"{"@context":"https://example.org/b","@id":"https://example.org/a","@type":"http://www.w3.org/ns/json-ld#Context"}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("b.jsonld"),
            r#"{"@context":"https://example.org/a","@id":"https://example.org/b","@type":"http://www.w3.org/ns/json-ld#Context"}"#,
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert_eq!(load.load_errors.len(), 2);
        assert!(
            load.load_errors
                .values()
                .all(|error| error.contains("context is not registered")),
            "{:?}",
            load.load_errors
        );
        for url in ["https://example.org/a", "https://example.org/b"] {
            assert!(
                dataset
                    .contains(QuadRef::new(
                        NamedNodeRef::new(url).unwrap(),
                        rdf::TYPE,
                        JSONLD_CONTEXT_CLASS,
                        FILE_CATALOG_GRAPH,
                    ))
                    .unwrap()
            );
            assert_eq!(context_bodies(&dataset).unwrap().get(url), None);
        }
    }

    #[test]
    fn resolves_a_json_context_body() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/ctx",
            "body.json",
            br#"{"@context":{"name":"urn:name"}}"#,
        );
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn records_a_malformed_registered_body() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/ctx",
            "body.jsonld",
            b"{",
        );
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        let error = &load.load_errors[Path::new("alice.jsonld")];
        assert!(!error.contains("context is not registered"), "{error}");
        assert!(error.contains("https://example.org/ctx"), "{error}");
    }

    #[test]
    fn accepts_identical_content_url_declarations() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("body.jsonld"),
            r#"{"@context":{"name":"urn:name"}}"#,
        )
        .unwrap();
        let declaration = serde_json::json!({
            "@context": {
                "jsonld": "http://www.w3.org/ns/json-ld#",
                "contentUrl": {
                    "@id": "https://schema.org/contentUrl",
                    "@type": "@id"
                }
            },
            "@id": "https://example.org/ctx",
            "@type": "jsonld:Context",
            "contentUrl": "body.jsonld"
        });
        fs::write(
            directory.path().join("a-declaration.jsonld"),
            serde_json::to_vec(&declaration).unwrap(),
        )
        .unwrap();
        fs::write(
            directory.path().join("b-declaration.jsonld"),
            serde_json::to_vec(&declaration).unwrap(),
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn joins_a_declaration_split_across_graphs() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("body.jsonld"),
            r#"{"@context":{"name":"urn:name"}}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("type.jsonld"),
            r#"{"@id":"https://example.org/ctx","@type":"http://www.w3.org/ns/json-ld#Context"}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("z-url.jsonld"),
            serde_json::to_vec(&serde_json::json!({
                "@context": {
                    "contentUrl": {
                        "@id": "https://schema.org/contentUrl",
                        "@type": "@id"
                    }
                },
                "@id": "https://example.org/ctx",
                "contentUrl": "body.jsonld"
            }))
            .unwrap(),
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn resolves_an_arbitrary_scheme_from_a_registration() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"ipfs://bafyexample/context","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "ipfs://bafyexample/context",
            "body.jsonld",
            br#"{"@context":{"name":"urn:name"}}"#,
        );
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn rejects_a_registered_body_outside_the_served_directory() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("z-declaration.jsonld"),
            serde_json::to_vec(&serde_json::json!({
                "@context": {
                    "jsonld": "http://www.w3.org/ns/json-ld#",
                    "contentUrl": {
                        "@id": "https://schema.org/contentUrl",
                        "@type": "@id"
                    }
                },
                "@id": "https://example.org/ctx",
                "@type": "jsonld:Context",
                "contentUrl": "sparqld:../outside.jsonld"
            }))
            .unwrap(),
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        let error = &load.load_errors[Path::new("alice.jsonld")];
        assert!(error.contains("outside the served directory"), "{error}");
    }

    #[test]
    fn resolves_nested_imports_against_the_requested_context_uri() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/parent","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("parent.jsonld"),
            r#"{"@context":{"@import":"https://example.org/child"}}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("child.jsonld"),
            r#"{"@context":{"name":"urn:name"}}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("parent-declaration.jsonld"),
            serde_json::to_vec(&serde_json::json!({
                "@context": {
                    "jsonld": "http://www.w3.org/ns/json-ld#",
                    "contentUrl": {
                        "@id": "https://schema.org/contentUrl",
                        "@type": "@id"
                    }
                },
                "@id": "https://example.org/parent",
                "@type": "jsonld:Context",
                "contentUrl": "parent.jsonld"
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            directory.path().join("z-child-declaration.jsonld"),
            serde_json::to_vec(&serde_json::json!({
                "@context": {
                    "jsonld": "http://www.w3.org/ns/json-ld#",
                    "contentUrl": {
                        "@id": "https://schema.org/contentUrl",
                        "@type": "@id"
                    }
                },
                "@id": "https://example.org/child",
                "@type": "jsonld:Context",
                "contentUrl": "child.jsonld"
            }))
            .unwrap(),
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn reloads_dependents_when_a_registered_body_changes() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/ctx",
            "body.jsonld",
            br#"{"@context":{"name":"urn:old"}}"#,
        );
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        fs::write(
            directory.path().join("body.jsonld"),
            br#"{"@context":{"name":"urn:new"}}"#,
        )
        .unwrap();
        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([directory.path().join("body.jsonld")]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
        )
        .unwrap();
        assert!(report.loaded_sources.contains(Path::new("alice.jsonld")));
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:new").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn reloads_dependents_when_a_declaration_is_deleted() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("alice.jsonld"),
            r#"{"@context":"https://example.org/ctx","@id":"urn:alice","name":"Alice"}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/ctx",
            "body.jsonld",
            br#"{"@context":{"name":"urn:name"}}"#,
        );
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        fs::remove_file(directory.path().join("z-declaration.jsonld")).unwrap();
        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([directory.path().join("z-declaration.jsonld")]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
        )
        .unwrap();
        let error = &report.load_errors[Path::new("alice.jsonld")];
        assert!(error.contains("context is not registered"), "{error}");
        assert!(
            !dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:alice").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:alice.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn rejects_unregistered_context_urls() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("source.jsonld"),
            r#"{"@context":"https://example.org/context.jsonld","@id":"urn:subject","name":"Alice"}"#,
        )
        .unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        assert!(load.loaded_sources.is_empty());
        let error = &load.load_errors[Path::new("source.jsonld")];
        assert!(
            error.contains("context is not registered"),
            "unexpected error: {error}"
        );
        assert_eq!(
            context_dependencies(&dataset),
            vec![(
                "<sparqld:source.jsonld>".into(),
                "<https://example.org/context.jsonld>".into()
            )]
        );
    }

    #[test]
    fn selects_all_catalogued_failures_for_retry() {
        let directory = tempfile::tempdir().unwrap();
        let dataset = Store::new().unwrap();
        let catalog = FileCatalog::new(&dataset, directory.path()).unwrap();
        for source in ["missing-context.jsonld", "invalid.ttl"] {
            catalog
                .describe_load_error(Path::new(source), "load failed")
                .unwrap();
        }
        assert_eq!(
            retry::failed_files(&dataset).unwrap(),
            BTreeSet::from([
                PathBuf::from("missing-context.jsonld"),
                PathBuf::from("invalid.ttl"),
            ])
        );
    }

    #[test]
    fn separates_one_reload_from_failure_recovery() {
        let directory = Path::new("docs/examples/context-reload/chain");
        let dataset = Store::new().unwrap();
        let patterns = SourcePatterns::all();
        let sources = source_paths(directory, &patterns).unwrap();
        let first = reload_changed_with_patterns(
            directory,
            &sources,
            &BTreeSet::new(),
            &BTreeMap::new(),
            &dataset,
            &patterns,
        )
        .unwrap();
        assert_eq!(
            first.load_errors.keys().cloned().collect::<BTreeSet<_>>(),
            BTreeSet::from([
                PathBuf::from("alice.yamlld"),
                PathBuf::from("second.yamlld"),
            ])
        );

        let recovered = retry::reload_with_retries(
            directory,
            &retry::failed_files(&dataset).unwrap(),
            &first.loaded_sources,
            &first.load_errors,
            &dataset,
            &patterns,
        )
        .unwrap();
        assert_eq!(recovered.loaded_sources, sources);
        assert!(recovered.load_errors.is_empty());
        assert!(retry::failed_files(&dataset).unwrap().is_empty());
    }

    #[test]
    fn queries_context_dependencies_for_failed_sources() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("source.yamlld"),
            include_str!("../docs/examples/missing-context.yamlld"),
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        load_directory_with_stats(directory.path(), &dataset).unwrap();

        let QueryResults::Solutions(solutions) = SparqlEvaluator::new()
            .parse_query(include_str!("../docs/queries/context-dependencies.rq"))
            .unwrap()
            .on_store(&dataset)
            .execute()
            .unwrap()
        else {
            panic!("expected SELECT results");
        };
        let solutions = solutions.collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(solutions.len(), 1);
        assert_eq!(
            solutions[0].get("file"),
            Some(&NamedNode::new("sparqld:source.yamlld").unwrap().into())
        );
        assert_eq!(
            solutions[0].get("context"),
            Some(
                &NamedNode::new("https://example.org/context")
                    .unwrap()
                    .into()
            )
        );
    }

    #[test]
    fn records_every_context_request_in_a_context_array() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("source.jsonld"),
            r#"{"@context":["https://example.org/first","https://example.org/second"]}"#,
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        let error = &load.load_errors[Path::new("source.jsonld")];
        assert_eq!(
            context_dependencies(&dataset),
            vec![
                (
                    "<sparqld:source.jsonld>".into(),
                    "<https://example.org/first>".into()
                ),
                (
                    "<sparqld:source.jsonld>".into(),
                    "<https://example.org/second>".into()
                ),
            ]
        );
        assert!(error.contains("https://example.org/first"));
    }

    #[test]
    fn records_context_requests_alongside_context_failures() {
        for first in ["missing-local.jsonld", "invalid.jsonld"] {
            let directory = tempfile::tempdir().unwrap();
            fs::write(directory.path().join("invalid.jsonld"), "{").unwrap();
            fs::write(
                directory.path().join("source.jsonld"),
                serde_json::to_vec(&serde_json::json!({
                    "@context": [first, "https://example.org/remote"]
                }))
                .unwrap(),
            )
            .unwrap();
            let dataset = Store::new().unwrap();
            let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
            let error = &load.load_errors[Path::new("source.jsonld")];
            let mut expected = vec![
                (
                    "<sparqld:source.jsonld>".into(),
                    format!("<sparqld:{first}>"),
                ),
                (
                    "<sparqld:source.jsonld>".into(),
                    "<https://example.org/remote>".into(),
                ),
            ];
            expected.sort();
            assert_eq!(context_dependencies(&dataset), expected);
            assert!(error.contains(first), "{error}");
        }
    }

    #[test]
    fn records_requests_from_imports_and_scoped_contexts() {
        let contexts = [
            serde_json::json!({"@import": "https://example.org/terms"}),
            serde_json::json!({"child": {"@id": "urn:child", "@context": "https://example.org/terms"}}),
        ];
        for context in contexts {
            let directory = tempfile::tempdir().unwrap();
            fs::write(
                directory.path().join("source.jsonld"),
                serde_json::to_vec(&serde_json::json!({
                    "@context": context,
                    "@id": "urn:subject",
                    "child": {"name": "Alice"}
                }))
                .unwrap(),
            )
            .unwrap();
            let dataset = Store::new().unwrap();
            let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
            assert!(load.load_errors.contains_key(Path::new("source.jsonld")));
            assert_eq!(
                context_dependencies(&dataset),
                vec![(
                    "<sparqld:source.jsonld>".into(),
                    "<https://example.org/terms>".into()
                )],
                "{context}"
            );
        }
    }

    #[test]
    fn catalogs_context_requests_for_arbitrary_uri_schemes() {
        for context in [
            "ipfs://bafyexample/context",
            "ipns://example.org/context",
            "did:example:123",
            "ni:///sha-256;example",
            "https://example.org/context?version=1#terms",
        ] {
            let directory = tempfile::tempdir().unwrap();
            fs::write(
                directory.path().join("source.md"),
                format!("---\n'@context': '{context}'\n---\n# Example\n"),
            )
            .unwrap();
            let dataset = Store::new().unwrap();
            let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
            assert!(load.load_errors.contains_key(Path::new("source.md")));
            assert_eq!(
                context_dependencies(&dataset),
                vec![("<sparqld:source.md>".into(), format!("<{context}>"))]
            );
        }
    }

    #[test]
    fn keeps_local_context_failures_and_source_attempts_distinct() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("a-remote.yamlld"),
            "'@context': https://example.org/context\n",
        )
        .unwrap();
        fs::write(directory.path().join("invalid.jsonld"), "{").unwrap();
        fs::write(directory.path().join("invalid.yamlld"), "'@context': [").unwrap();
        for (name, context) in [
            ("b-missing", "missing.jsonld"),
            ("c-invalid-json", "invalid.jsonld"),
            ("d-invalid-yaml", "invalid.yamlld"),
        ] {
            fs::write(
                directory.path().join(format!("{name}.yamlld")),
                format!("'@context': {context}\n"),
            )
            .unwrap();
        }
        fs::write(directory.path().join("ordinary.jsonld"), "{").unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        assert_eq!(load.load_errors.len(), 7);
        assert_eq!(
            context_dependencies(&dataset),
            vec![
                (
                    "<sparqld:a-remote.yamlld>".into(),
                    "<https://example.org/context>".into()
                ),
                (
                    "<sparqld:b-missing.yamlld>".into(),
                    "<sparqld:missing.jsonld>".into()
                ),
                (
                    "<sparqld:c-invalid-json.yamlld>".into(),
                    "<sparqld:invalid.jsonld>".into()
                ),
                (
                    "<sparqld:d-invalid-yaml.yamlld>".into(),
                    "<sparqld:invalid.yamlld>".into()
                ),
            ]
        );
    }

    #[test]
    fn preserves_context_dependencies_during_unrelated_reloads() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("source.yamlld"),
            include_str!("../docs/examples/missing-context.yamlld"),
        )
        .unwrap();
        let other = directory.path().join("other.ttl");
        fs::write(&other, "<urn:other> <urn:value> <urn:old> .").unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        let before_dependencies = context_dependencies(&dataset);

        fs::write(&other, "<urn:other> <urn:value> <urn:new> .").unwrap();
        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([other]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
        )
        .unwrap();

        assert_eq!(report.updates.len(), 1);
        assert_eq!(report.updates[0].path, Path::new("other.ttl"));
        assert_eq!(context_dependencies(&dataset), before_dependencies);
        assert_eq!(report.load_errors, load.load_errors);
    }

    #[test]
    fn replaces_context_dependencies_and_errors_on_reload() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.jsonld");
        fs::write(&source, r#"{"@context":"https://example.org/old"}"#).unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        let mut errors = load.load_errors;
        let mut sources = load.loaded_sources;

        for content in [
            r#"{"@context":["https://example.org/old","https://example.org/new"]}"#,
            r#"{"@context":"https://example.org/new"}"#,
            "{",
            r#"{"@context":"https://example.org/old"}"#,
            r#"{"@id":"urn:subject","urn:value":"recovered"}"#,
        ] {
            fs::write(&source, content).unwrap();
            let report = reload_changed(
                directory.path(),
                &BTreeSet::from([source.clone()]),
                &sources,
                &errors,
                &dataset,
            )
            .unwrap();
            let fresh = Store::new().unwrap();
            let fresh_load = load_directory_with_stats(directory.path(), &fresh).unwrap();
            assert_eq!(context_dependencies(&dataset), context_dependencies(&fresh));
            assert_eq!(report.load_errors.len(), fresh_load.load_errors.len());
            let entry = NamedNode::new("sparqld:source.jsonld#load-error").unwrap();
            assert_eq!(
                dataset
                    .quads_for_pattern(Some(entry.as_ref().into()), None, None, None)
                    .count(),
                fresh
                    .quads_for_pattern(Some(entry.as_ref().into()), None, None, None)
                    .count()
            );
            errors = report.load_errors;
            sources = report.loaded_sources;
        }
        assert!(errors.is_empty());
        assert!(sources.contains(Path::new("source.jsonld")));
    }

    #[test]
    fn removes_dependencies_and_errors_when_a_failed_source_is_deleted() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.yamlld");
        fs::write(
            &source,
            include_str!("../docs/examples/missing-context.yamlld"),
        )
        .unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        assert_eq!(context_dependencies(&dataset).len(), 1);

        fs::remove_file(&source).unwrap();
        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([source]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
        )
        .unwrap();
        assert!(report.load_errors.is_empty());
        assert!(context_dependencies(&dataset).is_empty());
        assert_eq!(
            dataset
                .quads_for_pattern(
                    Some(
                        NamedNodeRef::new("sparqld:source.yamlld#load-error")
                            .unwrap()
                            .into()
                    ),
                    None,
                    None,
                    None,
                )
                .count(),
            0
        );
    }

    #[test]
    fn catalogs_source_graphs_and_directories() {
        let dataset = Store::new().unwrap();

        load_directory(Path::new("docs"), &dataset).unwrap();

        let decision =
            NamedNodeRef::new("sparqld:project/decisions/implement-sparqld-in.md").unwrap();
        let decisions = NamedNodeRef::new("sparqld:project/decisions/").unwrap();
        let project = NamedNodeRef::new("sparqld:project/").unwrap();
        assert!(
            dataset
                .contains(QuadRef::new(
                    decision,
                    rdf::TYPE,
                    nfo::FILE_DATA_OBJECT,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    decision,
                    nfo::BELONGS_TO_CONTAINER,
                    decisions,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    decisions,
                    rdf::TYPE,
                    nfo::FOLDER,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    decisions,
                    nfo::BELONGS_TO_CONTAINER,
                    project,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    project,
                    rdf::TYPE,
                    nfo::FOLDER,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    project,
                    nfo::BELONGS_TO_CONTAINER,
                    FILE_CATALOG_GRAPH,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
    }

    #[test]
    fn reloads_only_the_changed_source() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first.ttl");
        let second = directory.path().join("second.ttl");
        fs::write(&first, "<urn:first> <urn:value> <urn:old> .").unwrap();
        fs::write(&second, "<urn:second> <urn:value> <urn:preserved> .").unwrap();
        let dataset = Store::new().unwrap();
        let loaded_sources = load_directory(directory.path(), &dataset).unwrap();

        fs::write(&first, "<urn:first> <urn:value> <urn:new> .").unwrap();
        fs::write(&second, "this unrelated source is now invalid").unwrap();
        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([first]),
            &loaded_sources,
            &BTreeMap::new(),
            &dataset,
        )
        .unwrap();

        assert_eq!(report.updates.len(), 1);
        assert_eq!(report.updates[0].kind, SourceUpdateKind::Reloaded);
        assert_eq!(report.updates[0].graph, "sparqld:first.ttl");
        assert_eq!(report.updates[0].triples, 1);
        assert_eq!(report.loaded_sources.len(), 2);
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:first").unwrap(),
                    NamedNodeRef::new("urn:value").unwrap(),
                    NamedNodeRef::new("urn:new").unwrap(),
                    NamedNodeRef::new("sparqld:first.ttl").unwrap(),
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:second").unwrap(),
                    NamedNodeRef::new("urn:value").unwrap(),
                    NamedNodeRef::new("urn:preserved").unwrap(),
                    NamedNodeRef::new("sparqld:second.ttl").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn loads_only_supported_sources_matching_patterns() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir(directory.path().join("archive")).unwrap();
        fs::write(
            directory.path().join("current.ttl"),
            "<urn:current> <urn:value> <urn:included> .",
        )
        .unwrap();
        fs::write(
            directory.path().join("archive/old.ttl"),
            "<urn:archive> <urn:value> <urn:excluded> .",
        )
        .unwrap();
        fs::write(
            directory.path().join("notes.jsonld"),
            r#"{"@id":"urn:notes","urn:value":"ignored"}"#,
        )
        .unwrap();
        let patterns =
            SourcePatterns::compile(&["**/*.ttl".to_owned(), "!archive".to_owned()]).unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_patterns(directory.path(), &dataset, &patterns).unwrap();

        assert_eq!(
            load.loaded_sources,
            BTreeSet::from([PathBuf::from("current.ttl")])
        );
        assert_eq!(load.ignored_files, 2);
        assert!(
            dataset
                .contains_named_graph(NamedNodeRef::new("sparqld:current.ttl").unwrap())
                .unwrap()
        );
        assert!(
            !dataset
                .contains_named_graph(NamedNodeRef::new("sparqld:archive/old.ttl").unwrap())
                .unwrap()
        );
        assert!(
            !dataset
                .contains_named_graph(NamedNodeRef::new("sparqld:notes.jsonld").unwrap())
                .unwrap()
        );
    }

    #[test]
    fn ignores_changes_to_an_excluded_source() {
        let directory = tempfile::tempdir().unwrap();
        let included = directory.path().join("current.ttl");
        let excluded = directory.path().join("archive.ttl");
        fs::write(&included, "<urn:current> <urn:value> <urn:included> .").unwrap();
        fs::write(&excluded, "<urn:archive> <urn:value> <urn:old> .").unwrap();
        let patterns = SourcePatterns::compile(&["!archive.ttl".to_owned()]).unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_patterns(directory.path(), &dataset, &patterns).unwrap();

        fs::write(&excluded, "<urn:archive> <urn:value> <urn:new> .").unwrap();
        let report = reload_changed_with_patterns(
            directory.path(),
            &BTreeSet::from([excluded]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
            &patterns,
        )
        .unwrap();

        assert!(report.updates.is_empty());
        assert_eq!(
            report.loaded_sources,
            BTreeSet::from([PathBuf::from("current.ttl")])
        );
        assert!(
            dataset
                .contains_named_graph(NamedNodeRef::new("sparqld:current.ttl").unwrap())
                .unwrap()
        );
        assert!(
            !dataset
                .contains_named_graph(NamedNodeRef::new("sparqld:archive.ttl").unwrap())
                .unwrap()
        );
    }

    #[test]
    fn records_a_reload_error_and_empties_the_source_graph() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.ttl");
        fs::write(&source, "<urn:subject> <urn:value> <urn:old> .").unwrap();
        let dataset = Store::new().unwrap();
        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        fs::write(&source, "not valid Turtle").unwrap();
        let failed = reload_changed(
            directory.path(),
            &BTreeSet::from([source.clone()]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
        )
        .unwrap();

        assert_eq!(failed.updates.len(), 1);
        assert_eq!(failed.updates[0].kind, SourceUpdateKind::Removed);
        assert_eq!(failed.updates[0].triples, 1);
        assert_eq!(failed.failures.len(), 1);
        assert_eq!(failed.failures[0].path, Path::new("source.ttl"));
        assert!(!failed.loaded_sources.contains(Path::new("source.ttl")));
        assert!(failed.load_errors.contains_key(Path::new("source.ttl")));
        assert!(
            !dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:subject").unwrap(),
                    NamedNodeRef::new("urn:value").unwrap(),
                    NamedNodeRef::new("urn:old").unwrap(),
                    NamedNodeRef::new("sparqld:source.ttl").unwrap(),
                ))
                .unwrap()
        );

        fs::write(&source, "<urn:subject> <urn:value> <urn:new> .").unwrap();
        let recovered = reload_changed(
            directory.path(),
            &BTreeSet::from([source]),
            &failed.loaded_sources,
            &failed.load_errors,
            &dataset,
        )
        .unwrap();

        assert!(recovered.failures.is_empty());
        assert!(recovered.load_errors.is_empty());
        assert_eq!(recovered.updates[0].kind, SourceUpdateKind::Loaded);
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:subject").unwrap(),
                    NamedNodeRef::new("urn:value").unwrap(),
                    NamedNodeRef::new("urn:new").unwrap(),
                    NamedNodeRef::new("sparqld:source.ttl").unwrap(),
                ))
                .unwrap()
        );
        assert!(
            !dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("sparqld:source.ttl#load-error").unwrap(),
                    rdf::TYPE,
                    rlog::ENTRY,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
    }

    #[test]
    fn reports_a_new_sources_graph_and_triple_count() {
        let directory = tempfile::tempdir().unwrap();
        let dataset = Store::new().unwrap();
        let loaded_sources = load_directory(directory.path(), &dataset).unwrap();
        let source = directory.path().join("new.ttl");
        fs::write(
            &source,
            "<urn:one> <urn:value> <urn:first> .\n<urn:two> <urn:value> <urn:second> .",
        )
        .unwrap();

        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([source]),
            &loaded_sources,
            &BTreeMap::new(),
            &dataset,
        )
        .unwrap();

        assert_eq!(report.updates.len(), 1);
        assert_eq!(report.updates[0].kind, SourceUpdateKind::Loaded);
        assert_eq!(report.updates[0].graph, "sparqld:new.ttl");
        assert_eq!(report.updates[0].triples, 2);
    }

    #[test]
    fn removes_deleted_sources_and_orphaned_catalog_directories() {
        let directory = tempfile::tempdir().unwrap();
        let nested = directory.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let source = nested.join("source.ttl");
        fs::write(&source, "<urn:subject> <urn:value> <urn:object> .").unwrap();
        let dataset = Store::new().unwrap();
        let loaded_sources = load_directory(directory.path(), &dataset).unwrap();

        fs::remove_file(&source).unwrap();
        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([source]),
            &loaded_sources,
            &BTreeMap::new(),
            &dataset,
        )
        .unwrap();

        assert_eq!(report.updates.len(), 1);
        assert_eq!(report.updates[0].kind, SourceUpdateKind::Removed);
        assert_eq!(report.updates[0].graph, "sparqld:nested/source.ttl");
        assert_eq!(report.updates[0].triples, 1);
        assert!(report.loaded_sources.is_empty());
        assert!(
            !dataset
                .contains_named_graph(NamedNodeRef::new("sparqld:nested/source.ttl").unwrap())
                .unwrap()
        );
        assert!(
            !dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("sparqld:nested/").unwrap(),
                    rdf::TYPE,
                    nfo::FOLDER,
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );
    }

    #[test]
    fn loads_relative_contexts_and_context_imports() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("context.jsonld"),
            r#"{"@context":{"@version":1.1,"@import":"terms.jsonld"}}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("terms.jsonld"),
            r#"{"@context":{"name":"urn:name"}}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("source.json"),
            r#"{
                "@context": "context.jsonld",
                "@id": "urn:subject",
                "name": "Alice"
            }"#,
        )
        .unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        let graph = NamedNodeRef::new("sparqld:source.json").unwrap();
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:subject").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    graph,
                ))
                .unwrap()
        );
    }

    #[test]
    fn reloads_sources_when_a_declared_context_or_its_import_changes() {
        let directory = tempfile::tempdir().unwrap();
        let contexts = directory.path().join("contexts/astronomy");
        fs::create_dir_all(&contexts).unwrap();
        let terms = contexts.join("terms.yamlld");
        let context = directory.path().join("main.jsonld");
        let source = directory.path().join("source.jsonld");
        fs::write(&terms, "'@context':\n  value: urn:old-predicate\n").unwrap();
        fs::write(
            &context,
            r#"{"@context":{"@version":1.1,"@import":"contexts/astronomy/terms.yamlld"}}"#,
        )
        .unwrap();
        fs::write(
            &source,
            r#"{"@context":"main.jsonld","@id":"urn:subject","value":"object"}"#,
        )
        .unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();
        let terms_relative = PathBuf::from("contexts/astronomy/terms.yamlld");
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("sparqld:source.jsonld").unwrap(),
                    JSONLD_CONTEXT,
                    graph_name(&terms_relative).unwrap().as_ref(),
                    FILE_CATALOG_GRAPH,
                ))
                .unwrap()
        );

        fs::write(&terms, "'@context':\n  value: urn:new-predicate\n").unwrap();
        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([terms]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
        )
        .unwrap();

        assert!(
            report
                .updates
                .iter()
                .any(|update| update.path == Path::new("source.jsonld"))
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:subject").unwrap(),
                    NamedNodeRef::new("urn:new-predicate").unwrap(),
                    LiteralRef::new_simple_literal("object"),
                    NamedNodeRef::new("sparqld:source.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn reloads_an_included_source_when_its_excluded_context_changes() {
        let directory = tempfile::tempdir().unwrap();
        let context = directory.path().join("main.jsonld");
        let source = directory.path().join("source.json");
        fs::write(&context, r#"{"@context":{"value":"urn:old"}}"#).unwrap();
        fs::write(
            &source,
            r#"{"@context":"main.jsonld","@id":"urn:subject","value":"object"}"#,
        )
        .unwrap();
        let patterns =
            SourcePatterns::compile(&["source.json".to_owned(), "!main.jsonld".to_owned()])
                .unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_patterns(directory.path(), &dataset, &patterns).unwrap();
        assert_eq!(
            load.loaded_sources,
            BTreeSet::from([PathBuf::from("source.json")])
        );
        assert!(
            !dataset
                .contains_named_graph(NamedNodeRef::new("sparqld:main.jsonld").unwrap())
                .unwrap()
        );

        fs::write(&context, r#"{"@context":{"value":"urn:new"}}"#).unwrap();
        let report = reload_changed_with_patterns(
            directory.path(),
            &BTreeSet::from([context]),
            &load.loaded_sources,
            &load.load_errors,
            &dataset,
            &patterns,
        )
        .unwrap();

        assert_eq!(report.updates.len(), 1);
        assert_eq!(report.updates[0].path, PathBuf::from("source.json"));
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:subject").unwrap(),
                    NamedNodeRef::new("urn:new").unwrap(),
                    LiteralRef::new_simple_literal("object"),
                    NamedNodeRef::new("sparqld:source.json").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn expands_keyword_aliases_from_a_registered_context() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("source.yamlld"),
            "'@context': https://example.org/aliases\n$id: urn:yaml-subject\n$type: urn:Type\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("source.jsonld"),
            r#"{"@context":"https://example.org/aliases","$id":"urn:json-subject","$type":"urn:Type"}"#,
        )
        .unwrap();
        write_context_declaration(
            directory.path(),
            "https://example.org/aliases",
            "aliases.jsonld",
            br#"{"@context":{"$id":"@id","$type":"@type"}}"#,
        );
        let dataset = Store::new().unwrap();

        let loaded = load_directory(directory.path(), &dataset).unwrap();

        assert_eq!(
            loaded,
            BTreeSet::from([
                PathBuf::from("aliases.jsonld"),
                PathBuf::from("source.jsonld"),
                PathBuf::from("source.yamlld"),
                PathBuf::from("z-declaration.jsonld"),
            ])
        );
        for (subject, graph) in [
            ("urn:yaml-subject", "sparqld:source.yamlld"),
            ("urn:json-subject", "sparqld:source.jsonld"),
        ] {
            assert!(
                dataset
                    .contains(QuadRef::new(
                        NamedNodeRef::new(subject).unwrap(),
                        rdf::TYPE,
                        NamedNodeRef::new("urn:Type").unwrap(),
                        NamedNodeRef::new(graph).unwrap(),
                    ))
                    .unwrap()
            );
        }
    }

    #[test]
    fn does_not_apply_directory_contexts_implicitly() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("context.jsonld"),
            r#"{"@context":{"name":"urn:name"}}"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("source.jsonld"),
            r#"{"@id":"urn:subject","name":"Alice"}"#,
        )
        .unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        assert!(
            load.load_errors.is_empty(),
            "load errors: {:?}",
            load.load_errors
        );
        assert!(
            !dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:subject").unwrap(),
                    NamedNodeRef::new("urn:name").unwrap(),
                    LiteralRef::new_simple_literal("Alice"),
                    NamedNodeRef::new("sparqld:source.jsonld").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn forbids_context_paths_outside_the_served_directory() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("data");
        fs::create_dir(&root).unwrap();
        fs::write(directory.path().join("outside.jsonld"), "{}").unwrap();

        let error = local_context_relative("sparqld:../outside.jsonld").unwrap_err();

        assert!(error.to_string().contains("outside the served directory"));
    }

    #[test]
    fn parses_yaml_with_yaml_1_2_scalar_rules() {
        let json =
            parse_yaml_ld(b"legacy_boolean: yes\nboolean: true\ndate: 2026-07-31\n").unwrap();

        assert_eq!(json["legacy_boolean"], "yes");
        assert_eq!(json["boolean"], true);
        assert_eq!(json["date"], "2026-07-31");
    }

    #[test]
    fn uses_the_containing_directory_as_the_base_iri() {
        assert_eq!(
            directory_iri(Path::new("examples/alpha-centauri.yamlld")).unwrap(),
            "sparqld:examples/"
        );
        assert_eq!(
            directory_iri(Path::new("alpha-centauri.yamlld")).unwrap(),
            "sparqld:"
        );
    }

    #[test]
    fn extracts_yaml_ld_from_markdown_front_matter() {
        assert_eq!(
            markdown_front_matter(b"---\r\n\"@id\": example\r\n---\r\n# Body\r\n").unwrap(),
            Some(b"\"@id\": example\r\n".as_slice())
        );
        assert_eq!(markdown_front_matter(b"# No front matter\n").unwrap(), None);
    }

    #[test]
    fn percent_encodes_graph_names() {
        assert_eq!(
            graph_name(Path::new("examples/alpha centauri.jsonld"))
                .unwrap()
                .as_str(),
            "sparqld:examples/alpha%20centauri.jsonld"
        );
    }
}
