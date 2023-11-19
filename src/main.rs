use settings::Settings;
use reqwest::{header::{HeaderMap, HeaderValue}, Client};
mod settings;
mod semesters;
mod courses;
mod doc_sync;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error>{
    let settings = Settings::load();

    println!("Downloaded course files will be saved to: {}", settings.course_directory_path);

    let semester = semesters::get_current_semester(get_client(), &settings).await?;

    println!("Current semester: {}", semester.token);

    let courses = courses::get_courses_by_semester(get_client(), &settings, semester).await?;

    println!("Courses this semester: ");

    for course in courses {
        print!("  {}..", course.title);
        let changelog = course.sync(get_client(), &settings).await?;
        println!("synced. \nChangelog: \n{}", changelog);
    }

    Ok(())
}

fn get_client() -> Client {
    let mut def_head = HeaderMap::new();
    def_head.insert("User-Agent", HeaderValue::from_static("UniSync/0.1"));

    reqwest::ClientBuilder::new()
    .default_headers(def_head)
    .build().unwrap()
}
