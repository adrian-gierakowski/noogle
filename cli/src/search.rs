use crate::model::DocItem;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

pub struct SearchResult<'a> {
    pub doc: &'a DocItem,
    pub score: i64,
}

pub fn search<'a>(query: &str, data: &'a [DocItem]) -> Vec<SearchResult<'a>> {
    let matcher = SkimMatcherV2::default();
    let mut results: Vec<SearchResult> = data
        .iter()
        .filter_map(|doc| {
            let title = doc.title();
            // Match against title
            let score = matcher.fuzzy_match(&title, query);

            if let Some(score) = score {
                 Some(SearchResult { doc, score })
            } else {
                 // Optionally match against content if no title match?
                 // Or just strictly match title for now as typical fuzzy finders do.
                 // The plan mentioned matching content too.

                 // Let's try matching content if title didn't match strongly?
                 // Actually fuzzy matching content is usually too noisy and slow.
                 // Let's stick to title for now as primary.
                 None
            }
        })
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}
