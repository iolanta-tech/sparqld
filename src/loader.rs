use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::{LiteralRef, NamedNode, NamedNodeRef, Quad, QuadRef, vocab::rdf};
use oxigraph::store::Store;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde_json::{Map, Value};

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

#[derive(Clone, Copy)]
enum SourceFormat {
    Rdf(RdfFormat),
    JsonLd,
    YamlLd,
    Markdown,
}

struct ContextResolver<'a> {
    root: &'a Path,
    cache: HashMap<PathBuf, Option<Value>>,
}

struct FileCatalog<'a> {
    dataset: &'a Store,
}

pub(crate) struct ReloadReport {
    pub(crate) loaded_sources: BTreeSet<PathBuf>,
    pub(crate) updates: Vec<SourceUpdate>,
    pub(crate) impacts: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
}

pub(crate) struct DirectoryLoad {
    pub(crate) loaded_sources: BTreeSet<PathBuf>,
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

    fn describe_source(&self, relative: &Path) -> LoadResult<()> {
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
        self.insert_container(graph.as_ref(), container.as_ref())
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

impl<'a> ContextResolver<'a> {
    fn new(root: &'a Path) -> Self {
        Self {
            root,
            cache: HashMap::new(),
        }
    }

    fn context_for(&mut self, directory: &Path) -> LoadResult<Option<Value>> {
        if let Some(context) = self.cache.get(directory) {
            return Ok(context.clone());
        }

        let context = if let Some(file) = local_context_file(directory) {
            Some(read_context(&file)?)
        } else if directory == self.root {
            None
        } else {
            let parent = directory.parent().ok_or_else(|| {
                io::Error::other(format!(
                    "directory is outside the served tree: {}",
                    directory.display()
                ))
            })?;
            self.context_for(parent)?
        };
        self.cache.insert(directory.to_owned(), context.clone());
        Ok(context)
    }
}

#[cfg(test)]
fn load_directory(directory: &Path, dataset: &Store) -> LoadResult<BTreeSet<PathBuf>> {
    Ok(load_directory_with_stats(directory, dataset)?.loaded_sources)
}

pub(crate) fn load_directory_with_stats(
    directory: &Path,
    dataset: &Store,
) -> LoadResult<DirectoryLoad> {
    let root = directory.canonicalize()?;
    let mut files = directory_files(&root)?;
    files.sort();
    let mut contexts = ContextResolver::new(&root);
    let catalog = FileCatalog::new(dataset, &root)?;

    let mut loaded_sources = BTreeSet::new();
    let mut ignored_files = 0;
    for file in &files {
        if is_context_file(file) {
            continue;
        }
        if source_format(file).is_none() {
            ignored_files += 1;
            continue;
        }
        if load_file(&root, file, dataset, &mut contexts)? {
            let relative = file.strip_prefix(&root).map_err(io::Error::other)?;
            catalog.describe_source(relative)?;
            loaded_sources.insert(relative.to_owned());
        } else {
            ignored_files += 1;
        }
    }

    Ok(DirectoryLoad {
        loaded_sources,
        ignored_files,
    })
}

pub(crate) fn reload_changed(
    directory: &Path,
    changed_paths: &BTreeSet<PathBuf>,
    loaded_sources: &BTreeSet<PathBuf>,
    dataset: &Store,
) -> LoadResult<ReloadReport> {
    let root = directory.canonicalize()?;
    let current_sources = source_paths(&root)?;
    let impacts = source_impacts(&root, changed_paths, loaded_sources, &current_sources);
    let affected = impacts.values().flatten().cloned().collect::<BTreeSet<_>>();

    if affected.is_empty() {
        return Ok(ReloadReport {
            loaded_sources: loaded_sources.clone(),
            updates: Vec::new(),
            impacts,
        });
    }

    let staging = Store::new()?;
    let mut contexts = ContextResolver::new(&root);
    let mut staged_sources = BTreeSet::new();
    for relative in &affected {
        if current_sources.contains(relative)
            && load_file(&root, &root.join(relative), &staging, &mut contexts)?
        {
            staged_sources.insert(relative.clone());
        }
    }

    let mut next_sources = loaded_sources.clone();
    for relative in &affected {
        next_sources.remove(relative);
    }
    next_sources.extend(staged_sources.iter().cloned());

    let catalog = build_catalog(&root, &next_sources)?;
    let staged_quads = collect_quads(&staging)?;
    let catalog_quads = collect_quads(&catalog)?;
    let removed_triples = affected
        .iter()
        .filter(|relative| {
            loaded_sources.contains(*relative) && !staged_sources.contains(*relative)
        })
        .map(|relative| {
            let graph = graph_name(relative)?;
            Ok((
                relative.clone(),
                graph_triple_count(dataset, graph.as_ref())?,
            ))
        })
        .collect::<LoadResult<HashMap<_, _>>>()?;
    let mut transaction = dataset.start_transaction()?;
    for relative in &affected {
        transaction.remove_named_graph(graph_name(relative)?.as_ref())?;
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
                triples: graph_triple_count(&staging, graph.as_ref())?,
                kind: if loaded_sources.contains(relative) {
                    SourceUpdateKind::Reloaded
                } else {
                    SourceUpdateKind::Loaded
                },
            });
        } else if loaded_sources.contains(relative) {
            updates.push(SourceUpdate {
                path: relative.clone(),
                graph: graph.as_str().to_owned(),
                triples: removed_triples[relative],
                kind: SourceUpdateKind::Removed,
            });
        }
    }

    Ok(ReloadReport {
        loaded_sources: next_sources,
        updates,
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

fn source_paths(root: &Path) -> LoadResult<BTreeSet<PathBuf>> {
    source_files(root)?
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
) -> BTreeMap<PathBuf, BTreeSet<PathBuf>> {
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
        if is_context_file(changed) {
            let directory = changed.parent().unwrap_or_else(|| Path::new(""));
            affected.extend(
                loaded_sources
                    .union(current_sources)
                    .filter(|source| source.starts_with(directory))
                    .cloned(),
            );
        } else if loaded_sources.contains(changed) || current_sources.contains(changed) {
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
    impacts
}

fn build_catalog(root: &Path, sources: &BTreeSet<PathBuf>) -> LoadResult<Store> {
    let dataset = Store::new()?;
    let catalog = FileCatalog::new(&dataset, root)?;
    for source in sources {
        catalog.describe_source(source)?;
    }
    Ok(dataset)
}

fn collect_quads(dataset: &Store) -> LoadResult<Vec<Quad>> {
    dataset
        .into_iter()
        .collect::<Result<_, _>>()
        .map_err(Into::into)
}

fn source_files(directory: &Path) -> LoadResult<Vec<PathBuf>> {
    Ok(directory_files(directory)?
        .into_iter()
        .filter(|path| !is_context_file(path) && source_format(path).is_some())
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
    contexts: &mut ContextResolver<'_>,
) -> LoadResult<bool> {
    let format = source_format(file).ok_or_else(|| io::Error::other("unknown RDF format"))?;
    let relative = file.strip_prefix(root).map_err(io::Error::other)?;
    let graph = graph_name(relative)?;
    let base_iri = directory_iri(relative)?;
    let source = fs::read(file)?;
    let (format, source) = match format {
        SourceFormat::Rdf(format) => (format, source),
        SourceFormat::JsonLd => (
            json_ld_format(),
            prepare_json_ld(serde_json::from_slice(&source)?, file, contexts)?,
        ),
        SourceFormat::YamlLd => (
            json_ld_format(),
            prepare_json_ld(parse_yaml_ld(&source)?, file, contexts)?,
        ),
        SourceFormat::Markdown => {
            let Some(front_matter) = markdown_front_matter(&source)? else {
                return Ok(false);
            };
            (
                json_ld_format(),
                prepare_json_ld(parse_yaml_ld(front_matter)?, file, contexts)?,
            )
        }
    };
    let parser = RdfParser::from_format(format)
        .with_base_iri(base_iri)?
        .without_named_graphs()
        .with_default_graph(graph);

    dataset.load_from_slice(parser, &source)?;
    Ok(true)
}

fn source_format(path: &Path) -> Option<SourceFormat> {
    let extension = path.extension()?.to_str()?;
    match extension {
        extension if extension.eq_ignore_ascii_case("jsonld") => Some(SourceFormat::JsonLd),
        extension if extension.eq_ignore_ascii_case("yamlld") => Some(SourceFormat::YamlLd),
        extension if extension.eq_ignore_ascii_case("md") => Some(SourceFormat::Markdown),
        extension => RdfFormat::from_extension(extension).map(SourceFormat::Rdf),
    }
}

fn is_context_file(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("context.jsonld" | "context.yamlld")
    )
}

fn local_context_file(directory: &Path) -> Option<PathBuf> {
    let json = directory.join("context.jsonld");
    if json.is_file() {
        return Some(json);
    }
    let yaml = directory.join("context.yamlld");
    yaml.is_file().then_some(yaml)
}

fn read_context(path: &Path) -> LoadResult<Value> {
    let source = fs::read(path)?;
    let document: Value = match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("jsonld") => {
            serde_json::from_slice(&source)?
        }
        Some(extension) if extension.eq_ignore_ascii_case("yamlld") => parse_yaml_ld(&source)?,
        _ => return Err(io::Error::other("unknown context format").into()),
    };
    let Value::Object(mut document) = document else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("context document must be an object: {}", path.display()),
        )
        .into());
    };
    document.remove("@context").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("context document has no @context: {}", path.display()),
        )
        .into()
    })
}

