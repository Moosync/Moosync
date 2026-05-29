pub(crate) trait PageHandler {
    fn initialize(&self);
    fn on_show(&self);
    fn on_hide(&self);
}
