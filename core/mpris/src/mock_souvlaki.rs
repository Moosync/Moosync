pub mod souvlaki {
    use std::time::Duration;

    use std::error::Error;
    use std::fmt;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MediaControlEvent {
        Play, Pause, Toggle, Next, Previous, Stop,
        Seek(MediaPosition), SeekBy(MediaPosition), SetPosition(MediaPosition),
        OpenUri(String), Raise, Quit,
    }

    #[derive(Debug)]
    pub struct MockError(String);

    impl fmt::Display for MockError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for MockError {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct MediaPosition(pub Duration);

    #[derive(Debug, Clone)]
    pub struct PlatformConfig<'a> {
        pub display_name: &'a str,
        pub dbus_name: &'a str,
        pub hwnd: Option<*mut std::ffi::c_void>,
    }

    #[derive(Debug)]
    pub struct MediaControls {}

    impl MediaControls {
        pub fn new(_config: PlatformConfig) -> Result<Self, MockError> {
            Ok(Self {})
        }
        pub fn attach<F>(&mut self, _handler: F) -> Result<(), MockError> where F: Fn(MediaControlEvent) + Send + 'static {
            Ok(())
        }
        pub fn set_metadata(&mut self, _metadata: MediaMetadata) -> Result<(), MockError> {
            Ok(())
        }
        pub fn set_playback(&mut self, _playback: MediaPlayback) -> Result<(), MockError> {
            Ok(())
        }
    }

    pub struct MediaMetadata<'a> {
        pub title: Option<&'a str>,
        pub album: Option<&'a str>,
        pub artist: Option<&'a str>,
        pub cover_url: Option<&'a str>,
        pub duration: Option<Duration>,
    }

    pub enum MediaPlayback {
        Stopped,
        Paused { progress: Option<MediaPosition> },
        Playing { progress: Option<MediaPosition> },
    }
}
