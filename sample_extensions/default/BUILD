load("@extensions_sdk//:defs.bzl", "PROVIDER_SCOPES", "rust_extension")

package(default_visibility = ["//visibility:public"])

rust_extension(
    name = "sample_extension",
    package_name = "rs.sample",
    srcs = ["src/lib.rs"],
    allowed_hosts = [
        {
            "hosts": [
                "api.example.com",
                "*.moosync.app",
            ],
            "justification": "Access remote media streams and synchronize playlists",
        },
    ],
    allowed_paths = [
        {
            "paths": [
                {
                    "source": "/tmp/moosync",
                    "target": "/tmp/sandbox",
                },
                {
                    "source": "/music",
                    "target": "/media/music",
                },
            ],
            "justification": "Access local music files and cache sandbox storage",
        },
    ],
    display_name = "Sample Extension (Rust)",
    scopes = [
        {
            "scopes": [
                PROVIDER_SCOPES.SEARCH,
                PROVIDER_SCOPES.PLAYBACK_DETAILS,
                PROVIDER_SCOPES.SONG_CONTEXT_MENU,
                PROVIDER_SCOPES.PLAYLIST_CONTEXT_MENU,
                PROVIDER_SCOPES.COVER_ART,
            ],
            "justification": "Core playback, search, and context menu integration",
        },
    ],
    version = "1.0.0",
)

alias(
    name = "default",
    actual = ":sample_extension",
)
