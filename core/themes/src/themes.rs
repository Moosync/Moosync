use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, RwLock},
};

use fs_extra::dir::CopyOptions;
use futures::StreamExt;
use prost::Message;
use themes_proto::moosync::types::{ThemeDetails, ThemeManifest};
use tokio::{fs::File as AsyncFile, io::AsyncWriteExt};
use types::{
    plugin::{Plugin, PluginContext},
    subscription::SubscriberList,
};
use uuid::Uuid;
use zip_extensions::{zip_extract::zip_extract, zip_writer::zip_create_from_directory};

use crate::{error::ThemesError, remote::RemoteThemes};

pub struct ThemeHolder {
    pub theme_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub remote: RemoteThemes,
    remote_manifests: Arc<RwLock<Vec<ThemeManifest>>>,
    pub on_theme_changed: SubscriberList<ThemeDetails>,
    pub on_themes_updated: SubscriberList<()>,
}

#[plugin_macro::generate]
impl ThemeHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(theme_dir: PathBuf, tmp_dir: PathBuf) -> Self {
        Self {
            theme_dir,
            tmp_dir: tmp_dir.clone(),
            remote: RemoteThemes::new(tmp_dir),
            remote_manifests: Arc::new(RwLock::new(Vec::new())),
            on_theme_changed: SubscriberList::new(),
            on_themes_updated: SubscriberList::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save_theme(&self, theme: ThemeDetails) -> Result<(), ThemesError> {
        let mut theme = theme;
        if theme.id.is_empty() {
            theme.id = Uuid::new_v4().to_string();
        }

        let theme_path = self.theme_dir.join(theme.id.clone());

        if !theme_path.exists() {
            fs::create_dir_all(&theme_path).map_err(ThemesError::Io)?;
        }
        let theme_config = theme_path.join("config.pb");
        fs::write(theme_config, theme.encode_to_vec()).map_err(ThemesError::Io)?;

        self.on_theme_changed.emit(theme);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_theme(&self, id: String) -> Result<(), ThemesError> {
        let theme_path = self.theme_dir.join(id.clone());
        if theme_path.exists() {
            fs::remove_dir_all(&theme_path).map_err(ThemesError::Io)?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_theme(&self, id: String) -> Result<ThemeDetails, ThemesError> {
        if id == "default" {
            return Ok(ThemeDetails::default());
        }
        let root_dir = self.theme_dir.join(id.clone());
        let theme_config = root_dir.join("config.pb");
        if theme_config.exists() {
            let data = fs::read(theme_config).map_err(ThemesError::Io)?;
            return Ok(ThemeDetails::decode(&data[..])?);
        }

        Err(ThemesError::ThemeNotFound)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_all_themes(&self) -> Result<HashMap<String, ThemeDetails>, ThemesError> {
        let theme_dir = self.theme_dir.clone();
        if !theme_dir.exists() {
            fs::create_dir_all(&theme_dir).map_err(ThemesError::Io)?;
        }
        let entries = fs::read_dir(theme_dir).map_err(ThemesError::Io)?;
        let mut ret = HashMap::new();
        ret.insert("default".into(), ThemeDetails::default());
        for theme_dir in entries.flatten() {
            if theme_dir.path().is_dir() {
                let id = theme_dir.file_name().to_str().unwrap().to_string();
                match self.load_theme(id.clone()) {
                    Ok(theme) => {
                        ret.insert(id, theme);
                    }
                    Err(e) => tracing::error!("Failed to load theme {}: {:?}", id, e),
                }
            }
        }

        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn import_theme(&self, theme_path: String) -> Result<(), ThemesError> {
        let extract_dir = self
            .tmp_dir
            .join(format!("moosync_theme_{}", uuid::Uuid::new_v4()));

        let theme_path = PathBuf::from_str(&theme_path).unwrap();
        zip_extract(&theme_path, &extract_dir.clone())
            .map_err(|e| ThemesError::Zip(format!("{:?}", e)))?;

        for item in extract_dir.read_dir().map_err(ThemesError::Io)?.flatten() {
            let item = item.path();
            if item.is_file() && item.file_name().unwrap().to_string_lossy() == "config.pb" {
                let config = fs::read(item).map_err(ThemesError::Io)?;
                let parsed = ThemeDetails::decode(config.as_slice())?;
                let final_theme_path = self.theme_dir.join(&parsed.id);
                let options = CopyOptions::default().overwrite(true);

                fs::create_dir_all(final_theme_path.clone()).map_err(ThemesError::Io)?;

                let mut item_list = vec![];
                for items in extract_dir.read_dir().map_err(ThemesError::Io)? {
                    item_list.push(items.unwrap().path());
                }
                tracing::info!("Moving from {:?} to {:?}", item_list, final_theme_path);
                fs_extra::move_items(item_list.as_slice(), final_theme_path, &options)
                    .map_err(ThemesError::FsExtra)?;

                self.on_theme_changed.emit(parsed);
                self.on_themes_updated.emit(());

                return Ok(());
            }
        }
        Err(ThemesError::ParseThemeFailed)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn export_theme(&self, id: String, export_path: PathBuf) -> Result<(), ThemesError> {
        let mut export_path = export_path.clone();
        export_path.set_extension("mstx");

        let theme = self.load_theme(id.clone())?;
        let theme_dir = self.tmp_dir.join(format!("theme-unpacked-{}", id));
        if !theme_dir.exists() {
            fs::create_dir_all(theme_dir.clone()).map_err(ThemesError::Io)?;
        }

        let config_path = theme_dir.clone().join("config.pb");
        fs::write(config_path.clone(), theme.encode_to_vec()).map_err(ThemesError::Io)?;

        zip_create_from_directory(&export_path, &theme_dir)
            .map_err(|e| ThemesError::Zip(format!("{:?}", e)))?;

        fs::remove_file(config_path).map_err(ThemesError::Io)?;
        fs::remove_dir(theme_dir).map_err(ThemesError::Io)?;

        Ok(())
    }

    // TODO: Validate URL somehow
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn download_theme(&self, url: String) -> Result<(), ThemesError> {
        let file_path = self.tmp_dir.join(format!("{}.mstx", uuid::Uuid::new_v4()));

        let mut stream = reqwest::get(url)
            .await
            .map_err(ThemesError::Network)?
            .bytes_stream();
        let mut file = AsyncFile::create(file_path.clone())
            .await
            .map_err(ThemesError::Io)?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(ThemesError::Network)?;
            file.write_all(&chunk).await.map_err(ThemesError::Io)?;
        }

        tracing::info!("Wrote file");

        self.import_theme(file_path.to_string_lossy().to_string())?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn fetch_remote_manifests(
        &self,
        registries: HashSet<String>,
    ) -> Result<(), ThemesError> {
        let manifests = self.remote.get_theme_manifests(&registries).await?;
        self.set_remote_manifests(manifests);
        self.on_themes_updated.emit(());
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_remote_manifests(&self, manifests: Vec<ThemeManifest>) {
        if let Ok(mut lock) = self.remote_manifests.write() {
            *lock = manifests;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_remote_manifests(&self) -> Vec<ThemeManifest> {
        self.remote_manifests
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn install_remote_theme(&self, manifest: &ThemeManifest) -> Result<(), ThemesError> {
        let file_path = self.remote.download_theme_file(manifest).await?;
        self.import_theme(file_path.to_string_lossy().to_string())?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_themes_manifest(&self) -> Result<Vec<ThemeManifest>, ThemesError> {
        Ok(self.get_remote_manifests())
    }
}

impl Plugin for ThemeHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(context: &PluginContext) -> Arc<Self> {
        Arc::new(ThemeHolder::new(
            context.data_dir.join("themes"),
            context.tmp_dir.clone(),
        ))
    }
}
