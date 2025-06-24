# Moods System

This document outlines the mood-based music discovery and playlist generation system for Moosync.

## Overview

The Moods system provides intelligent music recommendations and automatic playlist generation based on user emotions, activities, time of day, weather, and listening patterns. It uses a combination of audio feature analysis, machine learning, and user feedback to create personalized music experiences.

## Core Components

### 1. Mood Detection

#### Audio Feature Analysis:
```rust
struct AudioFeatures {
    // Energy and Dynamics
    energy: f32,           // 0.0 - 1.0 (low to high energy)
    valence: f32,          // 0.0 - 1.0 (negative to positive)
    danceability: f32,     // 0.0 - 1.0 (not danceable to very danceable)
    
    // Musical Elements
    tempo: f32,            // BPM (beats per minute)
    loudness: f32,         // dB (relative loudness)
    acousticness: f32,     // 0.0 - 1.0 (not acoustic to very acoustic)
    instrumentalness: f32, // 0.0 - 1.0 (vocal to instrumental)
    liveness: f32,         // 0.0 - 1.0 (studio to live performance)
    speechiness: f32,      // 0.0 - 1.0 (music to speech-like)
    
    // Harmony and Timbre
    key: i32,              // Musical key (0 = C, 1 = C#, etc.)
    mode: i32,             // 0 = minor, 1 = major
    time_signature: i32,   // Time signature (e.g., 4 for 4/4)
}

impl AudioFeatures {
    pub fn analyze_file(path: &Path) -> Result<Self> {
        // Use librosa-like analysis or external service
        // Extract features using audio analysis libraries
    }
    
    pub fn calculate_mood_vector(&self) -> MoodVector {
        MoodVector {
            energy_level: self.energy * 0.7 + self.loudness.normalize() * 0.3,
            emotional_valence: self.valence,
            rhythmic_intensity: self.danceability * 0.6 + (self.tempo / 200.0) * 0.4,
            musical_complexity: 1.0 - self.acousticness,
        }
    }
}
```

### 2. Mood Categories

#### Predefined Mood Categories:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoodCategory {
    // Energy-based
    Energetic,
    Calm,
    Intense,
    Peaceful,
    
    // Emotion-based
    Happy,
    Sad,
    Angry,
    Romantic,
    Nostalgic,
    Confident,
    
    // Activity-based
    Workout,
    Study,
    Sleep,
    Party,
    Driving,
    Meditation,
    
    // Time-based
    Morning,
    Afternoon,
    Evening,
    Night,
    
    // Weather-based
    Sunny,
    Rainy,
    Cloudy,
    Stormy,
    
    // Custom user-defined
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct MoodVector {
    pub energy_level: f32,      // 0.0 (low) to 1.0 (high)
    pub emotional_valence: f32, // 0.0 (negative) to 1.0 (positive)
    pub rhythmic_intensity: f32,// 0.0 (slow/weak) to 1.0 (fast/strong)
    pub musical_complexity: f32,// 0.0 (simple) to 1.0 (complex)
}

impl MoodCategory {
    pub fn to_vector(&self) -> MoodVector {
        match self {
            MoodCategory::Energetic => MoodVector {
                energy_level: 0.9,
                emotional_valence: 0.8,
                rhythmic_intensity: 0.8,
                musical_complexity: 0.6,
            },
            MoodCategory::Calm => MoodVector {
                energy_level: 0.2,
                emotional_valence: 0.6,
                rhythmic_intensity: 0.3,
                musical_complexity: 0.4,
            },
            MoodCategory::Workout => MoodVector {
                energy_level: 0.95,
                emotional_valence: 0.7,
                rhythmic_intensity: 0.9,
                musical_complexity: 0.5,
            },
            // ... other mood mappings
        }
    }
}
```

### 3. Context Awareness

#### Environmental Context:
```rust
#[derive(Debug, Clone)]
pub struct UserContext {
    pub time_of_day: TimeOfDay,
    pub day_of_week: DayOfWeek,
    pub weather: Option<WeatherCondition>,
    pub location_type: LocationType,
    pub activity: Option<UserActivity>,
    pub device_type: DeviceType,
}

#[derive(Debug, Clone)]
pub enum TimeOfDay {
    EarlyMorning,  // 5-8 AM
    Morning,       // 8-12 PM
    Afternoon,     // 12-5 PM
    Evening,       // 5-8 PM
    Night,         // 8-11 PM
    LateNight,     // 11 PM-5 AM
}

#[derive(Debug, Clone)]
pub enum WeatherCondition {
    Sunny,
    Cloudy,
    Rainy,
    Stormy,
    Snowy,
    Foggy,
}

