use crate::install::apt::check_dependencies;
use crate::models::models::{Dependencies, ErrorMessage, YubiKey};
use crate::rest::consts::{DEPENDENCY_URL, YUBI_KEY_URL};
use actix_web::{HttpResponse, Responder};

#[utoipa::path(
get,
path = YUBI_KEY_URL,
tag = "Get all YubiKeys on the system",
responses(
(status = 200, description = "List of Yubi keys", body = Vec<YubiKey>, content_type = "application/json")
),
params()
)]
pub async fn get_yubikeys() -> impl Responder {
    let y1 = YubiKey{name: "yub1".to_string(), usb_port:1};
    let y2 = YubiKey{name: "yub2".to_string(), usb_port:1};
    HttpResponse::Ok().json(vec!(y1, y2))
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
    match check_dependencies(){
        Ok(dependencies) => {
            HttpResponse::Ok().json(dependencies)
        }
        Err(error) => {
            HttpResponse::InternalServerError().json(ErrorMessage {
                message: error.to_string(),
                ..Default::default()
            })

        }
    }
}

