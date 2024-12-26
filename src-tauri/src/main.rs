// TODO:
// - Fix Indexation Time
// - Add The Scoring system for favorite extension types

//// Imports
use dirs::{audio_dir, desktop_dir, document_dir, download_dir, picture_dir, video_dir};
use num_cpus;
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::env::consts::OS as OS_TYPE;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime};
use tauri::api::path::config_dir;
use tauri::{Manager, Window};
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;

//// Constants
const MINIMUM_SCORE: i16 = 20;
const SKIP_DIRECTORY: &str = "Library"; // Directory to skip
const OS: &str = OS_TYPE;
const DEPTH_STOP: usize = 20;
const FILE_INDEX: &str = "file_index.json";
const SETUP_FILE: &str = "setup_file.json";
const EXTENSIONS_INDEX: &str = "extensions_index.json";

//// Global Variables
static ROOT_FOLDER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
static EXTENSIONS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static IN_MEMORY_INDEX: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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

#[derive(Serialize, Deserialize)]
struct SetupData {
    valid: bool,
    key_folders: HashMap<String, HashMap<String, String>>,
    recent_files: Vec<(i32, (String, String))>,
}

//// Global Variables Getter and Setters
// Function to set the value of ROOT_FOLDER
async fn set_root_folder(root_folder: String) -> Result<(), String> {
    let mut folder = ROOT_FOLDER.lock().await;
    *folder = root_folder;
    Ok(())
}

// Helper function to set EXTENSIONS using Lazy<Mutex>
async fn set_extensions(extensions: Vec<String>) -> Result<(), String> {
    let mut ext = EXTENSIONS.lock().await;
    *ext = extensions;
    Ok(())
}

// Function to get the current value of ROOT_FOLDER
async fn get_root_folder() -> Result<String, String> {
    let folder = ROOT_FOLDER.lock().await;
    Ok(folder.clone())
}

// Function to get the current value of EXTENSIONS
async fn get_extensions() -> Result<Vec<String>, String> {
    let ext = EXTENSIONS.lock().await;
    Ok(ext.clone())
}

//// Setup Function
/// Function to check if setup file exists
async fn setup_file_check() -> Result<bool, String> {
    // Get the path to the setup file
    let path = config_dir().unwrap().join(SETUP_FILE);

    // Check if the file exists
    let exists = Path::new(&path).exists();

    Ok(exists)
}

/// Function to detect OS and return it
async fn detect_os() -> Result<String, String> {
    // Simply return the OS constant
    Ok(OS.to_string())
}

async fn detect_key_folders() -> HashMap<String, HashMap<String, String>> {
    let mut folders = HashMap::new();

    // Helper function to get the directory contents
    async fn get_directory_contents(path: &PathBuf) -> HashMap<String, String> {
        let mut contents = HashMap::new();

        // Add the folder's own path with a special key
        contents.insert(".folder_path".to_string(), path.to_string_lossy().to_string());

        if let Ok(mut entries) = fs::read_dir(path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let entry_path = entry.path();
                let entry_name = entry
                    .file_name()
                    .into_string()
                    .unwrap_or_else(|_| "Invalid UTF-8".to_string());
                contents.insert(entry_name, entry_path.to_string_lossy().to_string());
            }
        }
        contents
    }

    // Add key folders and their contents
    if let Some(path) = desktop_dir() {
        let contents = get_directory_contents(&path).await;
        folders.insert("Desktop".to_string(), contents);
    }
    if let Some(path) = document_dir() {
        let contents = get_directory_contents(&path).await;
        folders.insert("Documents".to_string(), contents);
    }
    if let Some(path) = download_dir() {
        let contents = get_directory_contents(&path).await;
        folders.insert("Downloads".to_string(), contents);
    }
    if let Some(path) = picture_dir() {
        let contents = get_directory_contents(&path).await;
        folders.insert("Pictures".to_string(), contents);
    }
    if let Some(path) = video_dir() {
        let contents = get_directory_contents(&path).await;
        folders.insert("Videos".to_string(), contents);
    }
    if let Some(path) = audio_dir() {
        let contents = get_directory_contents(&path).await;
        folders.insert("Music".to_string(), contents);
    }

    folders
}

