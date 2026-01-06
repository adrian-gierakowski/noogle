#[cfg(test)]
mod tests {
    use crate::model::{DocItem, DocsMeta, AttrMeta, LambdaMeta};
    use crate::search::search;

    fn create_mock_doc(name: &str) -> DocItem {
        DocItem {
            docs: DocsMeta {
                lambda: None,
                attr: AttrMeta {
                    position: None,
                    content: None,
                    expr: None,
                },
            },
            aliases: None,
            path: name.split('.').map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_search_match() {
        let doc1 = create_mock_doc("builtins.map");
        let doc2 = create_mock_doc("lib.lists.map");
        let doc3 = create_mock_doc("builtins.toString");

        let data = vec![doc1.clone(), doc2.clone(), doc3.clone()];

        let results = search("map", &data);

        assert!(results.len() >= 2);
        assert!(results.iter().any(|r| r.doc.title() == "builtins.map"));
        assert!(results.iter().any(|r| r.doc.title() == "lib.lists.map"));
    }

    #[test]
    fn test_search_fuzzy() {
        let doc = create_mock_doc("builtins.readFile");
        let data = vec![doc];

        let results = search("readfile", &data);
        assert!(!results.is_empty());

        let results = search("b.rf", &data); // "b"uiltins."r"ead"f"ile
        assert!(!results.is_empty());
    }
}