fn prepare_json_ld(
    document: Value,
    file: &Path,
    contexts: &mut ContextResolver<'_>,
) -> LoadResult<Vec<u8>> {
    let context = contexts.context_for(
        file.parent()
            .ok_or_else(|| io::Error::other("source file has no parent directory"))?,
    )?;
    Ok(serde_json::to_vec(&apply_context(document, context))?)
}

fn apply_context(document: Value, inherited: Option<Value>) -> Value {
    let Some(inherited) = inherited else {
        return document;
    };

    match document {
        Value::Object(mut document) => {
            let context = if let Some(local) = document.remove("@context") {
                combine_contexts(inherited, local)
            } else {
                inherited
            };
            document.insert("@context".into(), context);
            Value::Object(document)
        }
        Value::Array(graph) => Value::Object(Map::from_iter([
            ("@context".into(), inherited),
            ("@graph".into(), Value::Array(graph)),
        ])),
        document => document,
    }
}

fn combine_contexts(inherited: Value, local: Value) -> Value {
    let mut contexts = Vec::new();
    append_context(&mut contexts, inherited);
    append_context(&mut contexts, local);
    Value::Array(contexts)
}

fn append_context(contexts: &mut Vec<Value>, context: Value) {
    if let Value::Array(entries) = context {
        contexts.extend(entries);
    } else {
        contexts.push(context);
    }
}

