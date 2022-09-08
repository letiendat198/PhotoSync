#![windows_subsystem = "windows"]

use notify::{RecommendedWatcher, Watcher, RecursiveMode};
use std::sync::mpsc::channel;
use std::time::Duration;
use std::path;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use yup_oauth2::authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate};
use notify_rust::Notification;
use std::future::Future;
use std::pin::Pin;
use std::process::Command;
use webbrowser;
use dirs;

mod api_handler;

async fn browser_user_url(url: &str, need_code: bool) -> Result<String, String> {
    if webbrowser::open(url).is_ok() {
        println!("webbrowser was successfully opened.");
    }
    let def_delegate = DefaultInstalledFlowDelegate;
    def_delegate.present_user_url(url, need_code).await
}

#[derive(Copy, Clone)]
struct InstalledFlowBrowserDelegate;

impl InstalledFlowDelegate for InstalledFlowBrowserDelegate {
    /// the actual presenting of URL and browser opening happens in the function defined above here
    /// we only pin it
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(browser_user_url(url, need_code))
    }
}

#[tokio::main]
async fn main() {
    println!("Auth called");
    let secret = match yup_oauth2::read_application_secret("secret.json").await {
        Ok(a) => a,
        Err(_e) => {
            Notification::new().summary("PhotoSync").body("secret.json not found or invalid. Please make sure your application secret is renamed to secret.json and in the same folder as the executable. Panic!").show().unwrap();
            panic!("Missing application secret")
        }
    };
    println!("{:?}", secret);
    let authenticator = match InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect).flow_delegate(Box::new(InstalledFlowBrowserDelegate)).build().await {
        Ok(a) => a,
        Err(e) => {
            Notification::new().summary("PhotoSync").body("Unable to create Authenticator. Panic!").show().unwrap();
            panic!("{}", e);
        }
    };
    let scopes = &["https://www.googleapis.com/auth/photoslibrary.appendonly"];

    let is_ffmpeg = Command::new("cmd").arg("/C").arg("ffmpeg.exe").arg("-version").status().expect("Cannot execute");
    if !is_ffmpeg.success() {
        Notification::new().summary("PhotoSync").body("ffmpeg.exe not found! Please make sure ffmpeg.exe is in the same directory as the executable or in PATH. Panic!").show().unwrap();
        panic!("ffmpeg.exe not found!");
    }

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(5)).unwrap(); 

    let mut path: path::PathBuf = dirs::home_dir().unwrap();
    path.push("Pictures");
    path.push("Screenshots");
    if path.exists() {
        match watcher.watch(path, RecursiveMode::Recursive) {
            Ok(_a) => println!("Watcher inited"),
            Err(e) => {
                Notification::new().summary("PhotoSync").body("Unable to start Watcher. Panic!").show().unwrap();
                panic!("{}", e);
            } 
        }
    }
    else {
        let mut path: path::PathBuf = dirs::home_dir().unwrap();
        path.push("OneDrive");
        path.push("Pictures");
        path.push("Screenshots");
        match watcher.watch(path, RecursiveMode::Recursive) {
            Ok(_a) => println!("Watcher inited"),
            Err(e) => {
                Notification::new().summary("PhotoSync").body("Unable to start Watcher. Panic!").show().unwrap();
                panic!("{}", e);
            } 
        }
    }

    let mut path2: path::PathBuf = dirs::home_dir().unwrap();
    path2.push("Documents");
    path2.push("League of Legends");
    path2.push("Highlights");
    if path2.exists() {
        match watcher.watch(path2, RecursiveMode::Recursive) {
            Ok(_a) => println!("Watcher inited"),
            Err(e) => {
                Notification::new().summary("PhotoSync").body("Unable to start Watcher. Panic!").show().unwrap();
                panic!("{}", e);
            } 
        }
    }
    else {
        let mut path2: path::PathBuf = dirs::home_dir().unwrap();
        path2.push("OneDrive");
        path2.push("Documents");
        path2.push("League of Legends");
        path2.push("Highlights");
        match watcher.watch(path2, RecursiveMode::Recursive) {
            Ok(_a) => println!("Watcher inited"),
            Err(e) => {
                Notification::new().summary("PhotoSync").body("Unable to start Watcher. Panic!").show().unwrap();
                panic!("{}", e);
            } 
        }
    }

    Notification::new().summary("PhotoSync").body("PhotoSync is running!").show().unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("{:?}", event);
                match event {
                    notify::DebouncedEvent::Write(_) => println!("Write"),
                    notify::DebouncedEvent::Chmod(_) => println!("Chmod") ,
                    notify::DebouncedEvent::Error(_, _) => println!("Error"),
                    notify::DebouncedEvent::NoticeRemove(_) => println!("NoticeRemove"),
                    notify::DebouncedEvent::NoticeWrite(_) => println!("NoticeWrite"),
                    notify::DebouncedEvent::Remove(_) => println!("Remove"),
                    notify::DebouncedEvent::Rename(_, _) => println!("Rename"),
                    notify::DebouncedEvent::Rescan => println!("Rescan"),
                    notify::DebouncedEvent::Create(name) => {
                        Notification::new().summary("PhotoSync").body("Authenticating...").show().unwrap();
                        match authenticator.token(scopes).await {
                            Ok(token) => {
                                let token_str = token.as_str();
                                api_handler::upload(name.to_owned(), token_str).await;
                                let message = "File ".to_owned() + &name.into_os_string().into_string().unwrap() + " synced to Google Photos";
                                Notification::new().summary("PhotoSync").body(&message).show().unwrap();
                            },
                            Err(_e) => {
                                Notification::new().summary("PhotoSync").body("Unable to accquire token. Please try again!. Not panic!").show().unwrap();
                            }
                        };  
                    }
                };
            },
            Err(e) => println!("{:?}", e)
        };
    }
}
