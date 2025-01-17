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

use bitcode::{Decode, Encode};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::MoosyncError;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[serde(rename_all = "UPPERCASE")]
pub enum PlayerState {
    Playing,
    Paused,
    #[default]
    Stopped,
    Loading,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerEvents {
    Play,
    Pause,
    Ended,
    Loading,
    TimeUpdate(f64),

    #[serde(
        deserialize_with = "deserialize_moosync_error",
        serialize_with = "serialize_moosync_error"
    )]
    Error(MoosyncError),
}

impl Clone for PlayerEvents {
    fn clone(&self) -> Self {
        match self {
            PlayerEvents::Play => PlayerEvents::Play,
            PlayerEvents::Pause => PlayerEvents::Pause,
            PlayerEvents::Ended => PlayerEvents::Ended,
            PlayerEvents::Loading => PlayerEvents::Loading,
            PlayerEvents::TimeUpdate(time) => PlayerEvents::TimeUpdate(*time),
            PlayerEvents::Error(error) => PlayerEvents::Error(error.to_string().clone().into()),
        }
    }
}

fn serialize_moosync_error<S>(error: &MoosyncError, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&error.to_string())
}

fn deserialize_moosync_error<'de, D>(deserializer: D) -> Result<MoosyncError, D::Error>
where
    D: Deserializer<'de>,
{
    let error_str: String = Deserialize::deserialize(deserializer)?;
    Ok(MoosyncError::from(error_str))
}

#[derive(Debug, Default, Copy, Clone, Encode, Decode)]
pub enum VolumeMode {
    #[default]
    Normal,
    PersistSeparate,
    PersistClamp,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Encode, Decode)]
pub enum RepeatModes {
    #[default]
    None,
    Once,
    Loop,
}