#[derive(Debug, Clone)]
pub enum UserActivity {
    Working,
    Exercising,
    Commuting,
    Relaxing,
    Studying,
    Socializing,
    Sleeping,
}

impl UserContext {
    pub async fn detect_current() -> Result<Self> {
        Ok(Self {
            time_of_day: TimeOfDay::from_current_time(),
            day_of_week: DayOfWeek::from_current_date(),
            weather: WeatherAPI::get_current_weather().await.ok(),
            location_type: LocationDetector::detect().await.unwrap_or(LocationType::Unknown),
            activity: ActivityDetector::detect().await,
            device_type: DeviceType::detect(),
        })
    }
    
    pub fn influence_mood(&self, base_mood: &MoodVector) -> MoodVector {
        let mut mood = base_mood.clone();
        
        // Time of day influence
        match self.time_of_day {
            TimeOfDay::EarlyMorning => {
                mood.energy_level *= 0.8;
                mood.emotional_valence *= 1.1;
            },
            TimeOfDay::Night => {
                mood.energy_level *= 0.7;
                mood.rhythmic_intensity *= 0.8;
            },
            _ => {}
        }
        
        // Weather influence
        if let Some(weather) = &self.weather {
            match weather {
                WeatherCondition::Rainy => {
                    mood.emotional_valence *= 0.8;
                    mood.energy_level *= 0.9;
                },
                WeatherCondition::Sunny => {
                    mood.emotional_valence *= 1.2;
                    mood.energy_level *= 1.1;
                },
                _ => {}
            }
        }
        
        mood
    }
}
```

## 4. Mood-Based Recommendations

### Recommendation Engine:
```rust
pub struct MoodRecommendationEngine {
    song_features: HashMap<String, AudioFeatures>,
    user_preferences: UserMoodPreferences,
    listening_history: ListeningHistory,
    ml_model: Option<RecommendationModel>,
}

impl MoodRecommendationEngine {
    pub async fn recommend_for_mood(
        &self,
        target_mood: MoodCategory,
        context: &UserContext,
        limit: usize,
    ) -> Result<Vec<Song>> {
        let target_vector = target_mood.to_vector();
        let contextualized_vector = context.influence_mood(&target_vector);
        
        let mut candidates = self.find_matching_songs(&contextualized_vector).await?;
        
        // Apply collaborative filtering
        if let Some(model) = &self.ml_model {
            candidates = model.rerank_with_collaborative_filtering(candidates).await?;
        }
        
        // Apply diversity and freshness
        let recommendations = self.apply_diversity_and_freshness(candidates, limit);
        
        Ok(recommendations)
    }
    
    async fn find_matching_songs(&self, mood_vector: &MoodVector) -> Result<Vec<SongCandidate>> {
        let mut candidates = Vec::new();
        
        for (song_id, features) in &self.song_features {
            let song_mood = features.calculate_mood_vector();
            let similarity = self.calculate_mood_similarity(mood_vector, &song_mood);
            
            if similarity > 0.6 {  // Threshold for mood matching
                candidates.push(SongCandidate {
                    song_id: song_id.clone(),
                    similarity_score: similarity,
                    features: features.clone(),
                });
            }
        }
        
        // Sort by similarity
        candidates.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        Ok(candidates)
    }
    
    fn calculate_mood_similarity(&self, mood1: &MoodVector, mood2: &MoodVector) -> f32 {
        // Weighted Euclidean distance
        let energy_diff = (mood1.energy_level - mood2.energy_level).powi(2) * 0.3;
        let valence_diff = (mood1.emotional_valence - mood2.emotional_valence).powi(2) * 0.4;
        let rhythm_diff = (mood1.rhythmic_intensity - mood2.rhythmic_intensity).powi(2) * 0.2;
        let complexity_diff = (mood1.musical_complexity - mood2.musical_complexity).powi(2) * 0.1;
        
        let distance = (energy_diff + valence_diff + rhythm_diff + complexity_diff).sqrt();
        1.0 - (distance / 2.0).min(1.0)  // Normalize to 0-1 similarity
    }
}
```

### Smart Playlist Generation:
```rust
pub struct SmartPlaylistGenerator {
    recommendation_engine: MoodRecommendationEngine,
}

