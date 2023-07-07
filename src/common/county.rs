use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize, Debug, Hash)]
pub struct County(pub String);
