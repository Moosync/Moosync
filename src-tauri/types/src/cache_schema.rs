// @generated automatically by Diesel CLI.

diesel::table! {
    cache (id) {
        id -> Nullable<Integer>,
        url -> Text,
        blob -> Binary,
        expires -> BigInt,
    }
}
