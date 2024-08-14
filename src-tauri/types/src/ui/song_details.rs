use std::rc::Rc;


#[derive(Default, Clone)]
pub struct SongDetailIcons {
    pub play: Option<Rc<Box<dyn Fn()>>>,
    pub add_to_queue: Option<Rc<Box<dyn Fn()>>>,
    pub random: Option<Rc<Box<dyn Fn()>>>,
    pub add_to_library: Option<Rc<Box<dyn Fn()>>>,
}