/// Function to create and save setup_file.json
#[tauri::command]
async fn save_setup_file(
    window: Window,
    root_folder: String,
    extensions: Vec<String>,
) -> Result<(), String> {
    // Get the path to the setup file
    let path: PathBuf = config_dir()
        .ok_or("Failed to retrieve config directory")?
        .join(SETUP_FILE);

    // Define a default JSON structure for the setup file
    let mut setup = json!({
        "os": detect_os().await.unwrap_or_else(|_| "unknown".to_string()),
        "root_folder": "",
        "file_extensions": [],
        "key_folders": {}
    });

    // If the setup file exists, attempt to read and parse it
    if let Ok(existing_content) = fs::read_to_string(&path).await {
        if let Ok(existing_setup) = serde_json::from_str::<serde_json::Value>(&existing_content) {
            setup = existing_setup;
        }
    }

    // Update the root folder and extensions fields
    set_root_folder(root_folder.clone()).await?;
    setup["root_folder"] = serde_json::Value::String(root_folder);

    set_extensions(extensions.clone()).await?;
    setup["file_extensions"] = serde_json::Value::Array(
        extensions
            .into_iter()
            .map(serde_json::Value::String)
            .collect(),
    );

    // Dynamically detect and populate key folders
    let key_folders = detect_key_folders().await;
    setup["key_folders"] = serde_json::Value::Object(
        key_folders
            .into_iter()
            .map(|(folder_name, folder_contents)| {
                (
                    folder_name,
                    serde_json::Value::Object(
                        folder_contents
                            .into_iter()
                            .map(|(file_name, file_path)| {
                                (file_name, serde_json::Value::String(file_path))
                            })
                            .collect(),
                    ),
                )
            })
            .collect(),
    );

    // Serialize the updated setup structure to JSON
    let content = serde_json::to_string_pretty(&setup)
        .map_err(|e| format!("Failed to serialize setup file: {}", e))?;

    // Write the updated JSON content to the setup file
    fs::write(&path, content)
        .await
        .map_err(|e| format!("Failed to write setup file: {}", e))?;

    // Start indexing the files using the root folder
    let config_dir = config_dir().ok_or("Failed to retrieve config directory")?;
    let file_index_path = config_dir.join(FILE_INDEX);
    create_index(window, file_index_path).await?;

    Ok(())
}

