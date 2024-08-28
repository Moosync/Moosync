use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::errors::MoosyncError;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, Copy, Encode, Decode)]
#[serde(rename_all = "UPPERCASE")]
pub enum PlayerState {
    Playing,
    Paused,
    #[default]
    Stopped,
    Loading,
}

#[derive(Debug)]
pub enum PlayerEvents {
    Play,
    Pause,
    Ended,
    Loading,
    TimeUpdate(f64),
    Error(MoosyncError),
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
