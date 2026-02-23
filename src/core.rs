#[derive(Debug)]
pub struct McfProject {
    pub title: String,
    pub bpm: f32,
    pub scale_root: String,
    pub scale_name: String,
    pub tracks: Vec<Track>,
    pub notes: Vec<Note>,
}

#[derive(Debug)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub track_type: String,
    pub volume: f32,
    pub pan: f32,
}

#[derive(Debug)]
pub struct Note {
    pub track_id: String,
    pub pitch_name: String,
    pub pitch: u8,
    pub velocity: u8,
    pub start: f32,
    pub duration: f32,
}