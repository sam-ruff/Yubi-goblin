use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::rest::dependencies::are_dependencies_installed,
        crate::rest::dependencies::install_missing_dependencies,
        crate::rest::dependencies::remove_unwanted_dependencies,
        crate::rest::yubikey::get_yubikeys,
        crate::rest::yubikey::post_install_yubikey_for_user,
        crate::rest::yubikey::delete_yubikey_for_user,
        crate::rest::yubikey::get_check_yubikey_for_user,
        crate::rest::users::get_users,
    ),
    info(
        title="YubiGoblin",
        version="1.0.0",
        contact(name="Sam Ruff", email="sam@technesci.co.uk", url="https://www.technesci.co.uk")
    ),
    components(
        schemas(
            crate::models::models::Dependencies,
            crate::models::models::YubiKey,
            crate::models::models::YubikeyInstallRequest,
            crate::models::models::ActionResponse,
            crate::models::models::YubikeyStatusResponse,
        ),
    )
)]
pub struct ApiDoc;