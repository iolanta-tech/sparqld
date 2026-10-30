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

type LoadResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

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

const FILE_CATALOG_GRAPH: NamedNodeRef<'_> = NamedNodeRef::new_unchecked("sparqld:");
const DOLLAR_CONVENIENCE_CONTEXT_URL: &str =
    "https://json-ld.org/contexts/dollar-convenience.jsonld";
const JSONLD_CONTEXT: NamedNodeRef<'_> =
    NamedNodeRef::new_unchecked("http://www.w3.org/ns/json-ld#context");

mod nfo {
    use oxigraph::model::NamedNodeRef;

    macro_rules! term {
        ($local_name:literal) => {
            NamedNodeRef::new_unchecked(concat!(
                "http://www.semanticdesktop.org/ontologies/2007/03/22/nfo#",
                $local_name
            ))
        };
    }

    pub const BELONGS_TO_CONTAINER: NamedNodeRef<'_> = term!("belongsToContainer");
    pub const FILE_DATA_OBJECT: NamedNodeRef<'_> = term!("FileDataObject");
    pub const FILE_NAME: NamedNodeRef<'_> = term!("fileName");
    pub const FOLDER: NamedNodeRef<'_> = term!("Folder");
}

mod rlog {
    use oxigraph::model::NamedNodeRef;

    macro_rules! term {
        ($local_name:literal) => {
            NamedNodeRef::new_unchecked(concat!(
                "http://persistence.uni-leipzig.org/nlp2rdf/ontologies/rlog#",
                $local_name
            ))
        };
    }

    pub const ENTRY: NamedNodeRef<'_> = term!("Entry");
    pub const ERROR: NamedNodeRef<'_> = term!("ERROR");
    pub const LEVEL: NamedNodeRef<'_> = term!("level");
    pub const MESSAGE: NamedNodeRef<'_> = term!("message");
    pub const RESOURCE: NamedNodeRef<'_> = term!("resource");
}

mod sd {
    use oxigraph::model::NamedNodeRef;

    macro_rules! term {
        ($local_name:literal) => {
            NamedNodeRef::new_unchecked(concat!(
                "http://www.w3.org/ns/sparql-service-description#",
                $local_name
            ))
        };
    }

    pub const DATASET: NamedNodeRef<'_> = term!("Dataset");
    pub const NAME: NamedNodeRef<'_> = term!("name");
    pub const NAMED_GRAPH: NamedNodeRef<'_> = term!("namedGraph");
    pub const NAMED_GRAPH_CLASS: NamedNodeRef<'_> = term!("NamedGraph");
}

#[derive(Clone, Copy)]
enum SourceFormat {
    Rdf(RdfFormat),
    JsonLd,
    YamlLd,
    Markdown,
}

struct FileCatalog<'a> {
    dataset: &'a Store,
}

pub(crate) struct ReloadReport {
    pub(crate) loaded_sources: BTreeSet<PathBuf>,
    pub(crate) load_errors: BTreeMap<PathBuf, String>,
    pub(crate) updates: Vec<SourceUpdate>,
    pub(crate) failures: Vec<SourceFailure>,
    pub(crate) impacts: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
}

pub(crate) struct DirectoryLoad {
    pub(crate) loaded_sources: BTreeSet<PathBuf>,
    pub(crate) load_errors: BTreeMap<PathBuf, String>,
    pub(crate) ignored_files: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SourceUpdateKind {
    Loaded,
    Reloaded,
    Removed,
}

#[derive(Debug)]
pub(crate) struct SourceUpdate {
    pub(crate) path: PathBuf,
    pub(crate) graph: String,
    pub(crate) triples: usize,
    pub(crate) kind: SourceUpdateKind,
}

#[derive(Debug)]
pub(crate) struct SourceFailure {
    pub(crate) path: PathBuf,
    pub(crate) graph: String,
    pub(crate) format: &'static str,
    pub(crate) error: String,
}

impl<'a> FileCatalog<'a> {
    fn new(dataset: &'a Store, root: &Path) -> LoadResult<Self> {
        let catalog = Self { dataset };
        catalog.insert_type(FILE_CATALOG_GRAPH, nfo::FILE_DATA_OBJECT)?;
        catalog.insert_type(FILE_CATALOG_GRAPH, nfo::FOLDER)?;
        if let Some(name) = root.file_name().and_then(|name| name.to_str()) {
            catalog.insert_file_name(FILE_CATALOG_GRAPH, name)?;
        }
        Ok(catalog)
    }

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

