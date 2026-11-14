#[derive(Debug, Default, Copy, Clone)]
pub enum VolumeMode {
    #[default]
    Normal,
    PersistSeparate,
    PersistClamp,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum RepeatModes {
    #[default]
    None,
    Once,
    Loop,
}
