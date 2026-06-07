# BaitLoader 🚀

A lightweight  desktop meme-hub and anonymous imageboard built entirely in Rust.

---

## 💡 Project Idea

The core idea of **BaitLoader** is to give a group of friends a fast, independent space to dump absurd screenshots, cursed images, and funny photos under hilarious names and descriptions. 

---

## 🏗️ Tech Stack

* **Server (Backend):** [Axum](https://github.com/tokio-rs/axum) (async web framework) + [Redis](https://redis.io/) (hosted on Railway for blazing fast metadata storage).
* **Client (Frontend):** [egui / eframe](https://github.com/emilk/egui) (immediate-mode desktop GUI) + [reqwest](https://github.com/seanmonstar/reqwest) (HTTP requests).
* **Concurrency:** [Tokio](https://tokio.rs/) runtime running in background OS threads (`std::thread::spawn`) to ensure network actions never freeze the 60 FPS user interface.

---

## 🚀 How to Run

```bash
cargo run