    fn describe_load_error(&self, relative: &Path, error: &str) -> LoadResult<()> {
        let resource = graph_name(relative)?;
        let entry = NamedNode::new(format!("{}#load-error", resource.as_str()))?;

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

    fn insert_type(&self, resource: NamedNodeRef<'_>, class: NamedNodeRef<'_>) -> LoadResult<()> {
        self.dataset
            .insert(QuadRef::new(resource, rdf::TYPE, class, FILE_CATALOG_GRAPH))?;
        Ok(())
    }

    fn insert_file_name(&self, resource: NamedNodeRef<'_>, name: &str) -> LoadResult<()> {
        self.dataset.insert(QuadRef::new(
            resource,
            nfo::FILE_NAME,
            LiteralRef::new_simple_literal(name),
            FILE_CATALOG_GRAPH,
        ))?;
        Ok(())
    }

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

pub(crate) fn load_directory_with_patterns(
    directory: &Path,
    dataset: &Store,
    source_patterns: &SourcePatterns,
) -> LoadResult<DirectoryLoad> {
    let root = directory.canonicalize()?;
    let files = directory_files(&root)?;
    let mut sources = BTreeSet::new();
    for file in &files {
        let relative = file.strip_prefix(&root).map_err(io::Error::other)?;
        if source_format(file).is_some() && source_patterns.matches(relative) {
            sources.insert(relative.to_owned());
        }
    }
    FileCatalog::new(dataset, &root)?;
    let ignored_files = files.len() - sources.len();
    let mut load = load_sources(&root, sources, dataset)?;
    load.ignored_files += ignored_files;
    for (relative, error) in &load.load_errors {
        let format = source_format(relative)
            .ok_or_else(|| io::Error::other("failed source has no recognized format"))?;
        log::error!(
            "Could not load {} as {}: {error}",
            relative.display(),
            format.name()
        );
    }
    Ok(load)
}

fn load_sources(
    root: &Path,
    sources: impl IntoIterator<Item = PathBuf>,
    dataset: &Store,
) -> LoadResult<DirectoryLoad> {
    let catalog = FileCatalog { dataset };
    let mut loaded_sources = BTreeSet::new();
    let mut load_errors = BTreeMap::new();
    let mut ignored_files = 0;
    for relative in sources {
        let mut context_dependencies = BTreeSet::new();
        let result = load_file(
            root,
            &root.join(&relative),
            dataset,
            &mut context_dependencies,
        );
        catalog.describe_contexts(&relative, &context_dependencies)?;
        match result {
            Ok(Some(embedded_graphs)) => {
                catalog.describe_source(&relative, &embedded_graphs)?;
                loaded_sources.insert(relative);
            }
            Ok(None) => ignored_files += 1,
            Err(error) => {
                dataset.remove_named_graph(graph_name(&relative)?.as_ref())?;
                let error = error.to_string();
                catalog.describe_source(&relative, &BTreeSet::new())?;
                catalog.describe_load_error(&relative, &error)?;
                load_errors.insert(relative, error);
            }
        }
    }
    Ok(DirectoryLoad {
        loaded_sources,
        load_errors,
        ignored_files,
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
    let impacts = source_impacts(
        &root,
        changed_paths,
        &tracked_sources,
        &current_sources,
        dataset,
    )?;
    let affected = impacts.values().flatten().cloned().collect::<BTreeSet<_>>();

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
    let DirectoryLoad {
        loaded_sources: staged_sources,
        load_errors: failed_sources,
        ..
    } = load_sources(
        &root,
        affected.intersection(&current_sources).cloned(),
        &staging,
    )?;

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

    let catalog = build_catalog(&root, &next_sources, &next_errors, |source| {
        if affected.contains(source) {
            scoped_graphs_for_source(&staging, source)
        } else {
            scoped_graphs_for_source(dataset, source)
        }
    })?;
    let affected_graphs = affected
        .iter()
        .map(|source| graph_name(source))
        .collect::<LoadResult<BTreeSet<_>>>()?;
    for previous in dataset.quads_for_pattern(
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
        .into_iter()
        .map(|(path, error)| {
            let format = source_format(&path)
                .ok_or_else(|| io::Error::other("failed source has no recognized format"))?;
            Ok(SourceFailure {
                graph: graph_name(&path)?.as_str().to_owned(),
                path,
                format: format.name(),
                error: error.to_string(),
            })
        })
        .collect::<LoadResult<Vec<_>>>()?;

    Ok(ReloadReport {
        loaded_sources: next_sources,
        load_errors: next_errors,
        updates,
        failures,
        impacts,
    })
}

fn graph_triple_count(dataset: &Store, graph: NamedNodeRef<'_>) -> LoadResult<usize> {
    dataset
        .quads_for_pattern(None, None, None, Some(graph.into()))
        .try_fold(0, |count, quad| {
            quad?;
            Ok(count + 1)
        })
}

fn source_triple_count(dataset: &Store, relative: &Path) -> LoadResult<usize> {
    let graph = graph_name(relative)?;
    let embedded_graphs = scoped_graphs_for_source(dataset, relative)?;
    embedded_graphs.iter().try_fold(
        graph_triple_count(dataset, graph.as_ref())?,
        |count, graph| Ok(count + graph_triple_count(dataset, graph.as_ref())?),
    )
}

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

fn source_impacts(
    root: &Path,
    changed_paths: &BTreeSet<PathBuf>,
    loaded_sources: &BTreeSet<PathBuf>,
    current_sources: &BTreeSet<PathBuf>,
    dataset: &Store,
) -> LoadResult<BTreeMap<PathBuf, BTreeSet<PathBuf>>> {
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
        impacts.insert(changed.to_owned(), affected);
    }
    Ok(impacts)
}

fn build_catalog(
    root: &Path,
    sources: &BTreeSet<PathBuf>,
    load_errors: &BTreeMap<PathBuf, String>,
    mut embedded_graphs: impl FnMut(&Path) -> LoadResult<BTreeSet<NamedNode>>,
) -> LoadResult<Store> {
    let dataset = Store::new()?;
    let catalog = FileCatalog::new(&dataset, root)?;
    for source in sources
        .iter()
        .chain(load_errors.keys())
        .collect::<BTreeSet<_>>()
    {
        catalog.describe_source(source, &embedded_graphs(source)?)?;
    }
    for (source, error) in load_errors {
        catalog.describe_load_error(source, error)?;
    }
    Ok(dataset)
}

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

fn collect_quads(dataset: &Store) -> LoadResult<Vec<Quad>> {
    dataset
        .into_iter()
        .collect::<Result<_, _>>()
        .map_err(Into::into)
}

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

fn directory_files(directory: &Path) -> LoadResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_directory_files(directory, &mut files)?;
    Ok(files)
}

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

fn load_file(
    root: &Path,
    file: &Path,
    dataset: &Store,
    context_dependencies: &mut BTreeSet<NamedNode>,
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
            context_dependencies,
        )?,
        SourceFormat::YamlLd => load_json_ld(
            root,
            &serde_json::to_vec(&parse_yaml_ld(&source)?)?,
            &base_iri,
            graph,
            dataset,
            context_dependencies,
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
                context_dependencies,
            )?
        }
    };
    Ok(Some(embedded_graphs))
}

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
    fn name(self) -> &'static str {
        match self {
            Self::Rdf(format) => format.name(),
            Self::JsonLd => "JSON-LD",
            Self::YamlLd => "YAML-LD",
            Self::Markdown => "Markdown-LD",
        }
    }
}

