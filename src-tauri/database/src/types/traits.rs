pub trait SearchByTerm {
    fn search_by_term(term: Option<String>) -> Self;
}

pub trait BridgeUtils {
    fn insert_value(entity: String, song: String) -> Self;
}
