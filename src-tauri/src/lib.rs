pub mod protocol;
pub mod discover;
pub mod storage;
pub mod transfer;
pub mod util;

use std::{collections::HashMap, net::IpAddr, path::PathBuf};
use std::net::SocketAddr;
use std::sync::Arc;

use tauri::{async_runtime::JoinHandle, AppHandle, State};
use tokio::sync::{oneshot, Mutex};

use crate::discover::broadcaster::broadcast_discover;
use crate::discover::get_devices;
use crate::discover::{listener::listen_for_discover, DeviceInfo};
use crate::storage::TransferStore;
use crate::transfer::receiver::{receive_file, resume_transfer};
use crate::transfer::sender::{request_to_send_file, run_resume_listener};
use crate::transfer::TRANSFER_PORT;
use crate::util::emit_error;

pub type PinSenders = Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>;
pub type Devices = Arc<Mutex<HashMap<String, (DeviceInfo, IpAddr)>>>;

// Holds the running receiver task so we can stop it later
pub struct ReceiverHandle(pub std::sync::Mutex<Option<JoinHandle<()>>>);
pub struct ResumeHandle(pub std::sync::Mutex<Option<JoinHandle<()>>>);

fn load_my_device() -> Result<DeviceInfo, String> {
    let store = TransferStore::new().map_err(|e| e.to_string())?;
    let (id, name) = store.get_or_create_identity().map_err(|e| e.to_string())?;
    Ok(DeviceInfo::new(id, name, TRANSFER_PORT))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("guten morgen! {}", name)
}

#[tauri::command]
async fn submit_pin(
    connection_id: String,
    pin: String,
    pin_senders: State<'_, PinSenders>,
) -> Result<(), String> {
    let mut senders = pin_senders.lock().await;

    let tx = senders
        .remove(&connection_id)
        .ok_or("No transfer is waiting for a PIN with this connection ID")?;

    tx.send(pin)
        .map_err(|_| "Transfer is no longer waiting for PIN".to_string())?;

    Ok(())
}

#[tauri::command]
fn start_receiver(
    addr: String,
    app: AppHandle,
    handle: State<'_, ReceiverHandle>,
    pin_senders: State<'_, PinSenders>,
) -> Result<(), String> {
    let addr: SocketAddr = addr
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;
    let pin_senders = pin_senders.inner().clone();

    let mut guard = handle.0.lock().unwrap();

    // if one is already running, stop it first
    if let Some(old) = guard.take() {
        old.abort();
    }

    let task = tauri::async_runtime::spawn(async move {
        //function no4: runs when 'start_receiver' is invoked
        let result = receive_file(addr, Some(app.clone()), pin_senders)
            .await
            .map_err(|e| e.to_string());
        if let Err(msg) = result {
            emit_error(Some(&app), msg).await;
        }
    });
    *guard = Some(task);

    Ok(())
}

#[tauri::command]
fn stop_receiver(handle: State<'_, ReceiverHandle>) {
    if let Some(task) = handle.0.lock().unwrap().take() {
        task.abort();
    }
}

#[tauri::command]
fn start_resume_listener(
    addr: String,
    app: AppHandle,
    handle: State<'_, ResumeHandle>,
) -> Result<(), String> {
    let addr: SocketAddr = addr
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;
    let store = Arc::new(TransferStore::new().map_err(|e| e.to_string())?);

    let mut guard = handle.0.lock().unwrap();
    if let Some(old) = guard.take() {
        old.abort(); // restart cleanly if already running
    }

    let task = tauri::async_runtime::spawn(async move {
        //function no6: runs when 'start_resume_listener' is invoked
        let result = run_resume_listener(addr, store, Some(&app))
            .await
            .map_err(|e| e.to_string());
        if let Err(msg) = result {
            emit_error(Some(&app), msg).await;
        }
    });
    *guard = Some(task);
    Ok(())
}

#[tauri::command]
fn stop_resume_listener(handle: State<'_, ResumeHandle>) {
    if let Some(task) = handle.0.lock().unwrap().take() {
        task.abort();
    }
}

#[tauri::command]
async fn broadcast(app: AppHandle, devices: State<'_, Devices>) -> Result<(), String> {
    let my_device = load_my_device()?;
    let devices = devices.inner().clone();
    //function no2: runs when 'broadcast' is invoked
    broadcast_discover(devices, my_device, Some(&app))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_devices(
    devices: State<'_, Devices>,
) -> Result<Vec<(DeviceInfo, IpAddr)>, String> {
    //function no3: runs when 'list_devices' is invoked
    Ok(get_devices(devices.inner().clone()).await)
}

#[tauri::command]
async fn send_file(
    sender_id: String,
    addr: String,
    file_path: String,
    app: AppHandle,
) -> Result<(), String> {
    let addr: SocketAddr = addr
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;
    let path = PathBuf::from(file_path);
    let store = Arc::new(TransferStore::new().map_err(|e| e.to_string())?);
    //function no5: runs when 'send_file' is invoked
    request_to_send_file(sender_id, addr, &path, store, Some(&app))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn resume(
    transfer_id: String,
    app: AppHandle,
    devices: State<'_, Devices>,
) -> Result<(), String> {
    let store = Arc::new(TransferStore::new().map_err(|e| e.to_string())?);
    let devices = devices.inner().clone();
    resume_transfer(transfer_id, store, devices, Some(&app))
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let devices: Devices = Arc::new(Mutex::new(HashMap::new()));

    tauri::Builder::default()
        .manage(PinSenders::default())
        .manage(ReceiverHandle(std::sync::Mutex::new(None)))
        .manage(ResumeHandle(std::sync::Mutex::new(None))) // was missing, needed for start_resume_listener
        .manage(devices.clone())
        .setup(move |app| {
            let handle = app.handle().clone();
            let devices = devices.clone();

            tauri::async_runtime::spawn(async move {
                let my_device = match load_my_device() {
                    Ok(d) => d,
                    Err(msg) => {
                        emit_error(Some(&handle), msg).await;
                        return;
                    }
                };

                //function no1: runs throughout the running time of the app
                let result = listen_for_discover(devices, my_device, Some(&handle))
                    .await
                    .map_err(|e| e.to_string());

                if let Err(msg) = result {
                    emit_error(Some(&handle), msg).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet, //dummy fn, will remove it
            submit_pin, //for pin submission
            start_receiver, //starts the receive_file
            stop_receiver, //stops the receive file
            broadcast, //broadcasts the ip
            list_devices, //get the available devices
            send_file, //send file to receiver
            start_resume_listener, //sender listens to resume transfer
            stop_resume_listener, //stop the listen resume function
            resume //receiver invokes it asking to resume the transfer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
