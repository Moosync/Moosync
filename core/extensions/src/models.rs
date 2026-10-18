use extensions_proto::moosync::types::*;

use crate::{
    errors::ExtensionError,
    sanitize::{sanitize_account, sanitize_playlist, sanitize_song},
};

pub trait SanitizeCommand {
    fn sanitize(&mut self, package_name: &str) -> Result<(), ExtensionError>;
}

#[tracing::instrument(level = "debug", skip_all)]
fn sanitize_preference_key(data: &mut Option<PreferenceData>, package_name: &str) {
    if let Some(data) = data {
        data.key = format!("{}.{}", package_name, data.key);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn sanitize_preference_command(command: &mut main_command::Command, package_name: &str) -> bool {
    match command {
        main_command::Command::GetPreference(req) => {
            sanitize_preference_key(&mut req.data, package_name);
            true
        }
        main_command::Command::SetPreference(req) => {
            sanitize_preference_key(&mut req.data, package_name);
            true
        }
        main_command::Command::GetSecure(req) => {
            sanitize_preference_key(&mut req.data, package_name);
            true
        }
        main_command::Command::SetSecure(req) => {
            sanitize_preference_key(&mut req.data, package_name);
            true
        }
        _ => false,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn sanitize_song_command(command: &mut main_command::Command, package_name: &str) -> bool {
    match command {
        main_command::Command::AddSongs(req) => {
            for song in &mut req.songs {
                sanitize_song(song, package_name);
            }
            true
        }
        main_command::Command::RemoveSong(req) => {
            if let Some(ref mut song) = req.song {
                sanitize_song(song, package_name);
            }
            true
        }
        main_command::Command::UpdateSong(req) => {
            if let Some(ref mut song) = req.song {
                sanitize_song(song, package_name);
            }
            true
        }
        main_command::Command::AddToPlaylist(req) => {
            for song in &mut req.songs {
                sanitize_song(song, package_name);
            }
            true
        }
        _ => false,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn sanitize_entity_command(command: &mut main_command::Command, package_name: &str) {
    match command {
        main_command::Command::AddPlaylist(req) => {
            if let Some(ref mut playlist) = req.playlist {
                sanitize_playlist(playlist, package_name);
            }
        }
        main_command::Command::SetAccount(req) => {
            if let Some(ref mut account) = req.account {
                sanitize_account(account, package_name);
            }
        }
        _ => {}
    }
}

impl SanitizeCommand for MainCommand {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(&mut self, package_name: &str) -> Result<(), ExtensionError> {
        let Some(ref mut command) = self.command else {
            return Ok(());
        };

        if sanitize_preference_command(command, package_name) {
            return Ok(());
        }

        if sanitize_song_command(command, package_name) {
            return Ok(());
        }

        sanitize_entity_command(command, package_name);
        Ok(())
    }
}
