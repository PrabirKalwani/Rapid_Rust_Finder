use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::time::Instant;
use serde_json;

const MINIMUM_SCORE: i16 = 20;
const HIGHLIGHT_COLOR_START: &str = "\x1b[1;32m"; 
const HIGHLIGHT_COLOR_END: &str = "\x1b[0m";      
const SKIP_DIRECTORY: &str = "Library";          


fn highlight_match(filename: &str, query: &str) -> String {
    let mut result = String::new();
    let mut last_index = 0;

    for (start, matched_str) in filename.match_indices(query) {
        let end = start + matched_str.len();
        result.push_str(&filename[last_index..start]);
        result.push_str(HIGHLIGHT_COLOR_START);     
        result.push_str(&filename[start..end]);
        result.push_str(HIGHLIGHT_COLOR_END);       
        last_index = end;
    }

    
    result.push_str(&filename[last_index..]);
    result
}


fn score_filename(matcher: &SkimMatcherV2, filename: &str, query: &str) -> i16 {
    if filename == query {
        return 1000; 
    }
    matcher.fuzzy_match(filename, query).unwrap_or(0) as i16
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

fn main() {
    let index_path = Path::new("file_index.json");

    let index = if index_path.exists() {
        
        println!("Loading existing index...");
        load_index(index_path)
    } else {
        
        println!("Creating new index...");
        let start_path = Path::new("/Users/prabirkalwani");
        let mut new_index = FileIndex { files: HashMap::new() };
        index_recursive(start_path, &mut new_index);
        save_index(&new_index, index_path);
        new_index
    };

    let matcher = SkimMatcherV2::default().smart_case();

    println!("Enter the starting letters of the file name (type 'exit' to quit):");

    let mut input = String::new();
    loop {
        
        input.clear();

        
        let bytes_read = io::stdin().read_line(&mut input).unwrap();
        if bytes_read == 0 {
            
            break;
        }

        let query = input.trim();
        if query == "exit" {
            break;
        }

        let start_time = Instant::now();
        let mut results = Vec::new();

        
        for (filename, file_path) in &index.files {
            let cleaned_filename = Path::new(filename)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("");

            let score = score_filename(&matcher, cleaned_filename, query);
            if score >= MINIMUM_SCORE {
                let highlighted = highlight_match(filename, query);
                results.push((highlighted, file_path.clone()));
            }
        }

        
        println!("\nMatching files/directories:");
        for (highlighted_filename, file_path) in results {
            println!("{} -> {}", highlighted_filename, file_path);
        }

        let end_time = Instant::now();
        println!("\nElapsed time: {:?}", end_time - start_time);
    }
}

