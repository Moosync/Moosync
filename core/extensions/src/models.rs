use extensions_proto::moosync::types::*;
use songs_proto::moosync::types::{Album, Artist, Genre, Playlist, Song};

use crate::errors::ExtensionError;

pub trait SanitizeCommand {
    fn sanitize(&mut self, package_name: &str) -> Result<(), ExtensionError>;
}

impl SanitizeCommand for MainCommand {
    fn sanitize(&mut self, package_name: &str) -> Result<(), ExtensionError> {
        match &mut self.command {
            Some(main_command::Command::GetPreference(req)) => {
                if let Some(ref mut data) = req.data {
                    data.key = format!("extensions.{}.{}", package_name, data.key);
                }
            }
            Some(main_command::Command::SetPreference(req)) => {
                if let Some(ref mut data) = req.data {
                    data.key = format!("extensions.{}.{}", package_name, data.key);
                }
            }
            Some(main_command::Command::GetSecure(req)) => {
                if let Some(ref mut data) = req.data {
                    data.key = format!("extensions.{}.{}", package_name, data.key);
                }
            }
            Some(main_command::Command::SetSecure(req)) => {
                if let Some(ref mut data) = req.data {
                    data.key = format!("extensions.{}.{}", package_name, data.key);
                }
            }
            Some(main_command::Command::AddSongs(req)) => {
                let prefix = format!("{}:", package_name);
                for song in &mut req.songs {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(main_command::Command::RemoveSong(req)) => {
                let prefix = format!("{}:", package_name);
                if let Some(ref mut song) = req.song {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(main_command::Command::UpdateSong(req)) => {
                let prefix = format!("{}:", package_name);
                if let Some(ref mut song) = req.song {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(main_command::Command::AddPlaylist(req)) => {
                let prefix = format!("{}:", package_name);
                if let Some(ref mut playlist) = req.playlist {
                    sanitize_playlist(&prefix, playlist);
                }
            }
            Some(main_command::Command::AddToPlaylist(req)) => {
                let prefix = format!("{}:", package_name);
                // Note: AddToPlaylistRequest contains a list of songs to add
                for song in &mut req.songs {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(main_command::Command::UpdateAccounts(req)) => {
                // UpdateAccountsRequest has an optional 'account' string field
                req.account = Some(package_name.to_string());
            }
            Some(main_command::Command::RegisterOauth(_)) => {
                // Todo logic
            }
            Some(main_command::Command::OpenExternalUrl(_)) => {
                // Todo logic
            }
            // Handle other variants or ignore them
            _ => {}
        }

        Ok(())
    }
}

impl SanitizeCommand for ExtensionCommandResponse {
    fn sanitize(&mut self, package_name: &str) -> Result<(), ExtensionError> {
        let prefix = format!("{}:", package_name);

        match self.response.as_mut() {
            Some(extension_command_response::Response::GetAccounts(resp)) => {
                for account in &mut resp.accounts {
                    account.package_name = package_name.to_string();
                }
            }
            Some(extension_command_response::Response::RequestedPlaylists(resp)) => {
                for playlist in &mut resp.playlists {
                    sanitize_playlist(&prefix, playlist);
                }
            }
            Some(extension_command_response::Response::RequestedPlaylistSongs(resp)) => {
                for song in &mut resp.songs {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(extension_command_response::Response::RequestedArtistSongs(resp)) => {
                for song in &mut resp.songs {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(extension_command_response::Response::RequestedAlbumSongs(resp)) => {
                for song in &mut resp.songs {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(extension_command_response::Response::RequestedRecommendations(resp)) => {
                for song in &mut resp.songs {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(extension_command_response::Response::RequestedSongFromUrl(resp)) => {
                if let Some(ref mut song) = resp.song {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(extension_command_response::Response::RequestedSongFromId(resp)) => {
                if let Some(ref mut song) = resp.song {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(extension_command_response::Response::RequestedPlaylistFromUrl(resp)) => {
                if let Some(ref mut playlist) = resp.playlist {
                    sanitize_playlist(&prefix, playlist);
                }
                for song in &mut resp.songs {
                    sanitize_song(&prefix, song)?;
                }
            }
            Some(extension_command_response::Response::RequestedSearchResult(resp)) => {
                for song in &mut resp.songs {
                    sanitize_song(&prefix, song)?;
                }
                for playlist in &mut resp.playlists {
                    sanitize_playlist(&prefix, playlist);
                }
                for artist in &mut resp.artists {
                    sanitize_artist(&prefix, artist);
                }
                for album in &mut resp.albums {
                    sanitize_album(&prefix, album);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

pub fn sanitize_album(prefix: &str, album: &mut Album) {
    if let Some(id) = album.album_id.as_mut()
        && !id.starts_with(prefix)
    {
        *id = format!("{}{}", prefix, id);
    }
}

pub fn sanitize_artist(prefix: &str, artist: &mut Artist) {
    if let Some(id) = artist.artist_id.as_mut()
        && !id.starts_with(prefix)
    {
        *id = format!("{}{}", prefix, id);
    }
}

pub fn sanitize_genre(prefix: &str, genre: &mut Genre) {
    if let Some(id) = genre.genre_id.as_mut()
        && !id.starts_with(prefix)
    {
        *id = format!("{}{}", prefix, id);
    }
}

pub fn sanitize_song(prefix: &str, song: &mut Song) -> Result<(), ExtensionError> {
    if let Some(song) = song.song.as_mut() {
        if let Some(id) = song.id.as_mut()
            && !id.starts_with(prefix)
        {
            *id = format!("{}{}", prefix, id);
        }
    } else {
        return Err(ExtensionError::SanitizeError("Song cannot be empty".into()));
    }

    if let Some(album) = song.album.as_mut() {
        sanitize_album(prefix, album);
    }

    song.artists
        .iter_mut()
        .for_each(|a| sanitize_artist(prefix, a));

    song.genre
        .iter_mut()
        .for_each(|a| sanitize_genre(prefix, a));

    Ok(())
}

pub fn sanitize_playlist(prefix: &str, playlist: &mut Playlist) {
    if let Some(playlist_id) = playlist.playlist_id.as_mut()
        && !playlist_id.starts_with(prefix)
    {
        *playlist_id = format!("{}{}", prefix, playlist_id);
    }
}
