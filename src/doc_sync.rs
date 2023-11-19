use std::{path::Path, fs, io::Write};
use bytes::Bytes;
use reqwest::Client;
use serde::Deserialize;
use async_recursion::async_recursion;

use crate::{courses::Course, settings::Settings};

type Changelog = String;

#[derive(Deserialize)]
struct UniFile {
    pub id: String,
    pub name: String,
    pub is_downloadable: bool
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

async fn download_file(id:String, client:&Client, settings:&Settings) -> Result<Bytes, reqwest::Error> {
    let url = format!(
        "https://elearning.uni-oldenburg.de/api.php/file/{}/download",
        id
    );

    client
        .get(url)
        .basic_auth(&settings.api_username, Some(&settings.api_password))
        .send()
        .await?
        .bytes()
        .await
}

impl Course{
    pub async fn sync(self, client:Client, settings:&Settings) -> Result<Changelog, reqwest::Error> {
        let mut changelog = String::new();
        let p = &(settings.course_directory_path.clone() + "\\" + &(clear_special_chars(&self.title)));
        let dir = Path::new(p);

        if !dir.exists() {
            if fs::create_dir(dir).is_err() {
                //TODO maybe change to err
                return Ok("Failed to create directory, cancelling sync of this course\n".to_owned());
            }
            changelog += "      CREATED: course folder directory \n";
        }

        let top_folder_url = format!(
            "https://elearning.uni-oldenburg.de/api.php/course/{}/top_folder",
            self.course_id
        );

        let mut top_folder = client
        .get(top_folder_url)
        .basic_auth(&settings.api_username, Some(&settings.api_password))
        .send()
        .await?
        .json::<UniDir>()
        .await?;

        top_folder.path = Some(dir.to_str().unwrap().to_owned());

        Course::sync_dir(top_folder, &mut changelog, &client, settings).await
    }
    #[async_recursion]
    async fn sync_dir(folder:UniDir, cl: &mut Changelog, client:&Client, settings:&Settings) -> Result<Changelog, reqwest::Error> {
        let top_folder_url = format!(
            "https://elearning.uni-oldenburg.de/api.php/folder/{}",
            folder.id
        );

        let top_folder = client
        .get(top_folder_url)
        .basic_auth(&settings.api_username, Some(&settings.api_password))
        .send()
        .await?
        .json::<UniDir>()
        .await?;

        
        //1. Sync files
        for file in top_folder.file_refs.unwrap_or(Vec::new()) {
            if !file.is_downloadable {
                continue;
            }
            let sanitized_name = clear_special_chars(&file.name);
            let fp = &(folder.path.clone().unwrap() + "\\" + &sanitized_name);
            let f_path = Path::new(fp);

            if !f_path.exists() {
                let file_data = download_file(file.id, client, settings).await?;
                let mut f = fs::File::create(f_path).unwrap();

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
                    //TODO maybe change to err
                    return Ok("Failed to create directory, cancelling sync of this course\n".to_owned());
                }
                cl.push_str(&("     CREATED: course sub folder directory ".to_owned() + &sanitized_name + " at " + &dir.path.clone().unwrap() + "\n"));
            }

            Course::sync_dir(dir, cl, client, settings).await?;
        }

        Ok(cl.to_string())
    }
}