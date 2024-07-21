use crate::youtube::YoutubeScraper;

#[tokio::test]
async fn test_youtube_playlist_content() {
    let scraper = YoutubeScraper::default();

    let res = scraper
        .get_playlist_content("UCsRM0YB_dabtEPGPTKo-gcw".to_string(), None)
        .await
        .unwrap();

    println!(
        "{:?} {}",
        res.songs
            .clone()
            .into_iter()
            .map(|v| v.song.title.clone().unwrap())
            .collect::<Vec<String>>(),
        res.songs.clone().len()
    );

    let mut continuation = res.next_page_token.clone();
    loop {
        let res1 = scraper
            .get_playlist_content("UCsRM0YB_dabtEPGPTKo-gcw".to_string(), continuation.clone())
            .await
            .unwrap();

        continuation.clone_from(&res1.next_page_token);

        println!(
            "{:?}",
            res1.songs
                .into_iter()
                .map(|v| v.song.title.clone().unwrap())
                .collect::<Vec<String>>()
        );

        if res1.next_page_token.is_none() {
            println!("Breaking loop");
            break;
        }
    }
    // println!("res: {:?}", res1);
}
