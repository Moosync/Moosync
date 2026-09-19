use std::{fs, path::PathBuf};

use player_proto::moosync::types::PlayerData;
use prost::Message;

use crate::{context::PersistContext, error::PlayerError};

pub struct FilePersist {
    persist_path: PathBuf,
}

impl FilePersist {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            persist_path: data_dir.join("player_state.pb"),
        }
    }
}

impl PersistContext for FilePersist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn persist(&self, player_data: &PlayerData) -> Result<(), PlayerError> {
        let parent = self
            .persist_path
            .parent()
            .expect("persist path should not be in root");
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        let mut buf = Vec::with_capacity(player_data.encoded_len());
        player_data.encode(&mut buf).unwrap();

        fs::write(&self.persist_path, &buf)?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn load(&self) -> Result<PlayerData, PlayerError> {
        if !self.persist_path.exists() {
            return Ok(PlayerData::default());
        }

        let raw_data = fs::read(&self.persist_path)?;
        Ok(PlayerData::decode(raw_data.as_slice())?)
    }
}
