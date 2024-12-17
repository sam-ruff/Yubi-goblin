use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct Dependencies {
    #[schema(example = true)]
    /// Set to true if apt is installed
    pub(crate) apt: bool,
}