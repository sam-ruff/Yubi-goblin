use actix_web::{HttpResponse, Responder};
use crate::models::models::Dependencies;

pub const DEPENDENCY_URL: &str = "/api/v1/dependencies";
pub async fn submit_key() -> impl Responder {
    HttpResponse::Ok().body("ZugZug")
}

#[utoipa::path(
get,
path = DEPENDENCY_URL,
tag = "Find out if the dependencies are installed",
responses(
(status = 200, description = "Dependencies object", body = Dependencies, content_type = "application/json")
),
params()
)]
pub async fn are_dependencies_installed() -> impl Responder {
    let dependencies = Dependencies { 
        apt: false
    };
    HttpResponse::Ok().json(dependencies)
}