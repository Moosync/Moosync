// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::keys::{
    ActiveThemeId, ArtistSplitter, ArtworkPath, AutoStartup, ClearQueue, ExcludeMusicPaths,
    ExtensionKey, I18nLanguage, JukeboxMode, MinimizeToTray, MusicPaths, PreferenceKey,
    ScanInterval, ScanThreads, ThumbnailPath, VolumePersistMode,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_key_formatting() {
    let key = ExtensionKey {
        package_name: "moosync.spotify".to_string(),
        key: "client_id".to_string(),
    };
    assert_eq!(key.key(), "extensions.moosync.spotify.client_id");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_keys_definitions_and_values() {
    assert_eq!(MusicPaths.key(), "music_paths");
    assert_eq!(ExcludeMusicPaths.key(), "exclude_music_paths");
    assert_eq!(ScanThreads.key(), "scan_threads");
    assert_eq!(ArtistSplitter.key(), "artist_splitter");
    assert_eq!(ScanInterval.key(), "scan_interval");
    assert_eq!(ThumbnailPath.key(), "thumbnail_path");
    assert_eq!(ArtworkPath.key(), "artwork_path");
    assert_eq!(AutoStartup.key(), "auto_startup");
    assert_eq!(MinimizeToTray.key(), "minimize_to_tray");
    assert_eq!(JukeboxMode.key(), "jukebox_mode");
    assert_eq!(ClearQueue.key(), "clear_queue");
    assert_eq!(VolumePersistMode.key(), "volume_persist_mode");
    assert_eq!(I18nLanguage.key(), "i18n_language");
    assert_eq!(ActiveThemeId.key(), "active_theme_id");
}
