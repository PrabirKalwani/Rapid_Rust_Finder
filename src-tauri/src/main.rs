use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Error};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::api::path::config_dir;

const MINIMUM_SCORE: i16 = 20;
const SKIP_DIRECTORY: &str = "Library"; // Directory to skip
const ROOT_FOLDER: &str = "E:\\";

/// Scores the filename based on whether it starts with the query string
fn score_filename(filename: &str, query: &str) -> i16 {
    if filename.starts_with(query) {
        return 1000; // Return a high score for filenames that start with the query
    }
    0 // Return 0 for filenames that do not start with the query
}

/// Recursively indexes directories and subdirectories
fn index_recursive(path: &Path, index: &mut FileIndex) {
    if path.ends_with(SKIP_DIRECTORY) {
        return; // Skip the directory if it matches the skip pattern
    }

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    let filename = entry.file_name().into_string().unwrap();
                    let file_path = entry_path.display().to_string();

                    // Store the filename and path in the index
                    index.files.insert(filename.clone(), file_path.clone());

                    if entry_path.is_dir() {
                        index_recursive(&entry_path, index); // Recurse into subdirectories
                    }
                }
            }
        }
        Err(e) => println!("Error reading directory: {}", e),
    }
}

/// Saves the index to a file
fn save_index(index: &FileIndex, index_path: &Path) {
    let file = File::create(index_path).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, index).unwrap();
}

/// Loads the index from a file
fn load_index(index_path: &Path) -> FileIndex {
    let file = File::open(index_path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

/// Data structure to hold the index
#[derive(Serialize, Deserialize, Debug)]
struct FileIndex {
    files: HashMap<String, String>, // Maps filename to file path
}

#[derive(Serialize, Deserialize)]
struct Data {
    recent: Vec<String>,
}

/// Searches for files based on the query
#[tauri::command]
fn search_files(query: String) -> Vec<(String, String)> {
    let start_time = Instant::now(); // Start the timer

    let index_path = config_dir().unwrap().join("file_index.json");
    let index = if index_path.exists() {
        // Load the existing index
        println!("Loading existing index...");
        load_index(&index_path)
    } else {
        // Create a new index
        println!("Creating new index...");
        let start_path = Path::new(ROOT_FOLDER);
        let mut new_index = FileIndex {
            files: HashMap::new(),
        };
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

    let duration = start_time.elapsed(); // Measure the elapsed time
    println!("Search completed in {:?}", duration);
    results
}

#[tauri::command]
fn process_recent(data: Vec<String>) -> Result<(), String> {
    // Path to save the JSON file
    let file_path = config_dir().unwrap().join("recents.json");

    // Create a struct or directly serialize the vector
    let data = Data { recent: data };

    // Serialize the vector to JSON
    let json_data = serde_json::to_string_pretty(&data)
        .map_err(|err| format!("Failed to serialize data: {}", err))?;

    // Write the JSON to a file
    let mut file =
        File::create(file_path).map_err(|err| format!("Failed to create file: {}", err))?;

    file.write_all(json_data.as_bytes())
        .map_err(|err| format!("Failed to write data to file: {}", err))?;

    Ok(())
}

#[tauri::command]
fn get_recent_data() -> Result<Data, String> {
    let file_path = config_dir().unwrap().join("recents.json");

    // Open the JSON file
    let file = File::open(file_path).map_err(|err| format!("Failed to open file: {}", err))?;

    // Create a buffered reader for the file
    let reader = BufReader::new(file);

    // Deserialize JSON data into a Data struct
    let data: Data = serde_json::from_reader(reader)
        .map_err(|err| format!("Failed to parse JSON data: {}", err))?;

    // Return the deserialized data to the frontend
    Ok(data)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![search_files, process_recent, get_recent_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
