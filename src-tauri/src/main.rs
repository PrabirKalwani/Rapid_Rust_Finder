// TODO:
// - Fix Indexation Time
// - Add The Scoring system for favorite extension types

//// Imports
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::env::consts::OS as OS_TYPE;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Instant, SystemTime};
use tauri::api::path::config_dir;
use tauri::Manager;

//// Constants
const MINIMUM_SCORE: i16 = 20;
const SKIP_DIRECTORY: &str = "Library"; // Directory to skip
const OS: &str = OS_TYPE;
const DEPTH_STOP: usize = 20;
const FILE_INDEX: &str = "file_index.json";
const EXTENSIONS_INDEX: &str = "extensions_index.json";

//// Global Variables
static ROOT_FOLDER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static EXTENSIONS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static IN_MEMORY_INDEX: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

//// Data Structures
/// Data structure to hold the index
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileDetails {
    file_path: String,
    file_size: u64,
    file_type: String,
    creation_date: Option<SystemTime>,
    file_extension: String,
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
    file_extension: String,
}

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

/// Function to create and save setup_file.json
#[tauri::command]
async fn save_setup_file(root_folder: String, extensions: Vec<String>) -> Result<(), String> {
    save_root_folder(root_folder).await?;
    save_file_extensions(extensions).await?;
    Ok(())
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
        extensions
            .into_iter()
            .map(serde_json::Value::String)
            .collect(),
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
fn load_index(index_path: &Path) {
    // Load the index from the file and deserialize it
    let index: FileIndex = match fs::read_to_string(index_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| FileIndex { files: HashMap::new() }),
        Err(_) => FileIndex { files: HashMap::new() },
    };

    // Acquire a lock on the in-memory index and clear any existing entries
    let mut in_memory_index = IN_MEMORY_INDEX.lock().unwrap();
    in_memory_index.clear();  // Clear existing entries if any

    // Insert only file names and their paths into the in-memory hashmap
    for (file_name, details) in index.files.iter() {
        in_memory_index.insert(file_name.clone(), details.file_path.clone()); // Store file name and path only
    }

    println!("Index loaded with file names and paths.");
}

