use std::{rc::Rc, sync::Arc};

#[derive(Default, Clone)]
pub struct SongDetailIcons {
    pub play: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
    pub add_to_queue: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
    pub random: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
    pub add_to_library: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
}

#[derive(Default, Clone)]
pub struct DefaultDetails {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
}
