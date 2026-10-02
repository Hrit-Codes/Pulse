use std::{collections::HashMap, net::IpAddr, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle,Emitter};
use tokio::sync::Mutex;
pub mod broadcaster;
pub mod listener;

pub const PORT:u16 = 45820;
pub const BROADCAST_ADDR:&str = "255.255.255.255";
pub const LOCAL_BIND_ADDR:&str = "0.0.0.0:0"; //available port on any network interface 
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct DeviceInfo{
    pub id: String,
    pub name: String,
    pub port: u16
}
impl DeviceInfo {
    pub fn new(id:String,name:String,port:u16)->Self{
        Self { id , name, port}
    }
}


pub(super) async fn add_device(
    devices: &Arc<Mutex<HashMap<String, (DeviceInfo, IpAddr)>>>,
    device: DeviceInfo,
    ip: IpAddr,app:Option<&AppHandle>)
{
    {
        let mut devices = devices.lock().await;
        devices.insert(device.id.clone(),(device, ip));
    }
    if let Some(app) = app{
        let _ = app.emit("devices_changed", ());
    }
}


pub async fn get_devices(
    devices: Arc<Mutex<HashMap<String, (DeviceInfo, IpAddr)>>>,
) -> Vec<(DeviceInfo, IpAddr)> {
    let devices = devices.lock().await;

    devices.values().cloned().collect()
}
