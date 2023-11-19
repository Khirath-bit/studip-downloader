# Studip course file downloader

Small syncing app to always be up to date on documents and never be forced to download them manually from studip.

## Config

```json
{
    "course_directory_path": "",
    "university_base_api_url": "",
    "user_id": "",
    "api_password": "",
    "api_username": ""
}
```
- **course_directory_path** is the path where the studip course structure will be stored and synced at.
- **university_base_api_url** is the base url of your university's studip domain
- **user_id** is your user id, you can usually find it by opening your profile and copying it from the url.
- **api_password** and **api_username** are your studip credentials.

## Host as cron job
I recommend running this as task or service depending on your os.

## Notes
This downloader replaces every special file/directory character forbidden in windows with an empty space.

Todos:
- [ ] Add wrapper for get requests