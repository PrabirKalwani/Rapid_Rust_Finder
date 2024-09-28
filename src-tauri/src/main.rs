use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use serde_json;
use tauri::api::path::config_dir;
use std::time::Instant;

const MINIMUM_SCORE: i16 = 20;
const SKIP_DIRECTORY: &str = "Library"; 
const ROOT_FOLDER: &str = "/Users/aayushshah";


fn score_filename(filename: &str, query: &str) -> i16 {
    if filename.starts_with(query) {
        return 1000; 
    }
    0 
}


fn index_recursive(path: &Path, index: &mut FileIndex) {
    if path.ends_with(SKIP_DIRECTORY) {
        return; 
    }

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    let filename = entry.file_name().into_string().unwrap();
                    let file_path = entry_path.display().to_string();
                    
                    
                    index.files.insert(filename.clone(), file_path.clone());

                    if entry_path.is_dir() {
                        index_recursive(&entry_path, index); 
                    }
                }
            }
        }
        Err(e) => println!("Error reading directory: {}", e),
    }
}


fn save_index(index: &FileIndex, index_path: &Path) {
    let file = File::create(index_path).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, index).unwrap();
}


fn load_index(index_path: &Path) -> FileIndex {
    let file = File::open(index_path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}


#[derive(Serialize, Deserialize, Debug)]
struct FileIndex {
    files: HashMap<String, String>, 
}


#[tauri::command]
fn search_files(query: String) -> Vec<(String, String)> {
    let start_time = Instant::now(); 

    let index_path = config_dir().unwrap().join("new_index.json");
    let index = if index_path.exists() {
        
        println!("Loading existing index...");
        load_index(&index_path)
    } else {
        
        println!("Creating new index...");
        let start_path = Path::new(ROOT_FOLDER);
        let mut new_index = FileIndex { files: HashMap::new() };
        index_recursive(start_path, &mut new_index);
        save_index(&new_index, &index_path);
        new_index
    };

    let mut results = Vec::new();
    for (filename, file_path) in &index.files {
        let cleaned_filename = Path::new(filename)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");

        let score = score_filename(cleaned_filename, &query);
        if score >= MINIMUM_SCORE {
            results.push((filename.clone(), file_path.clone()));
        }
    }

    let duration = start_time.elapsed(); 
    println!("Search completed in {:?}", duration);
    results
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![search_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
