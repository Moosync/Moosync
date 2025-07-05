// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use rusty_ytdl::search::SearchType;
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

    tracing::info!("res: {:?}", res);
    assert!(!res.is_empty());
}

#[tokio::test]
async fn test_youtube_suggestions() {
    let scraper = YoutubeScraper::default();
    let res = scraper.get_suggestions().await.unwrap();

    println!("res: {:?}", res);
}

#[tokio::test]
async fn test_playlist_from_url() {
    let scraper = YoutubeScraper::default();
    let res = scraper
        .get_playlist_from_url("https://www.youtube.com/watch?v=GEyBFg3E_dI&list=PLO0BbVUKhzndkgijKVT8QivNgcUTtAjqg&pp=gAQB".into())
        .await
        .unwrap();

    println!("res: {:?}", res);
}
