use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilePosition {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LambdaMeta {
    #[allow(non_snake_case)]
    pub isPrimop: bool,
    #[allow(non_snake_case)]
    pub isFunctor: Option<bool>,
    pub name: Option<String>,
    pub position: Option<FilePosition>,
    pub args: Option<Vec<String>>,
    pub experimental: Option<bool>,
    pub arity: Option<usize>,
    pub content: Option<String>,
    #[allow(non_snake_case)]
    pub countApplied: Option<usize>,
    // expr is serialized AST, we treat it as string or ignore
    pub expr: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttrMeta {
    pub position: Option<FilePosition>,
    pub content: Option<String>,
    pub expr: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocsMeta {
    pub lambda: Option<LambdaMeta>,
    pub attr: AttrMeta,
}

// Docs corresponds to one item in the root array of data.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocItem {
    pub docs: DocsMeta,
    pub aliases: Option<Vec<Vec<String>>>, // ValuePath is Vec<String>
    pub path: Vec<String>,
}

impl DocItem {
    pub fn title(&self) -> String {
        self.path.join(".")
    }

    pub fn content(&self) -> Option<&String> {
        // Prioritize lambda content, then attr content
        if let Some(lambda) = &self.docs.lambda {
             if let Some(content) = &lambda.content {
                 if !content.is_empty() {
                     return Some(content);
                 }
             }
        }
        if let Some(content) = &self.docs.attr.content {
            if !content.is_empty() {
                return Some(content);
            }
        }
        None
    }
}