impl SmartPlaylistGenerator {
    pub async fn generate_mood_playlist(
        &self,
        mood: MoodCategory,
        duration_minutes: Option<u32>,
        user_preferences: &PlaylistPreferences,
    ) -> Result<Playlist> {
        let context = UserContext::detect_current().await?;
        let target_duration = duration_minutes.unwrap_or(60);
        
        // Generate seed songs
        let seed_songs = self.recommendation_engine
            .recommend_for_mood(mood.clone(), &context, 20)
            .await?;
        
        let mut playlist = Playlist::new(&format!("{:?} Mix", mood));
        let mut current_duration = 0u32;
        let mut energy_progression = self.calculate_energy_progression(&mood, target_duration);
        
        for (index, target_energy) in energy_progression.iter().enumerate() {
            let adjusted_mood = self.adjust_mood_for_energy(&mood, *target_energy);
            let next_songs = self.recommendation_engine
                .recommend_for_mood(adjusted_mood, &context, 5)
                .await?;
            
            if let Some(best_song) = self.select_best_transition_song(&next_songs, &playlist) {
                playlist.add_song(best_song.clone());
                current_duration += best_song.duration_seconds / 60;
                
                if current_duration >= target_duration {
                    break;
                }
            }
        }
        
        Ok(playlist)
    }
    
    fn calculate_energy_progression(&self, mood: &MoodCategory, duration: u32) -> Vec<f32> {
        match mood {
            MoodCategory::Workout => {
                // High energy throughout with peak in middle
                self.generate_workout_progression(duration)
            },
            MoodCategory::Study => {
                // Consistent moderate energy
                vec![0.4; (duration / 4) as usize]
            },
            MoodCategory::Party => {
                // Building energy with peaks
                self.generate_party_progression(duration)
            },
            _ => {
                // Default progression
                self.generate_default_progression(duration)
            }
        }
    }
}
```

## 5. User Feedback Learning

### Feedback Collection:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MoodFeedback {
    pub song_id: String,
    pub predicted_mood: MoodCategory,
    pub user_rating: MoodRating,
    pub context: UserContext,
    pub timestamp: DateTime<Utc>,
    pub feedback_type: FeedbackType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MoodRating {
    Perfect,      // Song perfectly matches mood
    Good,         // Song is good for this mood
    Okay,         // Song is acceptable
    Poor,         // Song doesn't fit mood
    Wrong,        // Song is completely wrong for mood
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FeedbackType {
    Explicit,     // User manually rated
    Implicit,     // Inferred from behavior (skip, repeat, etc.)
    Skip,         // User skipped the song
    LongListen,   // User listened to most/all of song
    Repeat,       // User repeated the song
    AddToPlaylist,// User added to personal playlist
}

pub struct FeedbackLearning {
    feedback_history: Vec<MoodFeedback>,
    user_mood_profile: UserMoodProfile,
}

impl FeedbackLearning {
    pub fn update_user_profile(&mut self, feedback: MoodFeedback) {
        self.feedback_history.push(feedback.clone());
        
        // Update user's mood preferences
        match feedback.user_rating {
            MoodRating::Perfect | MoodRating::Good => {
                self.user_mood_profile.reinforce_mood_song_association(
                    &feedback.predicted_mood,
                    &feedback.song_id,
                );
            },
            MoodRating::Poor | MoodRating::Wrong => {
                self.user_mood_profile.penalize_mood_song_association(
                    &feedback.predicted_mood,
                    &feedback.song_id,
                );
            },
            _ => {}
        }
        
        // Learn contextual preferences
        self.user_mood_profile.update_context_preferences(&feedback.context, &feedback);
    }
    
    pub async fn retrain_model(&mut self) {
        if self.feedback_history.len() >= 100 {  // Minimum training data
            let training_data = self.prepare_training_data();
            // Retrain ML model with new feedback
            // This could be done locally or via cloud service
        }
    }
}
```

## 6. UI Integration

