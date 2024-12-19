use crate::models::consts::USERS_URL;
use crate::system::users;
use actix_web::{HttpResponse, Responder};

/// Get all users on the system as a JSON array of strings.
#[utoipa::path(
    get,
    path = USERS_URL,
    tag = "Users",
    description = "Get all Users on the system",
    responses(
        (status = 200, description = "List of Users", body = Vec<String>, content_type = "application/json")
    ),
    params()
)]
pub async fn get_users() -> impl Responder {
    match users::list_system_users() {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}