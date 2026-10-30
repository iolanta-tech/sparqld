//! Retry failed sources through the file-change reload operation.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use oxigraph::model::Term;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;

use crate::patterns::SourcePatterns;

use super::{
    LoadResult, ReloadReport, SourceUpdateKind, evaluate_union, local_context_relative,
    reload_changed_with_patterns, source_failure, source_format,
};

/// Reload changed files, then retry failures while their count decreases.
pub(crate) fn reload_with_retries(
    directory: &Path,
    changed_paths: &BTreeSet<PathBuf>,
    loaded_sources: &BTreeSet<PathBuf>,
    load_errors: &BTreeMap<PathBuf, String>,
    dataset: &Store,
    source_patterns: &SourcePatterns,
) -> LoadResult<ReloadReport> {
    let mut report = reload_changed_with_patterns(
        directory,
        changed_paths,
        loaded_sources,
        load_errors,
        dataset,
        source_patterns,
    )?;
    let mut updates: BTreeMap<_, _> = std::mem::take(&mut report.updates)
        .into_iter()
        .map(|update| (update.path.clone(), update))
        .collect();
    loop {
        let failed = failed_files(dataset)?;
        if failed.is_empty() {
            break;
        }
        let next = reload_changed_with_patterns(
            directory,
            &failed,
            &report.loaded_sources,
            &report.load_errors,
            dataset,
            source_patterns,
        )?;
        updates.extend(
            next.updates
                .into_iter()
                .map(|update| (update.path.clone(), update)),
        );
        for (path, affected) in next.impacts {
            report.impacts.entry(path).or_default().extend(affected);
        }
        report.loaded_sources = next.loaded_sources;
        report.load_errors = next.load_errors;
        if failed_files(dataset)?.len() >= failed.len() {
            break;
        }
    }
    report.updates = updates
        .into_values()
        .filter_map(|mut update| {
            if report.loaded_sources.contains(&update.path) {
                update.kind = if loaded_sources.contains(&update.path) {
                    SourceUpdateKind::Reloaded
                } else {
                    SourceUpdateKind::Loaded
                };
            } else if loaded_sources.contains(&update.path) {
                update.kind = SourceUpdateKind::Removed;
            } else {
                return None;
            }
            Some(update)
        })
        .collect();
    let attempted: BTreeSet<_> = report.impacts.values().flatten().collect();
    report.failures = report
        .load_errors
        .iter()
        .filter(|(path, _)| attempted.contains(path))
        .map(|(path, error)| source_failure(path, error))
        .collect::<LoadResult<_>>()?;
    log_load_errors(directory, &report.load_errors);
    Ok(report)
}

/// Query catalog error entries for failed source paths.
pub(super) fn failed_files(dataset: &Store) -> LoadResult<BTreeSet<PathBuf>> {
    let query = "
        PREFIX rlog: <http://persistence.uni-leipzig.org/nlp2rdf/ontologies/rlog#>
        SELECT DISTINCT ?file WHERE {
          GRAPH <sparqld:> {
            ?error a rlog:Entry ; rlog:level rlog:ERROR ; rlog:resource ?file .
          }
        }
    ";
    let QueryResults::Solutions(solutions) = evaluate_union(dataset, query)? else {
        return Err(io::Error::other("failed-file query did not return solutions").into());
    };
    let mut files = BTreeSet::new();
    for solution in solutions {
        let solution = solution?;
        if let Some(Term::NamedNode(file)) = solution.get("file") {
            files.insert(local_context_relative(file.as_str())?);
        }
    }
    Ok(files)
}

/// Log final source errors with their detected formats.
fn log_load_errors(root: &Path, errors: &BTreeMap<PathBuf, String>) {
    for (relative, error) in errors {
        let format = source_format(&root.join(relative))
            .map(|format| format.name())
            .unwrap_or("source");
        log::error!("Could not load {} as {format}: {error}", relative.display());
    }
}
