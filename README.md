# Studip course file downloader

[![Build Status](https://github.com/Khirath-bit/studip-downloader/actions/workflows/build.yml/badge.svg)](https://github.com/Khirath-bit/studip-downloader/actions/workflows/build.yml)
[![Releases](https://img.shields.io/github/v/release/Khirath-bit/studip-downloader)](https://github.com/Khirath-bit/studip-downloader/releases)

Small syncing app to always be up to date on documents and never be forced to download them manually from studip.

## Config

```json
{
    "course_directory_path": "",
    "university_base_api_url": "",
    "api_password": "",
    "api_username": "",
    "download_videos": false,
    "course_blacklist": []
}
```
- **course_directory_path** is the path where the studip course structure will be stored and synced at.
- **university_base_api_url** is the base url of your university's studip domain
- **user_id** is your user id, you can usually find it by opening your profile and copying it from the url.
- **api_password** and **api_username** are your studip credentials.
- **download_videos** is a flag to decide whether to download videos or not.
- **course_blacklist** is a collection of **course_id** to ignore certain courses in the sync process

## Host as cron job
I recommend running this as task or service depending on your os.

## Notes
This downloader replaces every special file/directory character forbidden in windows with an empty space.
