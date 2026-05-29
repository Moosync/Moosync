use std::{fmt::Display, path::PathBuf};

use songs_proto::moosync::types::Song;
use types::prelude::SongsExt;

use crate::{error::PlayerError, source::ValidSrc};

pub(crate) trait PlayerExt: Send + Sync {
    fn play(&self) -> Result<(), PlayerError>;
    fn pause(&self) -> Result<(), PlayerError>;
    fn stop(&self) -> Result<(), PlayerError>;
    fn set_volume(&self, volume: f32) -> Result<(), PlayerError>;
    fn seek(&self, position: f64) -> Result<(), PlayerError>;
    fn set_src(&self, src: ValidSrc) -> Result<(), PlayerError>;

    fn can_play(&self, src: ValidSrc) -> bool;
}