fn json_ld_format() -> RdfFormat {
    RdfFormat::JsonLd {
        profile: oxigraph::io::JsonLdProfileSet::empty(),
    }
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
    use serde_json::json;

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
    fn counts_ignored_files_but_not_dedicated_contexts() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("context.yamlld"),
            "'@context':\n  value: urn:value\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("source.yamlld"),
            "'@id': urn:subject\nvalue: object\n",
        )
        .unwrap();
        fs::write(directory.path().join("notes.md"), "# No front matter\n").unwrap();
        fs::write(directory.path().join("styles.css"), "body {}\n").unwrap();
        let dataset = Store::new().unwrap();

        let load = load_directory_with_stats(directory.path(), &dataset).unwrap();

        assert_eq!(load.loaded_sources.len(), 1);
        assert_eq!(load.ignored_files, 2);
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
    fn reloads_a_contexts_descendant_sources() {
        let directory = tempfile::tempdir().unwrap();
        let nested = directory.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let context = nested.join("context.yamlld");
        let source = nested.join("source.yamlld");
        let unrelated = directory.path().join("unrelated.ttl");
        fs::write(&context, "'@context':\n  value: urn:old-predicate\n").unwrap();
        fs::write(&source, "'@id': urn:subject\nvalue: object\n").unwrap();
        fs::write(&unrelated, "<urn:other> <urn:value> <urn:preserved> .").unwrap();
        let dataset = Store::new().unwrap();
        let loaded_sources = load_directory(directory.path(), &dataset).unwrap();

        fs::write(&context, "'@context':\n  value: urn:new-predicate\n").unwrap();
        fs::write(&unrelated, "this unrelated source is now invalid").unwrap();
        let report = reload_changed(
            directory.path(),
            &BTreeSet::from([context]),
            &loaded_sources,
            &dataset,
        )
        .unwrap();

        assert_eq!(report.updates.len(), 1);
        assert_eq!(report.updates[0].kind, SourceUpdateKind::Reloaded);
        assert_eq!(report.loaded_sources.len(), 2);
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:subject").unwrap(),
                    NamedNodeRef::new("urn:new-predicate").unwrap(),
                    LiteralRef::new_simple_literal("object"),
                    NamedNodeRef::new("sparqld:nested/source.yamlld").unwrap(),
                ))
                .unwrap()
        );
        assert!(
            dataset
                .contains(QuadRef::new(
                    NamedNodeRef::new("urn:other").unwrap(),
                    NamedNodeRef::new("urn:value").unwrap(),
                    NamedNodeRef::new("urn:preserved").unwrap(),
                    NamedNodeRef::new("sparqld:unrelated.ttl").unwrap(),
                ))
                .unwrap()
        );
    }

    #[test]
    fn inherits_the_nearest_directory_context() {
        let root = Path::new("docs").canonicalize().unwrap();
        let mut contexts = ContextResolver::new(&root);

        let context = contexts
            .context_for(&root.join("project/decisions/nested"))
            .unwrap()
            .unwrap();

        assert_eq!(context["title"], "schema:name");
    }

    #[test]
    fn prefers_json_contexts() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("context.yamlld"),
            "\"@context\": {}\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("context.jsonld"),
            "{\"@context\": {}}\n",
        )
        .unwrap();

        assert_eq!(
            local_context_file(directory.path()).unwrap(),
            directory.path().join("context.jsonld")
        );
    }

    #[test]
    fn applies_local_context_after_the_inherited_context() {
        let document = json!({
            "@context": {"name": "urn:local-name"},
            "name": "Alpha Centauri"
        });
        let inherited = json!({"name": "https://schema.org/name"});

        let document = apply_context(document, Some(inherited));

        assert_eq!(
            document["@context"],
            json!([
                {"name": "https://schema.org/name"},
                {"name": "urn:local-name"}
            ])
        );
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
