use serde::{Deserialize, Serialize};

pub mod broadcaster;
pub mod listener;

pub const PORT:u16 = 45820;
pub const BROADCAST_ADDR:&str = "255.255.255.255";
pub const LOCAL_BIND_ADDR:&str = "0.0.0.0:0"; //available port on any network interface 
#[derive(Serialize,Deserialize,Debug)]
pub struct DeviceInfo{
    pub id: String,
    pub name: String,
    pub port: u16
}
