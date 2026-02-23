mod core;
mod parsers;
mod compilers;

use std::env;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    println!["=== MCF Engine v0.7 ==="];
    
    let args: Vec<String> = env::args().collect();
    let target_file = if args.len() > 1 {
        args[1].clone()
    } else {
        println!["\n--- Main Menu ---"];
        print!["Enter file name [.als, .xml, .mcf]: "];
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Input reading error");
        input.trim().to_string()
    };

    if target_file.is_empty() {
        eprintln!["Error: File name cannot be empty."];
        return;
    }

    if !Path::new(&target_file).exists() {
        eprintln!["Error: File [{}] not found.", target_file];
        return;
    }

    let project_result = if target_file.ends_with(".mcf") {
        parsers::mcf_in::parse(&target_file)
    } else {
        parsers::ableton::parse(&target_file)
    };

    match project_result {
        Ok(project) => {
            println!["\nProject [{}] [BPM: {:.1}] successfully loaded!", project.title, project.bpm];
            println!["   Tracks found: {}", project.tracks.len()];
            println!["   Notes found: {}", project.notes.len()];

            if target_file.ends_with(".mcf") {
                println!["\n--- Export Menu ---"];
                println!["[1] Export as Ableton Live Set [.als]"];
                println!["[2] Export as Ableton XML [.xml]"];
                print!["Choose format [1/2]: "];
                io::stdout().flush().unwrap();

                let mut choice = String::new();
                io::stdin().read_line(&mut choice).expect("Input reading error");
                
                let output_ext = if choice.trim() == "2" { ".xml" } else { ".als" };
                let output_file = format!("{}{}", project.title, output_ext);
                
                match compilers::ableton_out::write(&project, &output_file) {
                    Ok(_) => println!["SUCCESS! File [{}] saved.", output_file],
                    Err(e) => eprintln!["Save error: {}", e],
                }
            } else {
                let output_mcf = format!("{}.mcf", project.title);
                match compilers::mcf_out::write(&project, &output_mcf) {
                    Ok(_) => println!["SUCCESS! File [{}] saved.", output_mcf],
                    Err(e) => eprintln!["Save error: {}", e],
                }
            }
        }
        Err(e) => eprintln!["Critical Error: {}", e],
    }
}