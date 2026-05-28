#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use eframe::egui;
use reqwest::multipart;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

#[derive(PartialEq)]
enum Page {
    Upload,
    Download,
}

struct MyApp {
    page: Page,
    name: String,
    dir_load: Option<PathBuf>,
    dec: String,
    rx: mpsc::Receiver<PathBuf>,
    tx: mpsc::Sender<PathBuf>,
    show_warning: bool,
    index: u64,
    loaded_app: Vec<AppStorage>,
    selected_app_index: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AppStorage {
    name: String,
    description: String,
    index: u64,
    path: PathBuf,
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
struct file_struct {
    name: String,
    data: Vec<u8>,
    path: PathBuf,
    index: u64,
    desc: String,
}

fn main() {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "baitloader",
        options,
        Box::new(|_cc| {
            Box::new(MyApp::default())
        }),
    )
    .unwrap();
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            page: Page::Upload,
            dir_load: None,
            name: String::new(),
            dec: String::new(),
            show_warning: false,
            tx,
            rx,
            index: 0,
            loaded_app: Vec::new(),
            selected_app_index: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Upload").clicked() {
                    self.page = Page::Upload;
                }

                if ui.button("Download").clicked() {
                    self.loaded_app.clear();
                    self.selected_app_index = None;
                    self.refresh_apps();
                    self.page = Page::Download;
                }
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.page {
                Page::Upload => {
                    ui.label("writ your app name");
                    ui.text_edit_singleline(&mut self.name);
                    ui.label("write description");
                    ui.text_edit_multiline(&mut self.dec);
                    
                    if ui.button("pick_folder").clicked() {
                        let tx = self.tx.clone();
                        std::thread::spawn(move || {
                            if let Some(path) = FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                                .pick_file()
                            {
                                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                    match ext {
                                        "jpeg" | "jpg" | "png" | "gif" | "webp" | "bmp" => {
                                            let _ = tx.send(path.clone());
                                            println!("image Choose: {:?}", path);
                                        }
                                        _ => {
                                            println!("not image");
                                        }
                                    }
                                }
                            }
                        });
                    }

                    while let std::result::Result::Ok(path) = self.rx.try_recv() {
                        self.dir_load = Some(path.clone());
                    }

                    if let Some(path) = &self.dir_load {
                        ui.label(format!("picked: {:?}", path));
                    }

                    if ui.button("upload").clicked() {
                        if self.name.trim().is_empty() || self.dec.trim().is_empty() {
                            self.show_warning = true;
                        } else {
                            self.show_warning = false;
                            if let Some(path) = &self.dir_load {
                                if let std::result::Result::Ok(bytes) = fs::read(path) {
                                    let index = get_last_index();

                                    let file_strucet = file_struct {
                                        name: self.name.clone(),
                                        data: bytes,
                                        path: path.clone(),
                                        index,
                                        desc: self.dec.clone(),
                                    };

                                    add_folder(file_strucet);

                                    self.dir_load = None;
                                    self.name.clear();
                                    self.dec.clear();
                                    self.loaded_app.clear();
                                }
                            }
                        }
                    }
                    
                    if self.show_warning {
                        egui::Window::new("warning")
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.label("Please enter name and description!");
                                if ui.button("OK").clicked() {
                                    self.show_warning = false;
                                }
                            });
                    }
                }

                Page::Download => {
                    ui.heading("apps are: ");
                    ui.separator();

                    if self.loaded_app.is_empty() {
                        ui.label("nothing in store");
                    } else {
                        for app in &self.loaded_app {
                            ui.vertical(|ui| {
                                let app_name_text = format!("📁 {}", app.name);
                                let is_open = self.selected_app_index == Some(app.index);

                                if ui.selectable_label(is_open, &app_name_text).clicked() {
                                    if is_open {
                                        self.selected_app_index = None;
                                    } else {
                                        self.selected_app_index = Some(app.index);
                                    }
                                }

                                if self.selected_app_index == Some(app.index) {
                                    ui.indent("app_detals", |ui| {
                                        ui.colored_label(egui::Color32::LIGHT_GRAY, format!("ID {}", app.index));
                                        ui.label("description:");
                                        ui.colored_label(egui::Color32::WHITE, &app.description);
                                        ui.add_space(5.0);
                                        
                                        if ui.button("download").clicked() {
                                            if let Some(os_str) = app.path.file_name() {
                                                let filenamestring = os_str.to_string_lossy().into_owned();
                                             let app_id = app.index;   
                                                std::thread::spawn(move || {
                                                    let rt = tokio::runtime::Builder::new_current_thread()
                                                        .enable_all()
                                                        .build()
                                                        .unwrap();
                                                        
                                                    rt.block_on(async {
                                                        if let Err(e) = get_image(filenamestring,app_id).await {
                                                            println!("Error getting image: {:?}", e);
                                                        }
                                                    });
                                                });
                                            } else {
                                                println!("cannot get file name");
                                            }
                                        }
                                    });
                                } 
                                ui.separator();
                            });
                        }
                    }
                }
            }
        });
    }
}

fn add_folder(file: file_struct) {
    let file_load = file.clone();

    let new_app = AppStorage {
        name: file.name,
        description: file.desc,
        index: file.index,
        path: file.path,
    };

    let mut apps: Vec<AppStorage> = if Path::new("storage.json").exists() {
        let text = fs::read_to_string("storage.json").unwrap();
        serde_json::from_str(&text).unwrap_or_default()
    } else {
        Vec::new()
    };

    apps.push(new_app);
    let json = serde_json::to_string_pretty(&apps).unwrap();
    fs::write("storage.json", json).unwrap();
    
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            if let Err(e) = load_img(file_load).await {
                   println!("Error uploading image: {:?}", e);
            }
        });
    });
    println!("saved");
}

async fn get_image(name: String,index: u64) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://baitloader-server-production.up.railway.app/image/{}", name);
    let resp = reqwest::get(url).await?;

    if !resp.status().is_success() {
        println!("dont have that img (status: {})", resp.status());
        return std::result::Result::Ok(());
    }
    let bytes = resp.bytes().await?;
    let _ = std::fs::write(format!("download{}.png",index), &bytes);
    println!("image downloaded");
    std::result::Result::Ok(())
}
async fn load_img(file: file_struct) -> Result<(), Box<dyn std::error::Error>> {
    let real_file_name = file.path
        .file_name()
        .map(|os_str| os_str.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown.png".to_string());

    let mime_type = match file.path.extension().and_then(|ext| ext.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => "image/png", 
    };

    let part = multipart::Part::bytes(file.data)
        .file_name(real_file_name) 
        .mime_str(mime_type)?;

    let form = multipart::Form::new().part("image", part);
    let client = reqwest::Client::new();

    let resp = client
        .post("https://baitloader-server-production.up.railway.app/upload")
        .multipart(form)
        .send()
        .await?;

    std::result::Result::Ok(())
}


fn get_last_index() -> u64 {
    if let std::result::Result::Ok(text) = std::fs::read_to_string("storage.json") {
        let apps: Vec<AppStorage> = serde_json::from_str(&text).unwrap_or_default();
        apps.last().map(|x| x.index + 1).unwrap_or(0)
    } else {
        0
    }
}

impl MyApp {
    fn refresh_apps(&mut self) {
        if Path::new("storage.json").exists() {
            if let std::result::Result::Ok(text) = fs::read_to_string("storage.json") {
                if let std::result::Result::Ok(apps) = serde_json::from_str::<Vec<AppStorage>>(&text) {
                    self.loaded_app = apps;
                }
            }
        }
    }
}