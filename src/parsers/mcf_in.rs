use crate::core::{McfProject, Note, Track};
use std::fs;

fn name_to_pitch(name: &str) -> u8 {
    let mut note_str = String::new();
    let mut octave_str = String::new();

    for c in name.chars() {
        if c.is_alphabetic() || c == '#' {
            note_str.push(c);
        } else if c.is_digit(10) || c == '-' {
            octave_str.push(c);
        }
    }

    let note_val = match note_str.as_str() {
        "C" => 0, "C#" => 1, "D" => 2, "D#" => 3, "E" => 4,
        "F" => 5, "F#" => 6, "G" => 7, "G#" => 8, "A" => 9,
        "A#" => 10, "B" => 11, _ => 0,
    };

    let octave = octave_str.parse::<i32>().unwrap_or(3);
    let pitch = (octave + 2) * 12 + note_val;
    
    if pitch < 0 { 0 } else if pitch > 255 { 255 } else { pitch as u8 }
}

pub fn parse(file_path: &str) -> Result<McfProject, Box<dyn std::error::Error>> {
    println!["Reading MCF: {}...", file_path];
    let text = fs::read_to_string(file_path)?;

    let mut project = McfProject {
        title: String::from("Unknown"),
        bpm: 120.0,
        scale_root: String::from("C"),
        scale_name: String::from("Major"),
        tracks: Vec::new(),
        notes: Vec::new(),
    };

    let mut current_section = "meta";

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("#") {
            continue;
        }

        if line == "tr:" {
            current_section = "tr";
            continue;
        }
        if line == "nt:" {
            current_section = "nt";
            continue;
        }

        match current_section {
            "meta" => {
                if line.starts_with("project:") {
                    project.title = line.replace("project:", "").replace("\"", "").trim().to_string();
                } else if line.starts_with("bpm:") {
                    let bpm_str = line.replace("bpm:", "").trim().to_string();
                    project.bpm = bpm_str.parse::<f32>().unwrap_or(120.0);
                } else if line.starts_with("key:") {
                    project.scale_root = line.replace("key:", "").replace("\"", "").trim().to_string();
                } else if line.starts_with("scale:") {
                    project.scale_name = line.replace("scale:", "").replace("\"", "").trim().to_string();
                }
            }
            "tr" => {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 5 {
                    let mut clean_name = parts[1].replace("\"", "");
                    
                    if let Some(dash_idx) = clean_name.find('-') {
                        if clean_name[..dash_idx].chars().all(|c| c.is_digit(10)) {
                            clean_name = clean_name[dash_idx + 1..].trim().to_string();
                        }
                    }

                    project.tracks.push(Track {
                        id: parts[0].to_string(),
                        name: clean_name,
                        track_type: parts[2].to_string(),
                        volume: parts[3].parse::<f32>().unwrap_or(0.85),
                        pan: parts[4].parse::<f32>().unwrap_or(0.0),
                    });
                }
            }
            "nt" => {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() >= 5 {
                    let pitch_name = parts[1].to_string();
                    project.notes.push(Note {
                        track_id: parts[0].to_string(),
                        pitch_name: pitch_name.clone(),
                        pitch: name_to_pitch(&pitch_name),
                        velocity: parts[2].parse::<u8>().unwrap_or(100),
                        start: parts[3].parse::<f32>().unwrap_or(0.0),
                        duration: parts[4].parse::<f32>().unwrap_or(1.0),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(project)
}