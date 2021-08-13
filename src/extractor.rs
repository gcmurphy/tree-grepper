use crate::language::Language;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Query, QueryCursor};

#[derive(Debug)]
pub struct Extractor {
    language: Language,
    query: Query,
}

impl Extractor {
    pub fn new(language: Language, query: Query) -> Extractor {
        Extractor { language, query }
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn extract_from_file(&self, path: &Path, parser: &mut Parser) -> Result<ExtractedFile> {
        let source =
            fs::read(path).with_context(|| format!("could not read {}", path.display()))?;

        // TODO: do we need to avoid calling self.language.language()
        // repeatedly? Is this something we can move to the init somehow?
        parser
            .set_language(self.language.language())
            .context("could not set language")?;

        let tree = parser
            .parse(&source, None)
            .with_context(|| format!("could not parse {}", path.display()))?;

        let mut cursor = QueryCursor::new();

        Ok(ExtractedFile {
            file: path.to_path_buf(),
            matches: cursor
                .matches(&self.query, tree.root_node(), |node| {
                    node.utf8_text(&source).unwrap_or("")
                })
                .flat_map(|query_match| query_match.captures)
                .map(|capture| {
                    Ok(ExtractedMatch {
                        text: capture
                            .node
                            .utf8_text(&source)
                            .map(|unowned| unowned.to_string())
                            .context("could not extract text from capture")?,
                    })
                })
                .collect::<Result<Vec<ExtractedMatch>>>()?,
        })
    }
}

#[derive(Debug)]
pub struct ExtractedFile {
    file: PathBuf,
    matches: Vec<ExtractedMatch>,
}

#[derive(Debug)]
pub struct ExtractedMatch {
    text: String,
}
