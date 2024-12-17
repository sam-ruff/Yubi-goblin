use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::rest::yubikey::are_dependencies_installed,
    ),
    info(
        title="YubiGoblin",
        version="1.0.0",
        contact(name="Sam Ruff", email="sam@technesci.co.uk", url="https://www.technesci.co.uk")
    ),
    components(
        schemas(
            crate::models::models::Dependencies,
        ),
    )
)]
pub struct ApiDoc;