use serde::{Deserialize, Serialize};

// This is a shared data struct that we will pass to the router
pub struct SharedData {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Todo {
    pub id: u8,
    pub name: String,
    pub description: String,
}
