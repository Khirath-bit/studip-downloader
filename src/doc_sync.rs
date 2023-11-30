use std::{path::Path, fs, io::Write};
use bytes::Bytes;
use serde::Deserialize;
use async_recursion::async_recursion;
use filetime_creation::{FileTime, set_file_ctime};
use crate::{courses::Course, settings::Settings};
use crate::get_authed;
use crate::get_client;

type Changelog = String;

#[derive(Deserialize)]
struct UniFile {
    pub id: String,
    pub name: String,
    pub is_downloadable: bool,
    pub mime_type: String,
    pub chdate: i64
}

#[derive(Deserialize)]
struct UniDir{
    pub id: String,
    pub name: String,
    pub subfolders: Option<Vec<UniDir>>,
    pub file_refs: Option<Vec<UniFile>>,
    pub is_visible: bool,
    pub path: Option<String>
}

fn clear_special_chars(input: &str) -> String{
    input
        .replace(['>', '<', ':', '\"', '\'', '/', '\\', '|', '?', '*'], "")
}

async fn download_file(id:String, settings:&Settings) -> Result<Bytes, reqwest::Error> {
    let url = format!(
        "https://elearning.uni-oldenburg.de/api.php/file/{}/download",
        id
    );

    get_client()
        .get(url)
        .basic_auth(&settings.api_username, Some(&settings.api_password))
        .send()
        .await?
        .bytes()
        .await
}

impl Course{
    pub async fn sync(self, settings:&Settings) -> Result<Changelog, reqwest::Error> {
        let mut changelog = String::new();
        let p = &(settings.course_directory_path.clone() + "\\" + &(clear_special_chars(&self.title)));
        let dir = Path::new(p);

        if !dir.exists() {
            if fs::create_dir(dir).is_err() {
                return Ok("Failed to create directory, cancelling sync of this course\n".to_owned());
            }
            changelog += "      CREATED: course folder directory \n";
        }

        let top_folder_url = format!(
            "https://elearning.uni-oldenburg.de/api.php/course/{}/top_folder",
            self.course_id
        );

        let mut top_folder = get_authed::<UniDir>(top_folder_url, settings).await?;

        top_folder.path = Some(dir.to_str().unwrap().to_owned());

        Course::sync_dir(top_folder, &mut changelog, settings).await
    }
    #[async_recursion]
    async fn sync_dir(folder:UniDir, cl: &mut Changelog, settings:&Settings) -> Result<Changelog, reqwest::Error> {
        let top_folder_url = format!(
            "https://elearning.uni-oldenburg.de/api.php/folder/{}",
            folder.id
        );

        let top_folder = get_authed::<UniDir>(top_folder_url, settings).await?;

        //1. Sync files
        for file in top_folder.file_refs.unwrap_or(Vec::new()) {
            if !file.is_downloadable || (!settings.download_videos && file.mime_type.starts_with("video")) {
                continue;
            }
            let sanitized_name = clear_special_chars(&file.name);
            let fp = &(folder.path.clone().unwrap() + "\\" + &sanitized_name);
            let f_path = Path::new(fp);

            if !f_path.exists() || FileTime::from(fs::metadata(f_path).unwrap().created().unwrap()) != FileTime::from_unix_time(file.chdate, 0) {
                let file_data = download_file(file.id, settings).await?;
                let mut f: fs::File = fs::File::create(f_path).unwrap();
                if set_file_ctime(f_path, FileTime::from_unix_time(file.chdate, 0)).is_err() {
                    cl.push_str(&("     ERROR: Failed to download ".to_owned() + &sanitized_name + " because created date couldnt be set\n"));
                }

                if f.write_all(&file_data).is_err() {
                    cl.push_str(&("     ERROR: Failed to download ".to_owned() + &sanitized_name + "\n"));
                    fs::remove_file(f_path).unwrap();
                }

                cl.push_str(&("     DOWNLOADED: File ".to_owned() + &sanitized_name + " downloaded to " + f_path.to_str().unwrap() + "\n"));
            }
        }

        //2. sync subfolders
        for mut dir in top_folder.subfolders.unwrap_or(Vec::new()) {
            if !dir.is_visible{
                continue;
            }
            let sanitized_name = clear_special_chars(&dir.name);
            let p = &(folder.path.clone().unwrap() + "\\" + &sanitized_name);
            let dir_p = Path::new(p);
            dir.path = Some(p.to_owned());
            if !dir_p.exists() {
                if fs::create_dir(dir_p).is_err() {
                    return Ok("Failed to create directory, cancelling sync of this course\n".to_owned());
                }
                cl.push_str(&("     CREATED: course sub folder directory ".to_owned() + &sanitized_name + " at " + &dir.path.clone().unwrap() + "\n"));
            }

            Course::sync_dir(dir, cl, settings).await?;
        }

        Ok(cl.to_string())
    }
}