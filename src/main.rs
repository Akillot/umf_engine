mod core;
mod parsers;
mod compilers;

use std::env;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    println!("=== MCF Engine v0.3 ===");
    
    let args: Vec<String> = env::args().collect();
    let target_file = if args.len() > 1 {
        args[1].clone()
    } else {
        print!("Enter XML file name [e.g. TestAbleton.xml]: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Input reading error");
        input.trim().to_string()
    };

    if target_file.is_empty() {
        eprintln!("Error: File name cannot be empty.");
        return;
    }

    if !Path::new(&target_file).exists() {
        eprintln!("Error: File [{}] not found.", target_file);
        return;
    }

    match parsers::ableton::parse(&target_file) {
        Ok(project) => {
            println!("\nProject [{}] [BPM: {:.1}] successfully read!", project.title, project.bpm);
            println!("   Tracks found: {}", project.tracks.len());
            println!("   Notes found: {}", project.notes.len());

            let output_mcf = format!("{}.mcf", project.title);
            match compilers::mcf_out::write(&project, &output_mcf) {
                Ok(_) => println!("SUCCESS! File [{}] saved.", output_mcf),
                Err(e) => eprintln!("Save error: {}", e),
            }
        }
        Err(e) => eprintln!("Critical Error: {}", e),
    }
}