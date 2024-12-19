use crate::models::consts::{CHECK_YUBI_KEY_FOR_USER, REMOVE_YUBI_KEY_FOR_USER, YUBI_KEY_URL};
use crate::models::models::{ActionResponse, ErrorMessage, YubiKey, YubikeyInstallRequest, YubikeyStatusResponse};
use crate::system::yubikey;
use actix_web::{web, HttpResponse, Responder};
use crate::system::yubikey::{check_if_yubikey_is_installed_for_user, install_yubikey_for_user, remove_yubikey_for_user};

#[utoipa::path(
    get,
    path = YUBI_KEY_URL,
    tag = "YubiKeys",
    description = "Get all YubiKeys on the system",
    responses(
        (status = 200, description = "List of Yubi keys", body = Vec<YubiKey>, content_type = "application/json")
    ),
    params()
)]
pub async fn get_yubikeys() -> impl Responder {
    match yubikey::fetch_yubikeys() {
        Ok(found_yubikeys) => HttpResponse::Ok().json(found_yubikeys),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}


/// Installs a YubiKey for a given user.
#[utoipa::path(
    post,
    path = YUBI_KEY_URL,
    tag = "YubiKeys",
    description = "Install a YubiKey for a given user",
    request_body = YubikeyInstallRequest,
    responses(
        (status = 200, description = "YubiKey installed successfully", content_type = "application/json"),
        (status = 500, description = "Error installing YubiKey", content_type = "application/json")
    )
)]
pub async fn post_install_yubikey_for_user(request: web::Json<YubikeyInstallRequest>) -> impl Responder {
    match install_yubikey_for_user(&request.username).await {
        Ok(_) => HttpResponse::Ok().json(format!("YubiKey installed successfully for user {}", request.username)),
        Err(e) => HttpResponse::InternalServerError().json(format!("Failed to install YubiKey: {}", e)),
    }
}

/// Checks if a user has a yubi key installed
#[utoipa::path(
    get,
    path = CHECK_YUBI_KEY_FOR_USER,
    tag = "YubiKeys",
    description = "Check if YubiKey is installed for a given user",
    params(
        ("username" = String, Path, description = "The username to check for YubiKey installation")
    ),
    responses(
        (status = 200, description = "YubiKey installation status for the user", body = YubikeyStatusResponse, content_type = "application/json"),
        (status = 500, description = "Error checking YubiKey installation", body = ErrorMessage, content_type = "application/json")
    )
)]
pub async fn get_check_yubikey_for_user(path: web::Path<String>) -> impl Responder {
    let username = path.into_inner();
    match check_if_yubikey_is_installed_for_user(&username) {
        Ok(is_installed) => {
            HttpResponse::Ok().json(YubikeyStatusResponse {
                username,
                yubikey_installed: is_installed,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorMessage {
                message: format!("Failed to check YubiKey installation: {}", e),
                error: true,
            })
        }
    }
}

/// Removes a yubikey for a given user
#[utoipa::path(
    delete,
    path = REMOVE_YUBI_KEY_FOR_USER,
    tag = "YubiKeys",
    description = "Remove a YubiKey for a given user",
    params(
        ("username" = String, Path, description = "The username from whom the YubiKey should be removed")
    ),
    responses(
        (status = 200, description = "YubiKey removed successfully", body = ActionResponse, content_type = "application/json"),
        (status = 500, description = "Error removing YubiKey", body = ErrorMessage, content_type = "application/json")
    )
)]
pub async fn delete_yubikey_for_user(path: web::Path<String>) -> impl Responder {
    let username = path.into_inner();
    match remove_yubikey_for_user(&username) {
        Ok(_) => HttpResponse::Ok().json(ActionResponse {
            username,
            yubikey_removed: true,
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorMessage {
            message: format!("Failed to remove YubiKey: {}", e),
            error: true,
        }),
    }
}