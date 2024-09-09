use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Command {
    pub binary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub invocations: Vec<String>,
}


