use std::{collections::HashMap, net::IpAddr, sync::Arc};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle,Emitter};
use tokio::sync::Mutex;
pub mod broadcaster;
pub mod listener;

pub const PORT:u16 = 45820;
pub const BROADCAST_ADDR:&str = "255.255.255.255";

#[derive(Serialize,Deserialize,Debug,Clone,PartialEq)]
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
    ip: IpAddr,
    my_id: &str,
    app:Option<&AppHandle>)
{
    if device.id == my_id {
        return;
    }
    let changed = {
        let mut devices = devices.lock().await;
        match devices.get(&device.id) {
            Some((existing, existing_ip)) if *existing == device && *existing_ip == ip => false,
            _ => {
                devices.insert(device.id.clone(),(device, ip));
                true
            }
        }
    };
    if changed {
        if let Some(app) = app{
            let _ = app.emit("devices_changed", ());
        }
    }
}


pub async fn get_devices(
    devices: Arc<Mutex<HashMap<String, (DeviceInfo, IpAddr)>>>,
) -> Vec<(DeviceInfo, IpAddr)> {
    let devices = devices.lock().await;

    devices.values().cloned().collect()
}
