use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum Status {
    Ok,
    Error,
}