### Mood Selection Interface:
```rust
// UI Components for mood selection
#[component]
pub fn MoodSelector(
    #[prop(into)] on_mood_selected: Callback<MoodCategory>,
    #[prop(optional)] current_context: Option<UserContext>,
) -> impl IntoView {
    let suggested_moods = create_memo(move |_| {
        if let Some(context) = &current_context.get() {
            suggest_moods_for_context(context)
        } else {
            default_mood_suggestions()
        }
    });
    
    view! {
        <div class="mood-selector">
            <h3>"How are you feeling?"</h3>
            
            // Context-aware suggestions
            <div class="suggested-moods">
                <For
                    each=suggested_moods
                    key=|mood| format!("{:?}", mood)
                    children=move |mood| {
                        view! {
                            <button 
                                class="mood-button suggested"
                                on:click=move |_| on_mood_selected.call(mood.clone())
                            >
                                {mood_to_emoji(&mood)} {format!("{:?}", mood)}
                            </button>
                        }
                    }
                />
            </div>
            
            // All available moods
            <div class="all-moods">
                // Grid of mood categories...
            </div>
        </div>
    }
}

#[component]
pub fn MoodPlaylist(
    #[prop(into)] mood: MoodCategory,
    #[prop(into)] songs: Vec<Song>,
) -> impl IntoView {
    view! {
        <div class="mood-playlist">
            <header class="playlist-header">
                <div class="mood-info">
                    <span class="mood-emoji">{mood_to_emoji(&mood)}</span>
                    <h2>{format!("{:?} Mix", mood)}</h2>
                    <p class="mood-description">{mood_description(&mood)}</p>
                </div>
                <div class="playlist-actions">
                    <button class="play-all">"Play All"</button>
                    <button class="shuffle">"Shuffle"</button>
                    <button class="save">"Save Playlist"</button>
                </div>
            </header>
            
            <SongList songs=songs />
        </div>
    }
}
```

### Mood Analytics Dashboard:
```rust
#[component]
pub fn MoodAnalytics() -> impl IntoView {
    let mood_history = use_mood_history();
    let mood_trends = create_memo(move |_| {
        analyze_mood_trends(&mood_history.get())
    });
    
    view! {
        <div class="mood-analytics">
            <h2>"Your Musical Moods"</h2>
            
            // Mood distribution chart
            <div class="mood-distribution">
                <MoodDistributionChart data=mood_trends />
            </div>
            
            // Time-based mood patterns
            <div class="mood-patterns">
                <MoodTimelineChart history=mood_history />
            </div>
            
            // Recommendations based on patterns
            <div class="mood-insights">
                <MoodInsights trends=mood_trends />
            </div>
        </div>
    }
}
```

## 7. API Endpoints

### REST API:
```rust
// Get mood recommendations
GET /api/v1/moods/{mood_category}/recommendations
Query parameters:
- limit: number (default: 20)
- context: json (optional user context)

// Generate mood playlist
POST /api/v1/moods/{mood_category}/playlist
Body: {
  "duration_minutes": 60,
  "preferences": { ... }
}

// Submit mood feedback
POST /api/v1/moods/feedback
Body: {
  "song_id": "string",
  "mood": "string", 
  "rating": "string",
  "context": { ... }
}

// Get user's mood profile
GET /api/v1/users/mood-profile

// Analyze song for mood features
POST /api/v1/songs/{song_id}/analyze-mood
```

## 8. Machine Learning Integration

### Feature Extraction Pipeline:
```rust
pub struct MoodMLPipeline {
    feature_extractor: AudioFeatureExtractor,
    mood_classifier: MoodClassifier,
    recommendation_model: CollaborativeFilteringModel,
}

impl MoodMLPipeline {
    pub async fn train_mood_classifier(&mut self, training_data: Vec<LabeledSong>) {
        let features = training_data.iter()
            .map(|song| self.feature_extractor.extract(&song.audio_file))
            .collect::<Result<Vec<_>>>()?;
        
        let labels = training_data.iter()
            .map(|song| song.mood_labels.clone())
            .collect();
        
        self.mood_classifier.train(features, labels).await?;
    }
    
    pub async fn update_recommendation_model(&mut self, user_interactions: Vec<UserInteraction>) {
        // Update collaborative filtering model with new user interactions
        self.recommendation_model.update(user_interactions).await?;
    }
}
```

## 9. Performance Considerations

### Caching Strategy:
- **Audio Features**: Cache extracted features to avoid recomputation
- **Mood Predictions**: Cache mood classifications for songs
- **Recommendations**: Cache recent recommendations with TTL
- **User Profiles**: Cache user mood profiles with periodic updates

### Optimization:
- **Batch Processing**: Process multiple songs simultaneously
- **Background Analysis**: Analyze new songs in background
- **Progressive Loading**: Load recommendations progressively
- **Smart Prefetching**: Prefetch likely next songs

## 10. Privacy and Ethics

### Privacy Protection:
- **Local Processing**: Keep sensitive mood data on device when possible
- **Anonymization**: Anonymize mood data before cloud processing
- **User Control**: Allow users to control data sharing
- **Transparency**: Clear explanation of how mood data is used

### Ethical Considerations:
- **Mood Manipulation**: Avoid manipulating user emotions
- **Diverse Recommendations**: Prevent filter bubbles
- **Mental Health**: Recognize signs of mood-related mental health issues
- **Cultural Sensitivity**: Respect cultural differences in music and emotion
