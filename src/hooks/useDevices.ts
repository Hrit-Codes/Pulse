import { invoke } from "@tauri-apps/api/core";
import { Device } from "../types";

export async function fetchDevices():Promise<Device[]>{
    try{
        const devices=await invoke<Device[]>("scan_devices");
        return devices;
    }catch(error){
        console.log("Failed to scan devices::",error);
        return[];
    }
}