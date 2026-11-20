use songs_proto::moosync::types::{AllAnalytics, all_analytics::SongListenTime};
use tracing::{info, trace};

use crate::{error::DatabaseError, pool::DbPools};

#[derive(Clone)]
pub struct AnalyticsRepository {
    pools: DbPools,
}

impl AnalyticsRepository {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(pools: DbPools) -> Self { Self { pools } }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn increment_play_count(&self, id: &str) -> Result<(), DatabaseError> {
        trace!("Incrementing play count");
        sqlx::query!(
            "INSERT INTO analytics (song_id, play_count, play_time)
             VALUES (?1, 1, 0.0)
             ON CONFLICT(song_id) DO UPDATE SET play_count = play_count + 1",
            id,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;

        info!("Incremented play count");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn increment_play_time(&self, id: &str, duration: f64) -> Result<(), DatabaseError> {
        trace!("Incrementing play time");
        sqlx::query!(
            "INSERT INTO analytics (song_id, play_count, play_time)
             VALUES (?1, 0, ?2)
             ON CONFLICT(song_id) DO UPDATE SET play_time = play_time + excluded.play_time",
            id,
            duration,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;

        info!("Incremented playtime");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_top_listened_songs(&self) -> Result<AllAnalytics, DatabaseError> {
        let rows = sqlx::query!(
            "SELECT song_id, play_time FROM analytics ORDER BY play_time DESC LIMIT 10"
        )
        .fetch_all(&self.pools.read)
        .await
        .map_err(DatabaseError::Query)?;

        let songs = rows
            .into_iter()
            .map(|row| SongListenTime {
                song_id: row.song_id,
                time: row.play_time,
            })
            .collect();

        let total_listen_time: Option<f64> =
            sqlx::query_scalar!("SELECT SUM(play_time) FROM analytics")
                .fetch_one(&self.pools.read)
                .await
                .unwrap_or(Some(0.0));

        Ok(AllAnalytics {
            total_listen_time: total_listen_time.unwrap_or_default(),
            songs,
        })
    }
}
