use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilePosition {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrimopMatter {
    pub name: Option<String>,
    pub args: Option<Vec<String>>,
    pub experimental: Option<bool>,
    pub arity: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceOrigin {
    pub position: Option<FilePosition>,
    pub path: Option<Vec<String>>,
    // pos_type omitted as it relies on an enum that might change, and we might not need it
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentSource {
    pub content: Option<String>,
    pub source: Option<SourceOrigin>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentFrontmatter {
    pub title: String,
    pub path: Vec<String>, // Rc<ValuePath> -> Vec<String>
    pub aliases: Option<Vec<Vec<String>>>, // AliasList -> Vec<Vec<String>>
    pub signature: Option<String>,
    pub is_primop: Option<bool>,
    pub primop_meta: Option<PrimopMatter>,
    pub is_functor: Option<bool>,
    pub attr_position: Option<FilePosition>,
    pub attr_expr: Option<String>,
    pub lambda_position: Option<FilePosition>,
    pub lambda_expr: Option<String>,
    pub count_applied: Option<usize>,
    pub content_meta: Option<SourceOrigin>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub meta: DocumentFrontmatter,
    pub content: Option<ContentSource>,
}

// Alias Document to DocItem to minimize changes in other files
pub type DocItem = Document;

impl Document {
    pub fn title(&self) -> &str {
        &self.meta.title
    }

    pub fn content(&self) -> Option<&String> {
        self.content.as_ref().and_then(|c| c.content.as_ref())
    }
}
