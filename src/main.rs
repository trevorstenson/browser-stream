use actix_files::Files;
use actix::Actor;
use actix_web::{web, App, HttpServer};
use browser::{BrowserManager};

use websocket::index;

mod browser;
mod websocket;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let browser = BrowserManager::new("https://stackoverflow.com/").start();
    println!("Starting server at http://localhost:3000");
    HttpServer::new(move || {
      App::new()
        .app_data(browser.clone())
        .route("/ws", web::get().to(index))
        .service(Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
