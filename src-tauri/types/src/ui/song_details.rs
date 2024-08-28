use std::rc::Rc;

#[derive(Default, Clone)]
pub struct SongDetailIcons {
    pub play: Option<Rc<Box<dyn Fn()>>>,
    pub add_to_queue: Option<Rc<Box<dyn Fn()>>>,
    pub random: Option<Rc<Box<dyn Fn()>>>,
    pub add_to_library: Option<Rc<Box<dyn Fn()>>>,
}

#[derive(Default, Clone)]
pub struct DefaultDetails {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
}
