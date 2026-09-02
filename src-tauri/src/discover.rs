use serde::{Deserialize, Serialize};

pub mod broadcaster;
pub mod listener;

pub const PORT:u16 = 45820;

#[derive(Serialize,Deserialize)]
pub struct DeviceInfo{
    pub id: String,
    pub name: String,
    pub port: u16
}
