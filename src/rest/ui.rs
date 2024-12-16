use actix_web::{web, HttpResponse, Responder};
use actix_web::http::header::ContentType;
use log::debug;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/static_files.rs"));

pub async fn index() -> impl Responder {
    let static_files = static_files();
    match static_files.get("index.html") {
        Some(content) => HttpResponse::Ok().content_type("text/html").body(*content),
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn handle_ui_files(req: web::Path<String>) -> impl Responder {
    debug!("Handling files for {}", req.to_string());
    let static_files = static_files();
    let path = req.as_str();

    match static_files.get(path) {
        Some(content) => {
            if path.ends_with(".js") {
                HttpResponse::Ok()
                    .insert_header(ContentType(mime::APPLICATION_JAVASCRIPT))
                    .body(*content)
            } else if path.ends_with(".css") {
                HttpResponse::Ok()
                    .insert_header(ContentType(mime::TEXT_CSS))
                    .body(*content)
            } else {
                HttpResponse::Ok().body(*content)
            }
        },
        None => HttpResponse::NotFound().body(""),
    }
}