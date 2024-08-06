use crate::youtube::YoutubeScraper;

#[tokio::test]
async fn test_youtube_playlist_content() {
    let scraper = YoutubeScraper::default();

    let res = scraper
        .get_playlist_content("UCsRM0YB_dabtEPGPTKo-gcw".to_string(), None)
        .await
        .unwrap();

    let mut continuation = res.next_page_token.clone();
    loop {
        let res1 = scraper
            .get_playlist_content("UCsRM0YB_dabtEPGPTKo-gcw".to_string(), continuation.clone())
            .await
            .unwrap();

        continuation.clone_from(&res1.next_page_token);

        if res1.next_page_token.is_none() {
            println!("Breaking loop");
            break;
        }
    }
    // println!("res: {:?}", res1);
}
