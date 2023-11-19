
use std::{fs::File, io::BufReader, time::Instant};

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub course_directory_path: String,
    pub university_base_api_url: String,
    pub user_id: String,
    pub api_password: String,
    pub api_username: String,
    pub download_period_cron: String
}

impl Settings{
    pub fn load() -> Settings{
        let now = Instant::now();

        let file = File::open(
            "./settings.json").unwrap();

        let settings: Settings = serde_json::from_reader(BufReader::new(file)).unwrap();

        println!("Settings loaded in 0.{}ms", now.elapsed().as_micros());

        settings
    }
}