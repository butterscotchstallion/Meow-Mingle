use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Ok,
    Error,
}
