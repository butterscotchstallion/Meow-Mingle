use serde::Serialize;

#[derive(Serialize, Debug, serde::Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Ok,
    Error,
}
