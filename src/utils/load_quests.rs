use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use eoplus::{parse_quest, Quest};
use glob::glob;

pub fn load_quests() -> HashMap<i32, Quest> {
    let mut quests = HashMap::new();

    for entry in glob("quests/*.eqf").unwrap() {
        match entry {
            Ok(path) => match load_quest(path) {
                Some((id, quest)) => {
                    let _ = quests.insert(id, quest);
                }
                None => continue,
            },
            Err(e) => {
                error!("Failed to parse entry: {}", e);
                continue;
            }
        }
    }

    quests
}

fn load_quest(path: PathBuf) -> Option<(i32, Quest)> {
    let id = match path.file_name() {
        Some(name) => match name.to_str().to_owned() {
            Some(name) => match name.split('.').collect::<Vec<&str>>()[0].parse::<i32>() {
                Ok(id) => id,
                Err(e) => {
                    error!("Failed to parse id: {}", e);
                    return None;
                }
            },
            None => return None,
        },
        None => return None,
    };

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to load file: {}", e);
            return None;
        }
    };

    let length = match file.metadata() {
        Ok(metadata) => metadata.len() as usize,
        Err(e) => {
            error!("Failed to get file metadata: {}", e);
            return None;
        }
    };

    let mut buf: Vec<u8> = Vec::with_capacity(length);

    if let Err(e) = file.read_to_end(&mut buf) {
        error!("Failed to read file: {}", e);
        return None;
    }

    let input = String::from_utf8_lossy(&buf);

    match parse_quest(&input) {
        Ok(quest) => Some((id, quest)),
        Err(e) => {
            error!("Failed to parse quest: {}", e);
            None
        }
    }
}
