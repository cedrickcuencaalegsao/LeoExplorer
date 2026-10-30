use std::{path::{Path, PathBuf}};
use shared::FileEntry;
use std::fs::read_dir;

pub fn open_roots(args: PathBuf) -> Result<Vec<FileEntry>, String>{
    let path_str = args.to_str().unwrap_or("default");
    let formatted_path = format_path(path_str);
    let entries = read_drive_content(&formatted_path);
    Ok(entries)
}

fn format_path(args: &str) -> PathBuf{
    let s = args.trim();
    if cfg!(target_os = "windows"){
        let letter = s.trim_end_matches(":");
        if letter.len() == 1 && letter.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false){
            PathBuf::from(format!("{}:\\", letter.to_uppercase()))
        } else if s.contains('\\') || s.contains("/"){
            PathBuf::from(s)
        } else {
            PathBuf::from(format!("{}:\\", letter.to_uppercase()))
        }
    } else {
        PathBuf::from(if s.is_empty() {"/"} else {s})
    }
}

fn read_drive_content(args: &Path) -> Vec<FileEntry>{
    let mut entries = Vec::with_capacity(100);

    let read_dir = match read_dir(args){
        Ok(dir)=> dir,
        Err(_) => return entries,
    };

    for entry in read_dir.flatten(){
        let entry_path = entry.path();
        let is_dir = match entry.metadata() {
            Ok(m) => m.is_dir(),
            Err(_) => continue,
        };
        let mut file_entry = FileEntry::from_path(&entry_path, is_dir);
        if is_dir{
            file_entry.children = Some(Vec::new());
        }
        entries.push(file_entry);
    }

    entries.sort_unstable_by(|a, b| match (a.is_dir, b.is_dir){
        (true, true) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    entries
}