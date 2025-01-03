use serde::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscribe {
    pub method: String,
    pub params: Vec<String>,
    pub id: i32,
}
