use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Stat {
    pub name: String,
    pub stat_type: String,
    pub type_value: String,
    pub using: String,
    pub data: HashMap<String, String>,
}

pub fn parse_stat_file(path: &str) -> Vec<Stat> {
    let f = File::open(path.clone()).expect("file not found");
    let reader = BufReader::new(f);

    let mut stats: Vec<Stat> = vec![];

    let mut current_stat = Stat {
        name: "".to_string(),
        stat_type: "".to_string(),
        type_value: "".to_string(),
        using: "".to_string(),
        data: HashMap::new(),
    };

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("new ") {
                let mut split_line = line.split_whitespace().collect::<Vec<&str>>();

                current_stat.name = split_line
                    .get(2)
                    .unwrap()
                    .replace("\"", "")
                    .to_string();

                current_stat.stat_type = split_line
                    .get(1)
                    .unwrap()
                    .to_string();

                current_stat.type_value = "".to_string();
                current_stat.using = "".to_string();
                current_stat.data = HashMap::new();

            } else if line.starts_with("data ") {
                let line = line.clone().replace("data \"", "");

                let data_entry = line
                    .split("\" \"")
                    .collect::<Vec<&str>>();

                current_stat.data.insert(
                    data_entry.get(0).unwrap().to_string(),
                    data_entry.get(1).unwrap().replace("\"", "").to_string(),
                );
            } else if line.starts_with("type ") {
                let line = line
                    .clone()
                    .replace("type \"", "")
                    .replace("\"", "");

                current_stat.type_value = line;
            } else if line.starts_with("using ") {
                let line = line
                    .clone()
                    .replace("using \"", "")
                    .replace("\"", "");

                current_stat.using = line;
            } else if line == "" {
                stats.push(current_stat.clone());
            }
        }
    }

    stats.push(current_stat.clone());
    return stats;
}