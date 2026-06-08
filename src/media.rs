use mpris::{PlaybackStatus, PlayerFinder};

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub status: MediaStatus,
    pub player_name: String,
    pub position: Option<std::time::Duration>,
    pub length: Option<std::time::Duration>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaStatus {
    Playing,
    Paused,
    Stopped,
}

impl Default for MediaState {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            status: MediaStatus::Stopped,
            player_name: String::new(),
            position: None,
            length: None,
        }
    }
}

impl MediaState {
    pub fn load() -> Self {
        let Ok(finder) = PlayerFinder::new() else {
            return Self::default();
        };

        // grab the first active player
        let Ok(player) = finder.find_active() else {
            return Self::default();
        };

        let status = match player
            .get_playback_status()
            .unwrap_or(PlaybackStatus::Stopped)
        {
            PlaybackStatus::Playing => MediaStatus::Playing,
            PlaybackStatus::Paused => MediaStatus::Paused,
            _ => MediaStatus::Stopped,
        };

        let (title, artist, length) = player
            .get_metadata()
            .map(|m| {
                let title = m.title().unwrap_or("Unknown").to_string();
                let artist = m
                    .artists()
                    .and_then(|a| a.first().cloned())
                    .unwrap_or_default()
                    .to_string();
                let length = m.length();
                (title, artist, length)
            })
            .unwrap_or_default();

        let position = player.get_position().ok();

        Self {
            title,
            artist,
            status,
            player_name: player.identity().to_string(),
            length,
            position,
        }
    }

    pub fn play_pause() {
        let Ok(finder) = PlayerFinder::new() else {
            return;
        };
        let Ok(player) = finder.find_active() else {
            return;
        };
        let _ = player.play_pause();
    }

    pub fn next() {
        let Ok(finder) = PlayerFinder::new() else {
            return;
        };
        let Ok(player) = finder.find_active() else {
            return;
        };
        let _ = player.next();
    }

    pub fn previous() {
        let Ok(finder) = PlayerFinder::new() else {
            return;
        };
        let Ok(player) = finder.find_active() else {
            return;
        };
        let _ = player.previous();
    }
}
