use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FnDetails {
    pub name: String,
    pub args: Vec<FnArgs>,
    pub ret: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct FnArgs {
    pub name: String,
    pub arg_type: String,
}
