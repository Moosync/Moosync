#[derive(Debug, Default, PartialEq, Clone, Copy)]
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
}
