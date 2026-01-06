#[cfg(test)]
mod tests {
    use crate::model::{DocItem, DocumentFrontmatter, Document};
    use crate::search::search;
    use serde_json::from_str;

    fn create_mock_doc(name: &str) -> DocItem {
        DocItem {
            meta: DocumentFrontmatter {
                title: name.to_string(),
                path: name.split('.').map(|s| s.to_string()).collect(),
                aliases: None,
                signature: None,
                is_primop: None,
                primop_meta: None,
                is_functor: None,
                attr_position: None,
                attr_expr: None,
                lambda_position: None,
                lambda_expr: None,
                count_applied: None,
                content_meta: None,
            },
            content: None,
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

    #[test]
    fn test_deserialization() {
        let json_data = r#"
        {
            "meta": {
                "title": "builtins.map",
                "path": ["builtins", "map"],
                "aliases": null,
                "signature": null,
                "is_primop": true,
                "primop_meta": {
                    "name": "map",
                    "args": ["f", "list"],
                    "experimental": false,
                    "arity": 2
                },
                "is_functor": null,
                "attr_position": null,
                "attr_expr": null,
                "lambda_position": null,
                "lambda_expr": null,
                "count_applied": null,
                "content_meta": null
            },
            "content": {
                "content": "Apply function f to each element of list.",
                "source": null
            }
        }
        "#;

        let doc: Document = from_str(json_data).expect("Failed to deserialize Document");
        assert_eq!(doc.title(), "builtins.map");
        assert_eq!(doc.content(), Some(&"Apply function f to each element of list.".to_string()));
        assert_eq!(doc.meta.is_primop, Some(true));
        assert_eq!(doc.meta.primop_meta.unwrap().arity, Some(2));
    }
}
