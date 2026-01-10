macro_rules! filter_field {
    ($predicate:expr, $field:expr, $column:expr, $inclusive:expr) => {
        if let Some(val) = $field {
            if $inclusive {
                QueryDsl::filter($predicate, $column.eq(val))
            } else {
                QueryDsl::or_filter($predicate, $column.eq(val))
            }
        } else {
            $predicate
        }
    };
}

macro_rules! filter_field_like {
    ($predicate:expr, $field:expr, $column:expr, $inclusive:expr) => {
        if let Some(val) = $field {
            if $inclusive {
                QueryDsl::filter($predicate, $column.like(val))
            } else {
                QueryDsl::or_filter($predicate, $column.like(val))
            }
        } else {
            $predicate
        }
    };
}

pub(crate) use filter_field;
pub(crate) use filter_field_like;
