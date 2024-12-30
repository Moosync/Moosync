use crate::themes::ThemeDetails;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeModalState {
    None,
    NewTheme(ThemeDetails),
    ImportTheme,
    DiscoverTheme,
}
