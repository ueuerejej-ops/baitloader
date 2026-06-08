#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
use eframe::egui::RichText;
use eframe::egui;
use ::multipart::server::nickel::nickel::extensions::response;
use rfd::FileDialog;
use rfd::MessageButtons::Ok as rdfok;
use serde::{Deserialize, Serialize};
use reqwest::multipart;
#[derive(Debug, Deserialize, Clone)]
pub struct AppInfo {
    pub file_name: String,
    pub desc: String,
    pub name: String
}

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
    tx_catalog: mpsc::Sender<String>,
    rx_catalog: mpsc::Receiver<String>,
    show_warning: bool,
    loaded_app: Vec<AppStorage>,
    selected_app_namex: Option<String>,
    catalog_raw: String,
    apps_list: Vec<AppInfo>,
}
#[derive(Debug, Deserialize)]
pub struct AppCatalog {
    pub apps: Vec<AppInfo>,
}
#[derive(Serialize, Deserialize, Debug)]

struct AppStorage {
    name: String,
    description: String,
    path: PathBuf,
}

#[derive(Clone)]
#[allow(non_camel_case_types)]
struct file_struct {
    name: String,
    data: Vec<u8>,
    path: PathBuf,
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
        
        let (tx_catalog, rx_catalog) = mpsc::channel();

        Self {
            page: Page::Upload,
            dir_load: None,
            name: String::new(),
            dec: String::new(),
            show_warning: false,
            tx,
            rx,
            tx_catalog,
            rx_catalog,
            loaded_app: Vec::new(),
            selected_app_namex: None,
            apps_list: Vec::new(), 
            catalog_raw: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
while let Ok(text) = self.rx_catalog.try_recv() {
    self.catalog_raw = text.clone();

    match serde_json::from_str::<AppCatalog>(&text) {
        Ok(catalog) => {
            self.apps_list = catalog.apps;
        }
        Err(err) => {

        }
    }
}
        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Upload").clicked() {
                    self.page = Page::Upload;
                }

                if ui.button("Download").clicked() {
                    self.loaded_app.clear();
                    self.selected_app_namex = None;
                    let tx_catalog = self.tx_catalog.clone();
                    if self.catalog_raw.is_empty() {
                    std::thread::spawn(move ||{
                        let text = fetch_catalog();

                       let _ = tx_catalog.send(text);
                       
                    });
                }
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

                                    let file_strucet = file_struct {
                                        name: self.name.clone(),
                                        data: bytes,
                                        desc: self.dec.clone(),
                                        path: path.to_path_buf()
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
                   ui.horizontal(|ui| {
        ui.heading("Список  картинок");
        
        if ui.button("reloade page").clicked() {
            self.catalog_raw = "Обновление...⏳".to_string();
            let tx_catalog = self.tx_catalog.clone();
            let ctx_clone = ctx.clone();
            std::thread::spawn(move || {
                let text = fetch_catalog();
                let _ = tx_catalog.send(text);
                ctx_clone.request_repaint();
            });
        }
    });

    ui.separator(); 

        egui::ScrollArea::vertical().show(ui, |ui| {
            for app in &self.apps_list {
              egui::CollapsingHeader::new(&app.name).id_source(&app.file_name).show(ui, |ui|{
                        ui.label(&app.desc);
                        
                        ui.add_space(4.0);

                        if ui.button("download").clicked(){
                            let file_name = app.file_name.clone();

                            std::thread::spawn(move ||{
                                let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                                rt.block_on(async{
                                    match get_image(file_name).await{
                                        Ok(_) => println!("Картинка успешно скачана!"),
                Err(e) => println!("Ошибка при скачивании: {:?}", e),
                                    }
                                })
                            });
                        }
              });
                ui.add_space(4.0);
            }
        });
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
        path: file.path,
    };

    
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

async fn get_image(name: String,) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://baitloader-server-production.up.railway.app/image/{}", name);
    let resp = reqwest::get(url).await?;

    if !resp.status().is_success() {
        println!("dont have that img (status: {})", resp.status());
        return std::result::Result::Ok(());
    }
    let bytes = resp.bytes().await?;
    let _ = std::fs::write(name, &bytes);
    println!("image downloaded");
    std::result::Result::Ok(())
}
async fn load_img(file: file_struct) -> Result<(), Box<dyn std::error::Error>> {
    let real_file_name = file.path
        .file_name()
        .map(|os_str| os_str.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown.png".to_string());

    let desc = file.desc;
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

    let form = multipart::Form::new().part("image", part).text("desc", desc).text("name",file.name );
    let client = reqwest::Client::new();

    let resp = client
        .post("https://baitloader-server-production.up.railway.app/upload")
        .multipart(form)
        .send()
        .await?;

    std::result::Result::Ok(())
}



fn fetch_catalog() -> String {
    let url = "https://baitloader-server-production.up.railway.app/info/all";
    
    let res = ::reqwest::blocking::get(url);
    
    let response = res.expect("Ошибка сети");
    
    let raw_html_or_json = response.text().expect("Не удалось прочитать текст ответа");
    
    raw_html_or_json
}
