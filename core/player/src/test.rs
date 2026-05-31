#[cfg(test)]
mod tests {
    use crate::{PlayerHandler, RepeatMode};
    use songs_proto::moosync::types::{Song, InnerSong};
    use std::sync::{Arc, Mutex};

    fn create_mock_song(id: &str, title: &str) -> Song {
        Song {
            song: Some(InnerSong {
                id: Some(id.to_string()),
                title: Some(title.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn create_player_handler() -> PlayerHandler {
        let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
        PlayerHandler::new(ended_tx)
    }

    #[tokio::test]
    async fn test_add_and_current_song() {
        let mut ph = create_player_handler();
        assert!(ph.current_song().is_none());

        let song1 = create_mock_song("1", "Song One");
        ph.add_to_queue(song1.clone());

        assert_eq!(ph.current_song().unwrap().song.as_ref().unwrap().id.as_ref().unwrap(), "1");
        
        let song2 = create_mock_song("2", "Song Two");
        ph.add_to_queue(song2);
        
        assert_eq!(ph.current_song().unwrap().song.as_ref().unwrap().id.as_ref().unwrap(), "1");
    }

    #[tokio::test]
    async fn test_play_now() {
        let mut ph = create_player_handler();
        let song1 = create_mock_song("1", "Song One");
        let song2 = create_mock_song("2", "Song Two");

        ph.add_to_queue(song1);
        ph.play_now(song2);

        assert_eq!(ph.current_song().unwrap().song.as_ref().unwrap().id.as_ref().unwrap(), "2");
        
        assert_eq!(ph.song_queue.len(), 2);
        assert_eq!(ph.song_queue[0].song.as_ref().unwrap().id.as_ref().unwrap(), "2");
        assert_eq!(ph.song_queue[1].song.as_ref().unwrap().id.as_ref().unwrap(), "1");
    }

    #[tokio::test]
    async fn test_next_prev_no_repeat() {
        let mut ph = create_player_handler();
        let song1 = create_mock_song("1", "Song One");
        let song2 = create_mock_song("2", "Song Two");
        let song3 = create_mock_song("3", "Song Three");

        ph.add_to_queue(song1);
        ph.add_to_queue(song2);
        ph.add_to_queue(song3);

        assert_eq!(ph.current_idx, 0);

        ph.next();
        assert_eq!(ph.current_idx, 1);
        assert_eq!(ph.current_song().unwrap().song.as_ref().unwrap().id.as_ref().unwrap(), "2");

        ph.next();
        assert_eq!(ph.current_idx, 2);
        assert_eq!(ph.current_song().unwrap().song.as_ref().unwrap().id.as_ref().unwrap(), "3");

        ph.next();
        assert_eq!(ph.current_idx, 2);

        ph.prev();
        assert_eq!(ph.current_idx, 1);

        ph.prev();
        assert_eq!(ph.current_idx, 0);

        ph.prev();
        assert_eq!(ph.current_idx, 0);
    }

    #[tokio::test]
    async fn test_repeat_infinite() {
        let mut ph = create_player_handler();
        let song1 = create_mock_song("1", "Song One");
        let song2 = create_mock_song("2", "Song Two");

        ph.add_to_queue(song1);
        ph.add_to_queue(song2);
        ph.repeat(RepeatMode::Infinite);

        assert_eq!(ph.current_idx, 0);
        ph.on_song_ended();
        assert_eq!(ph.current_idx, 0);

        ph.next();
        assert_eq!(ph.current_idx, 1);
        ph.on_song_ended();
        assert_eq!(ph.current_idx, 1);
    }

    #[tokio::test]
    async fn test_repeat_once_on_song_ended() {
        let mut ph = create_player_handler();
        let song1 = create_mock_song("1", "Song One");
        let song2 = create_mock_song("2", "Song Two");

        ph.add_to_queue(song1);
        ph.add_to_queue(song2);

        ph.repeat(RepeatMode::Once);
        ph.on_song_ended();
        assert_eq!(ph.current_idx, 0);
        assert_eq!(ph.repeat_mode, RepeatMode::None);

        ph.on_song_ended();
        assert_eq!(ph.current_idx, 1);
    }

    #[tokio::test]
    async fn test_repeat_changed_callback() {
        let mut ph = create_player_handler();
        let repeat_changed_fired = Arc::new(Mutex::new(Option::<RepeatMode>::None));
        
        let rc_clone = repeat_changed_fired.clone();
        ph.on_repeat_changed(move |mode| {
            let mut fired = rc_clone.lock().unwrap();
            *fired = Some(mode);
        });

        ph.repeat(RepeatMode::Once);
        assert_eq!(*repeat_changed_fired.lock().unwrap(), Some(RepeatMode::Once));

        *repeat_changed_fired.lock().unwrap() = None;
        ph.add_to_queue(create_mock_song("1", "Song"));
        ph.on_song_ended();
        assert_eq!(*repeat_changed_fired.lock().unwrap(), Some(RepeatMode::None));
    }

    #[tokio::test]
    async fn test_shuffle() {
        let mut ph = create_player_handler();
        for i in 1..=10 {
            ph.add_to_queue(create_mock_song(&i.to_string(), &format!("Song {}", i)));
        }

        ph.current_idx = 4;
        let current_song_id = ph.current_song().unwrap().song.as_ref().unwrap().id.clone();

        ph.shuffle();

        assert_eq!(ph.current_idx, 4);
        assert_eq!(ph.current_song().unwrap().song.as_ref().unwrap().id, current_song_id);
        assert_eq!(ph.song_queue.len(), 10);
    }

    #[tokio::test]
    async fn test_callbacks() {
        let mut ph = create_player_handler();
        let song_changed_fired = Arc::new(Mutex::new(false));
        let queue_updated_fired = Arc::new(Mutex::new(false));

        let sc_clone = song_changed_fired.clone();
        ph.on_song_changed(move |_song| {
            let mut fired = sc_clone.lock().unwrap();
            *fired = true;
        });

        let call_count = Arc::new(Mutex::new(0));
        let qu_clone = queue_updated_fired.clone();
        let cc_clone = call_count.clone();
        ph.on_queue_updated(move |queue| {
            let mut fired = qu_clone.lock().unwrap();
            *fired = true;
            let mut count = cc_clone.lock().unwrap();
            *count += 1;
            if *count == 1 {
                assert_eq!(queue.len(), 1);
            } else if *count == 2 {
                assert_eq!(queue.len(), 2);
            }
        });

        ph.add_to_queue(create_mock_song("1", "Song One"));
        assert!(*queue_updated_fired.lock().unwrap());

        *song_changed_fired.lock().unwrap() = false;
        ph.play_now(create_mock_song("2", "Song Two"));
        assert!(*song_changed_fired.lock().unwrap());
    }
}
