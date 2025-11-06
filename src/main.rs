use actix_web::{ web, App, HttpResponse, HttpServer, Responder};
use tokio::sync::mpsc;
pub mod routes;
pub use routes::*;
pub mod worker;
pub use worker::processor;
pub use worker::*;
pub mod utils;
pub use utils::*;

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
pub struct AppState {
    worker: mpsc::Sender<Request>,
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let worker = spawn_background_worker();
    HttpServer::new(move|| {
        App::new()
        .app_data(web::Data::new(AppState{worker : worker.clone()}))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}