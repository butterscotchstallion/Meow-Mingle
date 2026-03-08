use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Ok,
    Error,
}
