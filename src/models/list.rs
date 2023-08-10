use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde_with::skip_serializing_none]
pub struct List {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}
