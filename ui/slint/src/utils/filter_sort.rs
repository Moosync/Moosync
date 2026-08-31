use std::{cmp::Ordering, path::PathBuf};

use slint::{Model, ModelRc};

use super::lazy_model::LazySongVecModel;
use crate::{SongModel, SongSortCriterion};

#[tracing::instrument(level = "debug", skip_all)]
fn song_matches_query(song: &SongModel, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    song.title.to_lowercase().contains(&query.to_lowercase())
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_primary_artist_title(song: &SongModel) -> String {
    song.artists
        .row_data(0)
        .map(|a| a.title.to_string())
        .unwrap_or_default()
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_song_date_field(song: &SongModel) -> &str {
    if !song.year.is_empty() {
        return &song.year;
    }
    &song.date
}

#[tracing::instrument(level = "debug", skip_all)]
fn compare_song_dates(a: &SongModel, b: &SongModel) -> Ordering {
    let a_date = get_song_date_field(a);
    let b_date = get_song_date_field(b);
    if !a_date.is_empty() || !b_date.is_empty() {
        return a_date.cmp(b_date);
    }
    a.date_added.cmp(&b.date_added)
}

#[tracing::instrument(level = "debug", skip_all)]
fn compare_songs(
    a: &SongModel,
    b: &SongModel,
    criterion: SongSortCriterion,
    ascending: bool,
) -> Ordering {
    let ordering = match criterion {
        SongSortCriterion::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
        SongSortCriterion::Date => compare_song_dates(a, b),
        SongSortCriterion::Album => a
            .album_name
            .to_lowercase()
            .cmp(&b.album_name.to_lowercase()),
        SongSortCriterion::TrackNumber => a
            .track_no
            .partial_cmp(&b.track_no)
            .unwrap_or(Ordering::Equal),
        SongSortCriterion::Artist => get_primary_artist_title(a)
            .to_lowercase()
            .cmp(&get_primary_artist_title(b).to_lowercase()),
    };

    if !ascending {
        return ordering.reverse();
    }
    ordering
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn filter_and_sort_songs(
    songs: ModelRc<SongModel>,
    query: &str,
    criterion: SongSortCriterion,
    ascending: bool,
    item_height: usize,
    item_width: usize,
    cache_dir: PathBuf,
) -> ModelRc<SongModel> {
    let query = query.trim().to_lowercase();
    if query.is_empty() && criterion == SongSortCriterion::Title && ascending {
        return songs;
    }

    let mut filtered: Vec<SongModel> = (0..songs.row_count())
        .filter_map(|i| songs.row_data(i))
        .filter(|song| song_matches_query(song, &query))
        .collect();

    filtered.sort_by(|a, b| compare_songs(a, b, criterion, ascending));

    ModelRc::new(LazySongVecModel::new(
        filtered,
        item_height,
        item_width,
        cache_dir,
    ))
}