/// Function to load the setup_file.json
async fn load_setup() -> Result<HashMap<String, HashMap<String, String>>, String> {
    // Get the path to the setup file
    let path: PathBuf = config_dir()
        .ok_or("Failed to retrieve config directory")?
        .join(SETUP_FILE);

    // Attempt to read the setup file
    let content = match fs::read_to_string(&path).await {
        Ok(data) => data,
        Err(_) => {
            // If the file doesn't exist, return an empty HashMap
            return Ok(HashMap::new());
        }
    };

    // Parse the JSON content
    let setup: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse setup file: {}", e))?;

    // Load the root folder if available
    if let Some(root_folder) = setup.get("root_folder").and_then(|v| v.as_str()) {
        set_root_folder(root_folder.to_string()).await?;
    }

    // Load the extensions if available
    if let Some(extensions) = setup.get("file_extensions").and_then(|v| v.as_array()) {
        let extensions_vec: Vec<String> = extensions
            .iter()
            .filter_map(|ext| ext.as_str().map(|s| s.to_string()))
            .collect();
        set_extensions(extensions_vec).await?;
    }

    // Load the key folders if available
    if let Some(key_folders) = setup.get("key_folders").and_then(|v| v.as_object()) {
        let key_folders_map: HashMap<String, HashMap<String, String>> = key_folders
            .iter()
            .map(|(folder_name, folder_contents)| {
                let folder_map = folder_contents
                    .as_object()
                    .unwrap_or(&serde_json::Map::new())
                    .iter()
                    .map(|(file_name, file_path)| {
                        (
                            file_name.clone(),
                            file_path.as_str().unwrap_or("").to_string(),
                        )
                    })
                    .collect();
                (folder_name.clone(), folder_map)
            })
            .collect();

        return Ok(key_folders_map);
    }

    Ok(HashMap::new())
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
async fn save_index(index: &FileIndex, index_path: &Path) -> Result<(), String> {
    let serialized_data = serde_json::to_string_pretty(index)
        .map_err(|err| format!("Failed to serialize index: {}", err))?;

    tokio::fs::write(index_path, serialized_data)
        .await
        .map_err(|err| format!("Failed to write index to file: {}", err))?;

    println!("Index successfully saved.");
    Ok(())
}

/// Loads the index from a file
async fn load_index(index_path: &Path) -> Result<(), String> {
    let file_content = match tokio::fs::read_to_string(index_path).await {
        Ok(content) => content,
        Err(_) => {
            println!("Index file not found or unreadable. Initializing an empty index.");
            return Ok(()); // Return early with an empty index
        }
    };

    let index: FileIndex = serde_json::from_str(&file_content).unwrap_or_else(|_| FileIndex {
        files: HashMap::new(),
    });

    let mut in_memory_index = IN_MEMORY_INDEX.lock().await;
    in_memory_index.clear(); // Clear existing entries

    for (file_name, details) in index.files.iter() {
        in_memory_index.insert(file_name.clone(), details.file_path.clone());
    }

    println!("Index successfully loaded into memory.");
    Ok(())
}

/// Recursively indexes directories and subdirectories
async fn index_files(
    path: &Path,
    file_index: &Arc<tokio::sync::Mutex<FileIndex>>,
    extensions_index: &Arc<tokio::sync::Mutex<FileIndex>>,
) {
    let extensions = get_extensions().await.unwrap_or_else(|_| Vec::new());
    let queue = Arc::new(tokio::sync::Mutex::new(VecDeque::new()));
    queue.lock().await.push_back((path.to_path_buf(), 0));

    loop {
        let (current_path, depth) = {
            // Scope the lock to avoid holding it throughout the iteration
            let mut queue_lock = queue.lock().await;
            match queue_lock.pop_front() {
                Some(item) => item,
                None => break, // Exit the loop if the queue is empty
            }
        };

        if depth > DEPTH_STOP {
            continue; // Skip if depth exceeds the allowed depth
        }

        if current_path.ends_with(SKIP_DIRECTORY) {
            continue; // Skip specific directories
        }

        match tokio::fs::read_dir(&current_path).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let entry_path = entry.path();
                    let file_name = match entry.file_name().into_string() {
                        Ok(name) => name,
                        Err(_) => {
                            println!("Encountered a non-UTF-8 file name. Skipping.");
                            continue;
                        }
                    };

                    let file_path = entry_path.display().to_string();

                    match entry.metadata().await {
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

                            let file_extension = entry_path
                                .extension()
                                .and_then(|ext| ext.to_str())
                                .map_or("".to_string(), |ext| ext.to_string());

                            let details = FileDetails {
                                file_path: file_path.clone(),
                                file_size,
                                file_type: file_type.clone(),
                                creation_date,
                                file_extension: file_extension.clone(),
                            };

                            {
                                let mut file_index_lock = file_index.lock().await;
                                file_index_lock
                                    .files
                                    .insert(file_name.clone(), details.clone());
                            }

                            // Uncomment this block if filtering by extensions is needed
                            // if !extensions.is_empty() && extensions.contains(&file_extension) {
                            //     let mut extensions_index_lock = extensions_index.lock().await;
                            //     extensions_index_lock
                            //         .files
                            //         .insert(file_name.clone(), details);
                            // }

                            if entry_path.is_dir() {
                                let mut queue_lock = queue.lock().await;
                                queue_lock.push_back((entry_path, depth + 1));
                            }
                        }
                        Err(e) => {
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
            Err(e) => {
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
async fn startup(window: Window) -> Result<SetupData, String> {
    if !setup_file_check().await.unwrap_or(false) {
        println!("Setup file not found. Prompting user...");
        Ok(SetupData {
            valid: false,
            key_folders: detect_key_folders().await,
            recent_files: Vec::new(),
        })
    } else {
        println!("Setup file found. Loading details...");
        let key_folders = load_setup().await?;
        let recent_files = get_recent_data().await?;

        let config_dir = config_dir().unwrap();
        let file_index_path = config_dir.join(FILE_INDEX);

        // Check if the main file index exists
        if file_index_path.exists() {
            // Load the file index into memory if it exists
            load_index(&file_index_path).await?;
            window.emit("index-found", {}).unwrap();
        } else {
            create_index(window, file_index_path).await?;
        }

        Ok(SetupData {
            valid: true,
            key_folders,
            recent_files,
        })
    }
}

async fn create_index(window: Window, file_index_path: PathBuf) -> Result<(), String> {
    window.emit("indexing-started", {}).unwrap();

    let root_folder = match get_root_folder().await {
        Ok(folder) => folder,
        Err(e) => {
            println!("Error getting root folder: {}", e);
            return Err(e);
        }
    };

    let start_path = Path::new(&root_folder);
    let new_file_index = Arc::new(Mutex::new(FileIndex {
        files: HashMap::new(),
    }));
    let new_extensions_index = Arc::new(Mutex::new(FileIndex {
        files: HashMap::new(),
    }));

    // Clone the window and file_index_path to move into the async task
    let window_clone = window.clone();
    let file_index_path_clone = file_index_path.clone();
    let start_path_clone = start_path.to_path_buf();

    // Spawn a new task to run the indexing in the background
    tauri::async_runtime::spawn(async move {
        println!("Indexing files in the background...");
        // Index all files in the root folder
        index_files(&start_path_clone, &new_file_index, &new_extensions_index).await;

        // Save the new indices to the specified paths
        let file_index = new_file_index.lock().await;
        if let Err(e) = save_index(&file_index, &file_index_path_clone).await {
            println!("Error saving index: {}", e);
        } else {
            window_clone.emit("indexing-completed", {}).unwrap();
        }
    });

    Ok(())
}

//// Search and Recent Export functions
/// Searches for files based on the query
#[tauri::command]
async fn search_files(query: String) -> Result<Vec<(String, String)>, String> {
    let start_time = Instant::now(); // Start the timer

    // Acquire a lock on the in-memory index
    let index_guard = IN_MEMORY_INDEX.lock().await;
    let index_snapshot = index_guard.clone(); // Clone the index for use outside the lock
    drop(index_guard); // Release the lock early

    // Convert the query to lowercase for case-insensitive comparison
    let query_lower = query.to_lowercase();

    // Determine if parallel processing is needed
    let results: Vec<(String, String)> = if index_snapshot.len() > 1000 {
        println!("Parallelizing search with Rayon...");
        println!("Index size: {}", index_snapshot.len());
        let num_cores = num_cpus::get();
        println!("Using {} cores for parallel processing.", num_cores);

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cores)
            .build()
            .map_err(|e| format!("Failed to build Rayon thread pool: {}", e))?;

        pool.install(|| {
            index_snapshot
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
        index_snapshot
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
async fn process_recent(data: Vec<(i32, (String, String))>) -> Result<(), String> {
    let data_map: HashMap<i32, (String, String)> = data.into_iter().collect();
    let file_path: PathBuf = config_dir().unwrap().join("recent_files.json");

    // Serialize file names and file paths into JSON
    let json_data = serde_json::to_string_pretty(&data_map)
        .map_err(|err| format!("Failed to serialize data: {}", err))?;

    // Write JSON data to recent_files.json
    fs::write(&file_path, json_data.as_bytes())
        .await
        .map_err(|err| format!("Failed to write data to recent_files.json: {}", err))?;

    println!("Recent files successfully saved to recent_files.json");
    Ok(())
}

/// Function to retrieve the most recently opened files from recent_files.json
#[tauri::command]
async fn get_recent_data() -> Result<Vec<(i32, (String, String))>, String> {
    let file_path = config_dir().unwrap().join("recent_files.json");

    // If the file doesn't exist, return an empty vector
    if !file_path.exists() {
        println!("recent_files.json not found, returning an empty vector.");
        return Ok(Vec::new());
    }

    // Read the file asynchronously
    let mut file = fs::File::open(&file_path)
        .await
        .map_err(|err| format!("Failed to open recent_files.json: {}", err))?;

    let mut json_content = String::new();
    file.read_to_string(&mut json_content)
        .await
        .map_err(|err| format!("Failed to read recent_files.json: {}", err))?;

    // Deserialize JSON data into HashMap<i32, (String, String)>
    let data: HashMap<i32, (String, String)> = serde_json::from_str(&json_content)
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

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            startup,
            save_setup_file,
            search_files,
            process_recent,
            get_recent_data,
            open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
