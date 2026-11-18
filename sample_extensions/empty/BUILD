load("@extensions_sdk//:defs.bzl", "PROVIDER_SCOPES", "rust_extension")

package(default_visibility = ["//visibility:public"])

rust_extension(
    name = "sample_extension",
    package_name = "rs.sample.empty",
    srcs = ["src/lib.rs"],
    display_name = "Empty Sample Extension (Rust)",
    scopes = [
        {
            "scopes": [
                PROVIDER_SCOPES.ACCOUNTS,
                PROVIDER_SCOPES.SEARCH,
                PROVIDER_SCOPES.PLAYLISTS,
                PROVIDER_SCOPES.PLAYLIST_SONGS,
                PROVIDER_SCOPES.ARTIST_SONGS,
                PROVIDER_SCOPES.ALBUM_SONGS,
                PROVIDER_SCOPES.LYRICS,
            ],
            "justification": "Query metadata and manage playlists and user accounts",
        },
    ],
    version = "1.0.0",
)

alias(
    name = "empty",
    actual = ":sample_extension",
)
