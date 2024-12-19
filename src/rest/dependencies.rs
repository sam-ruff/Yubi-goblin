use actix_web::{web, HttpResponse, Responder};
use crate::install::apt::{check_dependencies, install_packages, remove_packages};
use crate::models::models::{Dependencies, ErrorMessage};
use crate::models::consts::DEPENDENCY_URL;

#[utoipa::path(
get,
path = DEPENDENCY_URL,
tag = "Dependencies",
description = "Get a list of packages or dependencies",
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


#[utoipa::path(
    post,
    path = DEPENDENCY_URL,
    tag = "Dependencies",
    description = "Install missing dependencies",
    request_body = Dependencies,
    responses(
        (status = 200, description = "Dependencies installed successfully", body = Dependencies, content_type = "application/json"),
        (status = 500, description = "Error installing dependencies", body = ErrorMessage, content_type = "application/json")
    ),
    params()
)]
pub async fn install_missing_dependencies(posted_deps: web::Json<Dependencies>) -> impl Responder {
    // Check the current state of dependencies on the system
    let current_deps = match get_current_dependencies() {
        Ok(deps) => deps,
        Err(err_response) => return err_response,
    };

    // Create a list of (requested_state, current_state, package_name)
    let desired_packages = vec![
        (posted_deps.apt, current_deps.apt, "apt"),
        (posted_deps.libpam_u2f, current_deps.libpam_u2f, "libpam-u2f"),
        (posted_deps.pamu2fcfg, current_deps.pamu2fcfg, "pamu2fcfg"),
    ];

    // Filter out the packages that are requested but currently not installed
    let to_install = get_packages(desired_packages);

    // If no packages need installation, return the current state
    if to_install.is_empty() {
        return HttpResponse::Ok().json(current_deps);
    }

    // Attempt to install missing packages
    if let Err(e) = install_packages(&to_install) {
        return HttpResponse::InternalServerError().json(ErrorMessage {
            message: format!("Failed to install packages: {}", e),
            ..Default::default()
        });
    }

    // After installation attempt, re-check dependencies and return the updated status
    match check_dependencies() {
        Ok(updated_deps) => HttpResponse::Ok().json(updated_deps),
        Err(error) => HttpResponse::InternalServerError().json(ErrorMessage {
            message: format!("Failed to check dependencies after installation: {}", error),
            ..Default::default()
        }),
    }
}

#[utoipa::path(
    delete,
    path = DEPENDENCY_URL,
    tag = "Dependencies",
    description = "Remove libpam_u2f and pamu2fcfg if they are installed",
    responses(
        (status = 200, description = "Dependencies removed successfully", content_type = "application/json"),
        (status = 500, description = "Error removing dependencies", body = ErrorMessage, content_type = "application/json")
    ),
    params()
)]
pub async fn remove_unwanted_dependencies() -> impl Responder {
    // Check the current state of dependencies on the system
    let current_deps = match get_current_dependencies() {
        Ok(deps) => deps,
        Err(err_response) => return err_response,
    };

    // Identify which packages need to be removed based on their installed state
    let mut to_remove = Vec::new();
    if current_deps.libpam_u2f {
        to_remove.push("libpam-u2f");
    }
    if current_deps.pamu2fcfg {
        to_remove.push("pamu2fcfg");
    }

    // If no packages need to be removed, return the current state
    if to_remove.is_empty() {
        return HttpResponse::Ok().json(current_deps);
    }

    // Attempt to remove the unwanted packages
    if let Err(e) = remove_packages(&to_remove) {
        return HttpResponse::InternalServerError().json(ErrorMessage {
            message: format!("Failed to remove packages: {}", e),
            ..Default::default()
        });
    }

    // After removal attempt, re-check dependencies and return the updated status
    match check_dependencies() {
        Ok(updated_deps) => HttpResponse::Ok().json(updated_deps),
        Err(error) => HttpResponse::InternalServerError().json(ErrorMessage {
            message: format!("Failed to check dependencies after removal: {}", error),
            ..Default::default()
        }),
    }
}



/// Attempts to retrieve the current dependencies, returning a Result with the dependencies or an HttpResponse error.
fn get_current_dependencies() -> Result<Dependencies, HttpResponse> {
    check_dependencies().map_err(|error| {
        HttpResponse::InternalServerError().json(ErrorMessage {
            message: format!("Failed to check current dependencies: {}", error),
            ..Default::default()
        })
    })
}


fn get_packages(desired_packages: Vec<(bool, bool, &str)>) -> Vec<&str> {
    let to_install: Vec<&str> = desired_packages
        .into_iter()
        .filter_map(|(wanted, current, pkg)| {
            if wanted && !current {
                Some(pkg)
            } else {
                None
            }
        })
        .collect();
    to_install
}