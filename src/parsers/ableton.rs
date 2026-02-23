use crate::core::{McfProject, Note, Track};
use roxmltree::Document;
use std::fs;
use std::path::Path;
use flate2::read::GzDecoder;
use std::io::Read;

fn pitch_to_name(pitch: u8) -> String {
    let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (pitch as i32 / 12) - 2;
    let note = notes[pitch as usize % 12];
    format!["{}{}", note, octave]
}

pub fn parse(file_path: &str) -> Result<McfProject, Box<dyn std::error::Error>> {
    println!["Reading file: {}...", file_path];

    let mut text = String::new();
    if file_path.ends_with(".als") {
        let file = fs::File::open(file_path)?;
        let mut gz = GzDecoder::new(file);
        gz.read_to_string(&mut text)?;
    } else {
        text = fs::read_to_string(file_path)?;
    }

    let doc = Document::parse(&text)?;

    let mut bpm: f32 = 120.0;
    for node in doc.descendants() {
        if node.has_tag_name("Tempo") {
            if let Some(manual) = node.children().find(|n| n.has_tag_name("Manual")) {
                if let Some(val) = manual.attribute("Value") {
                    bpm = val.parse::<f32>().unwrap_or(120.0);
                    break;
                }
            }
        }
    }

    let mut scale_root = String::from("C");
    let mut scale_name = String::from("Major");
    for node in doc.descendants() {
        if node.has_tag_name("ScaleInformation") {
            if let Some(root_node) = node.descendants().find(|n| n.has_tag_name("RootNote")) {
                if let Some(val) = root_node.attribute("Value") {
                    let root_idx = val.parse::<usize>().unwrap_or(0);
                    let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
                    scale_root = notes[root_idx % 12].to_string();
                }
            }
            if let Some(name_node) = node.descendants().find(|n| n.has_tag_name("Name")) {
                if let Some(val) = name_node.attribute("Value") {
                    let scales = [
                        "Major", "Minor", "Dorian", "Mixolydian", "Lydian", "Phrygian", "Locrian",
                        "Diminished", "Whole-half", "Whole Tone", "Minor Blues", "Minor Pentatonic",
                        "Major Pentatonic", "Harmonic Minor", "Melodic Minor", "Super Locrian",
                        "Bhairav", "Hungarian Minor", "Minor Gypsy", "Hirojoshi", "In-Sen", "Iwato",
                        "Kumoi", "Pelog", "Spanish"
                    ];
                    
                    if let Ok(idx) = val.parse::<usize>() {
                        if idx < scales.len() {
                            scale_name = scales[idx].to_string();
                        } else {
                            scale_name = format!["Scale_{}", idx];
                        }
                    } else {
                        scale_name = val.to_string();
                    }
                }
            }
            break;
        }
    }

    let mut tracks = Vec::new();
    let mut notes = Vec::new();
    let mut track_counter = 1;

    for track_node in doc.descendants() {
        if track_node.has_tag_name("MidiTrack") || track_node.has_tag_name("AudioTrack") {
            let track_type = if track_node.has_tag_name("MidiTrack") { "midi" } else { "audio" };
            let track_id = format!["t{:02}", track_counter];

            let mut track_name = format!["Track {}", track_counter];
            if let Some(name_node) = track_node.descendants().find(|n| n.has_tag_name("Name")) {
                let mut found_user_name = false;
                if let Some(user_name) = name_node.descendants().find(|n| n.has_tag_name("UserName")) {
                    if let Some(val) = user_name.attribute("Value") {
                        if !val.is_empty() {
                            track_name = val.to_string();
                            found_user_name = true;
                        }
                    }
                }
                
                if !found_user_name {
                    if let Some(eff_name) = name_node.descendants().find(|n| n.has_tag_name("EffectiveName")) {
                        if let Some(val) = eff_name.attribute("Value") {
                            if !val.is_empty() {
                                track_name = val.to_string();
                                if let Some(dash_idx) = track_name.find('-') {
                                    if track_name[..dash_idx].chars().all(|c| c.is_digit(10)) {
                                        track_name = track_name[dash_idx + 1..].to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut volume: f32 = 0.85;
            let mut pan: f32 = 0.0;

            if let Some(mixer) = track_node.descendants().find(|n| n.has_tag_name("Mixer")) {
                if let Some(vol_node) = mixer.descendants().find(|n| n.has_tag_name("Volume")) {
                    if let Some(manual) = vol_node.children().find(|n| n.has_tag_name("Manual")) {
                        if let Some(val) = manual.attribute("Value") {
                            volume = val.parse::<f32>().unwrap_or(0.85);
                        }
                    }
                }
                if let Some(pan_node) = mixer.descendants().find(|n| n.has_tag_name("Pan")) {
                    if let Some(manual) = pan_node.children().find(|n| n.has_tag_name("Manual")) {
                        if let Some(val) = manual.attribute("Value") {
                            pan = val.parse::<f32>().unwrap_or(0.0);
                        }
                    }
                }
            }

            tracks.push(Track {
                id: track_id.clone(),
                name: track_name,
                track_type: track_type.to_string(),
                volume,
                pan,
            });

            if track_type == "midi" {
                for key_track in track_node.descendants().filter(|n| n.has_tag_name("KeyTrack")) {
                    let mut pitch = 60;
                    if let Some(midi_key) = key_track.descendants().find(|n| n.has_tag_name("MidiKey")) {
                        if let Some(val) = midi_key.attribute("Value") {
                            pitch = val.parse::<u8>().unwrap_or(60);
                        }
                    }

                    for note_node in key_track.descendants().filter(|n| n.has_tag_name("MidiNoteEvent")) {
                        let velocity = note_node.attribute("Velocity").unwrap_or("100").parse::<u8>().unwrap_or(100);
                        let start = note_node.attribute("Time").unwrap_or("0.0").parse::<f32>().unwrap_or(0.0);
                        let duration = note_node.attribute("Duration").unwrap_or("1.0").parse::<f32>().unwrap_or(1.0);

                        notes.push(Note {
                            track_id: track_id.clone(),
                            pitch_name: pitch_to_name(pitch),
                            pitch,
                            velocity,
                            start,
                            duration,
                        });
                    }
                }
            }
            track_counter += 1;
        }
    }

    let project_name = Path::new(file_path)
        .file_stem()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("Unknown_Project");

    Ok(McfProject {
        title: project_name.to_string(),
        bpm,
        scale_root,
        scale_name,
        tracks,
        notes,
    })
}