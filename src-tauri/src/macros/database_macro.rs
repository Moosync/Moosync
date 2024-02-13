#[macro_export]
macro_rules! filter_field {
    ($predicate:expr, $field:expr, $column:expr, $inclusive:expr) => {
        if let Some(val) = $field {
            if $inclusive {
                QueryDsl::or_filter($predicate, $column.eq(val))
            } else {
                QueryDsl::filter($predicate, $column.eq(val))
            }
        } else {
            $predicate
        }
    };
}
