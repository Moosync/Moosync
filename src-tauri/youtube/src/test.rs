use types::providers::generic::Pagination;

use crate::youtube::YoutubeScraper;

#[tokio::test]
async fn test_youtube_playlist_content() {
    let scraper = YoutubeScraper::default();

    let res = scraper
        .get_playlist_content(
            "PLO0BbVUKhzndkgijKVT8QivNgcUTtAjqg".to_string(),
            Pagination::new_limit(50, 0),
        )
        .await
        .unwrap();

    println!("res: {:?}", res);
    assert!(!res.songs.is_empty());
}
