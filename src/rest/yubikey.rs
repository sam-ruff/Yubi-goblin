use actix_web::{HttpResponse, Responder};

pub async fn submit_key() -> impl Responder {
    HttpResponse::Ok().body("ZugZug")
}