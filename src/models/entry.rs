use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde_with::skip_serializing_none]
pub struct Entry {
    #[serde(rename = "_id")]
    pub id: String,
    pub list_id: String,
    pub name: String,
    pub done: bool,
}