fn load_json_ld(
    root: &Path,
    source: &[u8],
    base_iri: &str,
    graph: NamedNode,
    dataset: &Store,
    dependencies: &mut BTreeSet<NamedNode>,
) -> LoadResult<BTreeSet<NamedNode>> {
    let root = root.to_owned();
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
                load_context_document(&root, url)
            })
            .collect::<Result<Vec<_>, _>>()?;
        load_scoped_quads(dataset, quads, graph)
    })();
    dependencies.extend(
        loaded_contexts
            .lock()
            .map_err(|error| io::Error::other(error.to_string()))?
            .iter()
            .cloned(),
    );
    result
}

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

fn load_context_document(root: &Path, url: &str) -> LoadResult<LoadedDocument> {
    if url == DOLLAR_CONVENIENCE_CONTEXT_URL {
        return Ok(LoadedDocument {
            url: url.into(),
            content: include_bytes!("loader/contexts/dollar-convenience.jsonld").to_vec(),
            format: context_format(),
        });
    }

    let relative = local_context_relative(url)?;
    let path = local_context_path(root, &relative, url)?;
    let content = match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("jsonld") => fs::read(&path)?,
        Some(extension) if extension.eq_ignore_ascii_case("yamlld") => {
            serde_json::to_vec(&parse_yaml_ld(&fs::read(&path)?)?)?
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

fn context_format() -> RdfFormat {
    RdfFormat::JsonLd {
        profile: JsonLdProfile::Context.into(),
    }
}

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

fn local_context_path(root: &Path, relative: &Path, url: &str) -> LoadResult<PathBuf> {
    let path = root.join(relative).canonicalize()?;
    if !path.starts_with(root) {
        return Err(
            io::Error::other(format!("context is outside the served directory: {url}")).into(),
        );
    }
    Ok(path)
}

fn parse_yaml_ld(source: &[u8]) -> LoadResult<Value> {
    let options = serde_saphyr::options! {
        legacy_octal_numbers: false,
        strict_booleans: true,
    };
    Ok(serde_saphyr::from_slice_with_options(source, options)?)
}

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

fn trim_line_ending(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

fn graph_name(relative: &Path) -> LoadResult<NamedNode> {
    Ok(NamedNode::new(path_iri(relative, false)?)?)
}

fn directory_iri(relative: &Path) -> LoadResult<String> {
    path_iri(relative.parent().unwrap_or_else(|| Path::new("")), true)
}

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
                "bundled.jsonld",
                serde_json::json!({"@context": [DOLLAR_CONVENIENCE_CONTEXT_URL, DOLLAR_CONVENIENCE_CONTEXT_URL]}),
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
                    "<sparqld:bundled.jsonld>".into(),
                    format!("<{DOLLAR_CONVENIENCE_CONTEXT_URL}>")
                ),
                (
                    "<sparqld:ordinary.jsonld>".into(),
                    "<sparqld:terms.jsonld>".into()
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
            error.contains("context URL is not supported"),
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
    fn only_enables_dollar_keywords_with_the_explicit_context_url() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("source.yamlld"),
            format!(
                "'@context': {DOLLAR_CONVENIENCE_CONTEXT_URL}\n$id: urn:yaml-subject\n$type: urn:Type\nurn:value: YAML-LD\n"
            ),
        )
        .unwrap();
        fs::write(
            directory.path().join("source.jsonld"),
            format!(
                r#"{{"@context":"{DOLLAR_CONVENIENCE_CONTEXT_URL}","$id":"urn:json-subject","$type":"urn:Type","urn:value":"JSON-LD"}}"#
            ),
        )
        .unwrap();
        let dataset = Store::new().unwrap();

        let loaded = load_directory(directory.path(), &dataset).unwrap();

        assert_eq!(
            loaded,
            BTreeSet::from([
                PathBuf::from("source.jsonld"),
                PathBuf::from("source.yamlld")
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
