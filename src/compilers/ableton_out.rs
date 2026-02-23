use crate::core::McfProject;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

fn rewrite_ids(xml: &str, start_id: &mut usize) -> String {
    let mut result = String::new();
    let mut parts = xml.split("Id=\"");
    result.push_str(parts.next().unwrap_or(""));
    for part in parts {
        if let Some(end_quote) = part.find("\"") {
            result.push_str(&format!["Id=\"{}\"", start_id]);
            *start_id += 1;
            result.push_str(&part[end_quote + 1..]);
        } else {
            result.push_str("Id=\"");
            result.push_str(part);
        }
    }
    result
}

fn replace_name(xml: &str, new_name: &str) -> String {
    let tag = "<EffectiveName Value=\"";
    let mut res = xml.to_string();
    if let Some(start) = res.find(tag) {
        if let Some(end_quote) = res[start + tag.len()..].find("\"") {
            let abs_end = start + tag.len() + end_quote;
            res.replace_range(start + tag.len()..abs_end, new_name);
        }
    }
    res
}

fn inject_clip(track_xml: &str, clip_xml: &str) -> String {
    let mut res = track_xml.to_string();
    if let Some(pos) = res.find("<Events>") {
        res.insert_str(pos + 8, clip_xml);
    } else if let Some(pos) = res.find("<Events />") {
        res.replace_range(pos..pos + 10, &format!["<Events>{}</Events>", clip_xml]);
    } else if let Some(pos) = res.find("<Events/>") {
        res.replace_range(pos..pos + 9, &format!["<Events>{}</Events>", clip_xml]);
    }
    res
}

fn set_tempo(xml: &str, bpm: f32) -> String {
    let mut res = xml.to_string();
    if let Some(start) = res.find("<Tempo>") {
        if let Some(end) = res[start..].find("</Tempo>") {
            let abs_end = start + end + 8;
            res.replace_range(start..abs_end, &format!["<Tempo><Manual Value=\"{}\"/></Tempo>", bpm]);
        }
    }
    res
}

fn update_next_pointee_id(xml: &str, max_id: usize) -> String {
    let tag = "<NextPointeeId Value=\"";
    let mut res = xml.to_string();
    if let Some(start) = res.find(tag) {
        if let Some(end_quote) = res[start + tag.len()..].find("\"") {
            let abs_end = start + tag.len() + end_quote;
            res.replace_range(start + tag.len()..abs_end, &max_id.to_string());
        }
    }
    res
}

pub fn write(project: &McfProject, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let template_file = File::open("Template.als").map_err(|_| "Template.als not found")?;
    let mut decoder = GzDecoder::new(template_file);
    let mut template_xml = String::new();
    decoder.read_to_string(&mut template_xml)?;

    let track_start = template_xml.find("<MidiTrack").ok_or("No MidiTrack found in template")?;
    let track_end = template_xml[track_start..].find("</MidiTrack>").ok_or("No closing MidiTrack tag")? + track_start + 12;

    let mut prefix = template_xml[..track_start].to_string();
    let suffix = template_xml[track_end..].to_string();
    let track_template = template_xml[track_start..track_end].to_string();

    prefix = set_tempo(&prefix, project.bpm);

    let mut id_counter = 100000;
    let mut tracks_output = String::new();

    for track in &project.tracks {
        let mut track_xml = track_template.clone();
        track_xml = rewrite_ids(&track_xml, &mut id_counter);
        track_xml = replace_name(&track_xml, &track.name);

        let mut notes_by_pitch: HashMap<u8, Vec<&crate::core::Note>> = HashMap::new();
        for note in project.notes.iter().filter(|n| n.track_id == track.id) {
            notes_by_pitch.entry(note.pitch).or_insert_with(Vec::new).push(note);
        }

        let has_notes = !notes_by_pitch.is_empty();
        
        let mut clip_end: f32 = 4.0;
        if has_notes {
            for notes in notes_by_pitch.values() {
                for note in notes {
                    let end_time = note.start + note.duration;
                    if end_time > clip_end {
                        clip_end = end_time;
                    }
                }
            }
        }
        let clip_length = (clip_end / 4.0).ceil() * 4.0;

        let mut clip = String::new();
        clip.push_str(&format!["<MidiClip Id=\"{}\">", id_counter]);
        id_counter += 1;
        
        clip.push_str(&format![
            "<CurrentStart Value=\"0\"/><CurrentEnd Value=\"{0}\"/><Loop><LoopStart Value=\"0\"/><LoopEnd Value=\"{0}\"/><StartRelative Value=\"0\"/><LoopOn Value=\"true\"/><HiddenLoopStart Value=\"0\"/><HiddenLoopEnd Value=\"{0}\"/></Loop>",
            clip_length
        ]);
        
        clip.push_str("<Name><EffectiveName Value=\"MCF Clip\"/></Name>");
        clip.push_str("<Notes><KeyTracks>");

        if has_notes {
            for (pitch, notes) in notes_by_pitch {
                clip.push_str(&format!["<KeyTrack Id=\"{}\">", id_counter]);
                id_counter += 1;
                clip.push_str(&format!["<MidiKey Value=\"{}\"/>", pitch]);
                clip.push_str("<Notes>");
                for note in notes {
                    clip.push_str(&format!["<MidiNoteEvent Time=\"{}\" Duration=\"{}\" Velocity=\"{}\"/>", note.start, note.duration, note.velocity]);
                }
                clip.push_str("</Notes>");
                clip.push_str("</KeyTrack>");
            }
        }

        clip.push_str("</KeyTracks></Notes>");
        clip.push_str("</MidiClip>");

        if has_notes {
            track_xml = inject_clip(&track_xml, &clip);
        }

        tracks_output.push_str(&track_xml);
    }

    prefix = update_next_pointee_id(&prefix, id_counter + 1);

    let file = File::create(output_path)?;
    let mut writer: Box<dyn Write> = if output_path.ends_with(".als") {
        Box::new(GzEncoder::new(file, Compression::default()))
    } else {
        Box::new(file)
    };

    write![writer, "{}", prefix]?;
    write![writer, "{}", tracks_output]?;
    write![writer, "{}", suffix]?;

    Ok(())
}