// TODO:
// - Fix Indexation Time
// - Add BFS
// - Add The Scoring system for favorite extension types

//// Imports
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Instant, SystemTime};
use once_cell::sync::Lazy;
use tauri::api::path::config_dir;
use std::env::consts::OS as OS_TYPE;



//// Constants
const MINIMUM_SCORE: i16 = 20;
const SKIP_DIRECTORY: &str = "Library"; // Directory to skip
const OS: &str = OS_TYPE;



//// Global Variables
static ROOT_FOLDER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static EXTENSIONS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));



//// Data Structures
/// Data structure to hold the index
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileDetails {
    file_path: String,
    file_size: u64,
    file_type: String,
    creation_date: Option<SystemTime>,
}

/// Data structure to hold the index of files
#[derive(Serialize, Deserialize, Debug)]
struct FileIndex {
    files: HashMap<String, FileDetails>, // Maps filename to file details
}

/// Data Structure to hold the Most Recently used Files
#[derive(Serialize, Deserialize)]
struct Data {
    file_name: String,
    file_path: String,
    file_size: u64,
    file_type: String,
    creation_date: Option<SystemTime>,
}

type FileIndexMap = HashMap<i32, Data>;



//// Global Variables Getter and Setters
// Helper function to set ROOT_FOLDER using Lazy<Mutex>
fn set_root_folder(root_folder: String) -> Result<(), String> {
    let mut folder = ROOT_FOLDER.lock().map_err(|e| e.to_string())?;
    *folder = root_folder;
    Ok(())
}

// Helper function to set EXTENSIONS using Lazy<Mutex>
fn set_extensions(extensions: Vec<String>) -> Result<(), String> {
    let mut ext = EXTENSIONS.lock().map_err(|e| e.to_string())?;
    *ext = extensions;
    Ok(())
}

// Function to get the current value of ROOT_FOLDER
fn get_root_folder() -> Result<String, String> {
    let folder = ROOT_FOLDER.lock().map_err(|e| e.to_string())?;
    Ok(folder.clone())
}

// Function to get the current value of EXTENSIONS
fn get_extensions() -> Result<Vec<String>, String> {
    let ext = EXTENSIONS.lock().map_err(|e| e.to_string())?;
    Ok(ext.clone())
}



//// Setup Function
/// Function to check if setup file exists
#[tauri::command]
async fn setup_file_check() -> Result<bool, String> {
    // Get the path to the setup file
    let path = config_dir().unwrap().join("setup_file.json");

    // Check if the file exists
    let exists = Path::new(&path).exists();
    
    Ok(exists)
}

/// Function to detect OS and return it
#[tauri::command]
async fn detect_os() -> Result<String, String> {
    // Simply return the OS constant
    Ok(OS.to_string())
}

/// Function to save root folder
#[tauri::command]
async fn save_root_folder(root_folder: String) -> Result<(), String> {
    // Update the global ROOT_FOLDER
    let _ = set_root_folder(root_folder.clone());

    // Save root folder in setup.json
    let mut setup = load_setup().await?;
    setup["root_folder"] = serde_json::Value::String(root_folder);
    save_setup(&setup).await?;
    Ok(())
}

/// Function to save extensions
#[tauri::command]
async fn save_file_extensions(extensions: Vec<String>) -> Result<(), String> {
    // Update the global EXTENSIONS
    let _ = set_extensions(extensions.clone());

    // Save file extensions in setup.json
    let mut setup = load_setup().await?;
    setup["file_extensions"] = serde_json::Value::Array(
        extensions.into_iter().map(serde_json::Value::String).collect()
    );
    save_setup(&setup).await?;
    Ok(())
}

///Function to save the setup_file.json
async fn save_setup(setup: &serde_json::Value) -> Result<(), String> {
    let path: PathBuf = config_dir().unwrap().join("setup_file.json");
    fs::write(&path, setup.to_string()).map_err(|e| e.to_string())
}

///Function to load the setup_file.json
#[tauri::command]
async fn load_setup() -> Result<serde_json::Value, String> {
    let path: PathBuf = config_dir().unwrap().join("setup_file.json");

    let setup: serde_json::Value = if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        // If file doesn't exist, return a new JSON structure with defaults
        json!({
            "os": detect_os().await.unwrap(), 
            "root_folder": "", 
            "file_extensions": []
        })
    };

    // Load the values from the setup file into global variables
    if let Some(root_folder) = setup.get("root_folder").and_then(|v| v.as_str()) {
        set_root_folder(root_folder.to_string())?;
    }

    if let Some(extensions) = setup.get("file_extensions").and_then(|v| v.as_array()) {
        let extensions_vec: Vec<String> = extensions
            .iter()
            .filter_map(|ext| ext.as_str().map(|s| s.to_string()))
            .collect();
        set_extensions(extensions_vec)?;
    }

    Ok(setup)
}



