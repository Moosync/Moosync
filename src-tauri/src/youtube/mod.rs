use youtube::youtube::YoutubeScraper;

#[tracing::instrument(level = "trace", skip())]
pub fn get_youtube_scraper_state() -> YoutubeScraper {
    YoutubeScraper::default()
}
