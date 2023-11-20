use serde::Deserialize;

use crate::{settings, semesters::Semester};
use crate::get_authed;

#[derive(Deserialize, Clone)]
pub struct Course {
    pub course_id: String,
    pub title: String,
    pub start_semester: String,
    pub end_semester: Option<String>
}

#[derive(Deserialize)]
pub struct CourseCollection {
    #[serde(flatten)]
    pub courses: std::collections::BTreeMap<String, Course>,
}

#[derive(Deserialize)]
pub struct Courses {
    pub collection: CourseCollection
}

pub async fn get_courses_by_semester(settings:&settings::Settings, semester: Semester) -> Result<Vec<Course>, reqwest::Error> {        
    let user_id = "df10bd4a324818b5acaeece264767e1a";
    let limit = 150;

    let url = format!(
        "https://elearning.uni-oldenburg.de/api.php/user/{}/courses?limit={}",
        user_id, limit
    );

    let courses = get_authed::<Courses>(url, settings).await?;

    let current_semester_courses: Vec<Course> = courses
    .collection
    .courses
    .into_iter()
    .filter(|(_, course)| {
        course.start_semester.split('/').last().unwrap() == semester.id
            || course.end_semester.as_ref().unwrap_or(&("asd".to_owned())).split('/').last().unwrap() == semester.id
    })
    .map(|(_, course)| course)
    .collect();

    Ok(current_semester_courses)
}