//// Functions to Handle Indexing
/// Scores the filename based on whether it starts with the query string
fn score_filename(file_name: &str, query: &str) -> i16 {
    if file_name.starts_with(query) {
        return 1000; // Return a high score for filenames that start with the query
    }
    0 // Return 0 for filenames that do not start with the query
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
                    let file_name = entry.file_name().into_string().unwrap();
                    let file_path = entry_path.display().to_string();

                    // Get metadata
                    if let Ok(metadata) = entry.metadata() {
                        let file_size = metadata.len(); // File size
                        let file_type = if metadata.is_dir() {
                            "directory".to_string()
                        } else if metadata.is_file() {
                            "file".to_string()
                        } else {
                            "unknown".to_string()
                        };
                        let creation_date = metadata.created().ok(); // File creation date

                        // Store the filename and associated details in the index
                        let details = FileDetails {
                            file_path: file_path.clone(),
                            file_size,
                            file_type,
                            creation_date,
                        };
                        index.files.insert(file_name.clone(), details);
                    }

                    // Recurse into subdirectories
                    if entry_path.is_dir() {
                        index_recursive(&entry_path, index);
                    }
                }
            }
        }
        Err(e) => println!("Error reading directory: {}", e),
    }
}



//// Search and Recent Export functions
/// Searches for files based on the query
#[tauri::command]
fn search_files(query: String) -> Result<Vec<(String, FileDetails)>, String> {
    let start_time = Instant::now(); // Start the timer

    let index_path = config_dir().unwrap().join("file_index.json");
    let index = if index_path.exists() {
        // Load the existing index
        println!("Loading existing index...");
        load_index(&index_path)
    } else {
        // Create a new index
        println!("Creating new index...");
        let root_folder = get_root_folder()?;
        println!("{} is the root folder", &root_folder);
        let start_path = Path::new(&root_folder);
        let mut new_index = FileIndex {
            files: HashMap::new(),
        };
        index_recursive(start_path, &mut new_index);
        save_index(&new_index, &index_path);
        new_index
    };

    let mut results = Vec::new();
    for (file_name, details) in &index.files {
        let cleaned_file_name = Path::new(file_name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");

        let score = score_filename(cleaned_file_name, &query);
        if score >= MINIMUM_SCORE {
            results.push((file_name.clone(), (*details).clone())); // Dereference and clone FileDetails
        }
    }

    let duration = start_time.elapsed(); // Measure the elapsed time
    println!("Search completed in {:?}", duration);
    Ok(results)
}

/// Function to save the most recently opened files into recent_files.json
#[tauri::command]
fn process_recent(data: FileIndexMap) -> Result<(), String> {
    let file_path: PathBuf = config_dir().unwrap().join("recent_files.json");

    let json_data = serde_json::to_string_pretty(&data)
        .map_err(|err| format!("Failed to serialize data: {}", err))?;

    let mut file = File::create(file_path)
        .map_err(|err| format!("Failed to create recent_files.json: {}", err))?;

    file.write_all(json_data.as_bytes())
        .map_err(|err| format!("Failed to write data to recent_files.json: {}", err))?;

    println!("Recent files successfully saved to recent_files.json");
    Ok(())
}

/// Function to retrieve the most recently opened files from recent.json
#[tauri::command]
fn get_recent_data() -> Result<Vec<(i32, Data)>, String> {
    let file_path = config_dir().unwrap().join("recent_files.json");

    // If the file doesn't exist, return an empty vector
    if !file_path.exists() {
        println!("recent_files.json not found, returning an empty vector.");
        return Ok(Vec::new());
    }

    // Open the file and create a buffered reader
    let file = File::open(file_path)
        .map_err(|err| format!("Failed to open recent_files.json: {}", err))?;

    let reader = BufReader::new(file);

    // Deserialize the JSON data into a HashMap<i32, Data>
    let data: HashMap<i32, Data> = serde_json::from_reader(reader)
        .map_err(|err| format!("Failed to parse recent_files.json: {}", err))?;

    // Convert the HashMap into a Vec<(i32, Data)>
    let vec_data: Vec<(i32, Data)> = data.into_iter().collect();

    println!("Recent files successfully loaded and converted from recent_files.json");
    Ok(vec_data)
}




fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            setup_file_check,
            save_root_folder,
            save_file_extensions,
            load_setup,
            search_files,
            process_recent,
            get_recent_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
