use reqwest;
use std::path::PathBuf;
use std::io::Read;
use std::fs::File;
use std::fs;
use serde_json::json;
use std::process::Command;

pub async fn upload(path: PathBuf, token: &str) {
    println!("Upload called");
    let mut f = File::open(&path).unwrap();
    let metadata = fs::metadata(&path).unwrap();
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).unwrap();

    let mut extension = String::new();
    let file_type = path.extension().unwrap().to_str().unwrap().to_owned();

    if file_type == "webm" {
        let mut path_file = path.clone();
        path_file.set_extension("mp4");
        println!("{:?}", path);
        println!("{:?}", path_file);
        Command::new("cmd").arg("/C").arg("ffmpeg.exe").arg("-i").arg(path.to_str().unwrap()).arg(path_file.to_str().unwrap()).output().expect("Failed to run ffmpeg!");
        return
    }
    if file_type == "avi" || file_type == "mp4" {
        extension = "video/".to_owned() + &file_type;
    }
    else {
        extension = "image/".to_owned() + &file_type;
    }
    println!("{:?}", file_type);

    println!("Requesting!");
    let client = reqwest::Client::new();
    let res = client.post("https://photoslibrary.googleapis.com/v1/uploads").header("Content-type", "application/octet-stream").header("X-Goog-Upload-Content-Type", extension).header("X-Goog-Upload-Protocol", "raw").bearer_auth(&token).body(buffer).send().await.unwrap();
    let up_token = res.text().await.unwrap();
    println!("{:?}", &up_token);


/*     let content = json!({
        "album": {
            "title": "Screenshots PC"
        }
    });
    let create_album = client.post("https://photoslibrary.googleapis.com/v1/albums").header("Content-type", "application/json").bearer_auth(&token).body(content.to_string()).send().await.unwrap();
    let album_id = create_album.text().await.unwrap();
    println!("{:?}", album_id); */

    let request = json!({
        // "albumId": album_id,
        "newMediaItems": [
            {
                "description": "Sync from PC",
                "simpleMediaItem": {
                    "uploadToken": up_token
                }
            }
        ]
    });
    let create = client.post("https://photoslibrary.googleapis.com/v1/mediaItems:batchCreate").bearer_auth(&token).body(request.to_string()).send().await.unwrap();
    println!("{:?}", create.text().await.unwrap());
}