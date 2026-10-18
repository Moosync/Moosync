use extensions_proto::moosync::types::*;

use crate::{
    errors::ExtensionError,
    sanitize::{sanitize_account, sanitize_playlist, sanitize_song},
};

pub trait SanitizeCommand {
    fn sanitize(&mut self, package_name: &str) -> Result<(), ExtensionError>;
}

impl SanitizeCommand for MainCommand {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(&mut self, package_name: &str) -> Result<(), ExtensionError> {
        match &mut self.command {
            Some(main_command::Command::GetPreference(req)) => {
                if let Some(ref mut data) = req.data {
                    data.key = format!("{}.{}", package_name, data.key);
                }
            }
            Some(main_command::Command::SetPreference(req)) => {
                if let Some(ref mut data) = req.data {
                    data.key = format!("{}.{}", package_name, data.key);
                }
            }
            Some(main_command::Command::GetSecure(req)) => {
                if let Some(ref mut data) = req.data {
                    data.key = format!("{}.{}", package_name, data.key);
                }
            }
            Some(main_command::Command::SetSecure(req)) => {
                if let Some(ref mut data) = req.data {
                    data.key = format!("{}.{}", package_name, data.key);
                }
            }
            Some(main_command::Command::AddSongs(req)) => {
                for song in &mut req.songs {
                    sanitize_song(song, package_name);
                }
            }
            Some(main_command::Command::RemoveSong(req)) => {
                if let Some(ref mut song) = req.song {
                    sanitize_song(song, package_name);
                }
            }
            Some(main_command::Command::UpdateSong(req)) => {
                if let Some(ref mut song) = req.song {
                    sanitize_song(song, package_name);
                }
            }
            Some(main_command::Command::AddPlaylist(req)) => {
                if let Some(ref mut playlist) = req.playlist {
                    sanitize_playlist(playlist, package_name);
                }
            }
            Some(main_command::Command::AddToPlaylist(req)) => {
                for song in &mut req.songs {
                    sanitize_song(song, package_name);
                }
            }
            Some(main_command::Command::SetAccount(req)) => {
                if let Some(ref mut account) = req.account {
                    sanitize_account(account, package_name);
                }
            }
            Some(main_command::Command::RegisterOauth(_)) => {}
            Some(main_command::Command::OpenExternalUrl(_)) => {}
            _ => {}
        }

        Ok(())
    }
}
