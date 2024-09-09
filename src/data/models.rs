use serde::{Deserialize, Serialize};
use serde_json as sj;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Command {
    pub binary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub invocations: Vec<String>,
}


