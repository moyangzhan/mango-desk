use crate::enums::QueryIntent;

pub fn detect_intent(query: &str) -> QueryIntent {
    let q = query.trim();

    if q.contains('\\') || q.contains('/') {
        return QueryIntent::PathOnly;
    }

    if q.contains('*') || q.contains('.') {
        return QueryIntent::PathOnly;
    }

    let word_count = q.split_whitespace().count();
    if word_count <= 2 {
        return QueryIntent::PathOnly;
    }

    let semantic_keywords = [
        "about", "related", "that", "which", "where", "notes", "document",
    ];

    if semantic_keywords.iter().any(|k| q.contains(k)) {
        return QueryIntent::Hybrid;
    }

    if q.len() > 20 {
        return QueryIntent::SemanticOnly;
    }

    QueryIntent::Hybrid
}