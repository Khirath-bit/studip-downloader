use std::time::SystemTime;
use serde::Deserialize;

use crate::settings;
use crate::get_authed;

#[derive(Deserialize)]
pub struct Semester {
    pub id: String,
    pub token: String,
    pub begin: u64,
    pub end: u64
}

#[derive(Deserialize)]
pub struct SemesterCollection {
    #[serde(flatten)]
    pub semesters: std::collections::BTreeMap<String, Semester>,
}

#[derive(Deserialize)]
pub struct Semesters {
    pub collection: SemesterCollection
}

pub async fn get_current_semester(settings:&settings::Settings) -> Result<Semester, reqwest::Error> {
    let semester = get_authed::<Semesters>
        (settings.university_base_api_url.clone() + "/semesters", settings).await?;

    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let sem = semester.collection.semesters.into_iter()
    .find(|s| s.1.begin < now && s.1.end > now).unwrap();

    Ok(sem.1)
}