/// Recursively indexes directories and subdirectories
fn index_files(path: &Path, file_index: &mut FileIndex, extensions_index: &mut FileIndex) {
    let extensions = get_extensions().unwrap_or_default(); // Get allowed extensions
    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
    queue.push_back((path.to_path_buf(), 0));

    while let Some((current_path, depth)) = queue.pop_front() {
        // Skip if depth exceeds the allowed depth
        if depth > DEPTH_STOP {
            continue;
        }

        // Skip specific directories based on your custom logic
        if current_path.ends_with(SKIP_DIRECTORY) {
            continue;
        }

        match fs::read_dir(&current_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();

                        // Handle the possibility of non-UTF-8 file names
                        let file_name = match entry.file_name().into_string() {
                            Ok(name) => name,
                            Err(_) => {
                                println!("Encountered a non-UTF-8 file name. Skipping.");
                                continue; // Skip this file and continue with the next entry
                            }
                        };

                        let file_path = entry_path.display().to_string();

                        match entry.metadata() {
                            Ok(metadata) => {
                                let file_size = metadata.len();
                                let file_type = if metadata.is_dir() {
                                    "directory".to_string()
                                } else if metadata.is_file() {
                                    "file".to_string()
                                } else {
                                    "unknown".to_string()
                                };
                                let creation_date = metadata.created().ok();

                                // Extract the file extension
                                let file_extension = entry_path
                                    .extension()
                                    .and_then(|ext| ext.to_str())
                                    .map_or("".to_string(), |ext| ext.to_string());

                                // Populate the general file index with all files
                                let details = FileDetails {
                                    file_path: file_path.clone(),
                                    file_size,
                                    file_type: file_type.clone(),
                                    creation_date,
                                    file_extension: file_extension.clone(),
                                };
                                file_index.files.insert(file_name.clone(), details.clone());

                                // If the file extension matches the allowed list, populate the extensions index
                                if !extensions.is_empty() && extensions.contains(&file_extension) {
                                    extensions_index.files.insert(file_name.clone(), details);
                                }

                                // If it's a directory, enqueue it for further indexing
                                if entry_path.is_dir() {
                                    queue.push_back((entry_path, depth + 1));
                                }
                            }
                            Err(e) => {
                                // Handle permission denied errors
                                if e.kind() == std::io::ErrorKind::PermissionDenied {
                                    println!(
                                        "Permission denied for accessing: {}. Skipping.",
                                        file_path
                                    );
                                } else {
                                    println!("Error getting metadata for {}: {}", file_path, e);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Handle permission denied errors for directories
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    println!(
                        "Permission denied for directory: {}. Skipping.",
                        current_path.display()
                    );
                } else {
                    println!("Error reading directory {}: {}", current_path.display(), e);
                }
            }
        }
    }
}

//// Startup function
#[tauri::command]
fn startup() {
    let config_dir = config_dir().unwrap();
    let file_index_path = config_dir.join(FILE_INDEX);
    let extensions_index_path = config_dir.join(EXTENSIONS_INDEX);

    // Check if the main file index exists
    if file_index_path.exists() {
        // Load the file index into memory if it exists
        println!("Loading existing index into memory...");
        load_index(&file_index_path);
        println!("Index loaded into memory successfully.");
    } else {
        // If the index doesn't exist, create new file and extensions indices
        println!("Creating a new file index and extensions index...");

        let root_folder = match get_root_folder() {
            Ok(folder) => folder,
            Err(e) => {
                println!("Error getting root folder: {}", e);
                return;
            }
        };

        let start_path = Path::new(&root_folder);
        let mut new_file_index = FileIndex {
            files: HashMap::new(),
        };
        let mut new_extensions_index = FileIndex {
            files: HashMap::new(),
        };

        // Index all files in the root folder
        index_files(start_path, &mut new_file_index, &mut new_extensions_index);
        
        // Save the new indices to the specified paths
        save_index(&new_file_index, &file_index_path);
        println!("New indexes created and saved.");

        // Load the newly created file index into memory
        load_index(&file_index_path);
        println!("New index loaded into memory successfully.");
    }
}

//// Search and Recent Export functions
/// Searches for files based on the query
#[tauri::command]
fn search_files(query: String) -> Result<Vec<(String, String)>, String> {
    let start_time = Instant::now(); // Start the timer

    // Acquire a lock on the in-memory index
    let index = IN_MEMORY_INDEX.lock().map_err(|e| e.to_string())?;

    // Convert the query to lowercase for case-insensitive comparison
    let query_lower = query.to_lowercase();

    // Determine if parallel processing is needed
    let results: Vec<(String, String)> = if index.len() > 1000 {
        println!("Parallelizing search with Rayon...");
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();

        pool.install(|| {
            index
                .par_iter()
                .filter_map(|(file_name, file_path)| {
                    let cleaned_file_name = Path::new(file_name)
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("");

                    // Convert the filename to lowercase for case-insensitive comparison
                    let cleaned_file_name_lower = cleaned_file_name.to_lowercase();

                    // Calculate the similarity score
                    let score = score_filename(&cleaned_file_name_lower, &query_lower);
                    if score >= MINIMUM_SCORE {
                        Some((file_name.clone(), file_path.clone())) // Return matching file names and paths
                    } else {
                        None // Filter out non-matching results
                    }
                })
                .collect()
        })
    } else {
        // For small datasets, use sequential iteration
        println!("Using sequential search...");
        index
            .iter()
            .filter_map(|(file_name, file_path)| {
                let cleaned_file_name = Path::new(file_name)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("");

                // Convert the filename to lowercase for case-insensitive comparison
                let cleaned_file_name_lower = cleaned_file_name.to_lowercase();

                // Calculate the similarity score
                let score = score_filename(&cleaned_file_name_lower, &query_lower);
                if score >= MINIMUM_SCORE {
                    Some((file_name.clone(), file_path.clone())) // Return matching file names and paths
                } else {
                    None // Filter out non-matching results
                }
            })
            .collect()
    };

    let duration = start_time.elapsed(); // Measure the elapsed time
    println!("Search completed in {:?}", duration);

    Ok(results)
}


/// Function to save the most recently opened files into recent_files.json
#[tauri::command]
fn process_recent(data: Vec<(i32, (String, String))>) -> Result<(), String> {
    let data_map: HashMap<i32, (String, String)> = data.into_iter().collect();
    
    let file_path: PathBuf = config_dir().unwrap().join("recent_files.json");

    // Serialize file names and file paths into JSON
    let json_data = serde_json::to_string_pretty(&data_map)
        .map_err(|err| format!("Failed to serialize data: {}", err))?;

    // Write JSON data to recent_files.json
    let mut file = File::create(file_path)
        .map_err(|err| format!("Failed to create recent_files.json: {}", err))?;
    file.write_all(json_data.as_bytes())
        .map_err(|err| format!("Failed to write data to recent_files.json: {}", err))?;

    println!("Recent files successfully saved to recent_files.json");
    Ok(())
}

/// Function to retrieve the most recently opened files from recent_files.json
#[tauri::command]
fn get_recent_data() -> Result<Vec<(i32, (String, String))>, String> {
    let file_path = config_dir().unwrap().join("recent_files.json");

    // If the file doesn't exist, return an empty vector
    if !file_path.exists() {
        println!("recent_files.json not found, returning an empty vector.");
        return Ok(Vec::new());
    }

    // Open recent_files.json and deserialize it to retrieve file names and paths
    let file = File::open(file_path)
        .map_err(|err| format!("Failed to open recent_files.json: {}", err))?;
    let reader = BufReader::new(file);

    // Deserialize JSON data into HashMap<i32, (String, String)>
    let data: HashMap<i32, (String, String)> = serde_json::from_reader(reader)
        .map_err(|err| format!("Failed to parse recent_files.json: {}", err))?;

    // Convert HashMap to Vec<(i32, (String, String))>
    let vec_data: Vec<(i32, (String, String))> = data.into_iter().collect();

    println!("Recent files successfully loaded and converted from recent_files.json");
    Ok(vec_data)
}

#[tauri::command]
fn open_file(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    let shell_scope = app_handle.shell_scope();
    tauri::api::shell::open(&shell_scope, path, None)
        .map_err(|e| format!("Failed to open file: {}", e))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            startup,
            save_setup_file,
            setup_file_check,
            save_root_folder,
            save_file_extensions,
            load_setup,
            search_files,
            process_recent,
            get_recent_data, 
            open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
