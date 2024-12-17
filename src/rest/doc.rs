use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::rest::yubikey::are_dependencies_installed,
        crate::rest::yubikey::get_yubikeys,
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
        ),
    )
)]
pub struct ApiDoc;