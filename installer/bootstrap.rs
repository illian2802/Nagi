#![windows_subsystem = "windows"]
use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SETUP: &[u8] = include_bytes!("payload/Nagi-Setup-0.3.3.exe");
const PART0: &[u8] = include_bytes!("payload/Nagi-Setup-0.3.3-0.bin");
const PART1: &[u8] = include_bytes!("payload/Nagi-Setup-0.3.3-1.bin");

#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBoxW(hwnd: *mut c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
}

fn show_error(message: &str) {
    let text: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let caption: Vec<u16> = "Installation de Nagi".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10); }
}

fn run() -> Result<(), String> {
    let dir: PathBuf = std::env::temp_dir().join(format!("Nagi-Setup-{}", std::process::id()));
    fs::create_dir_all(&dir).map_err(|e| format!("Impossible de préparer l'installation : {e}"))?;
    fs::write(dir.join("Nagi-Setup-0.3.3.exe"), SETUP).map_err(|e| format!("Impossible de décompresser Nagi : {e}"))?;
    fs::write(dir.join("Nagi-Setup-0.3.3-0.bin"), PART0).map_err(|e| format!("Impossible de décompresser Nagi : {e}"))?;
    fs::write(dir.join("Nagi-Setup-0.3.3-1.bin"), PART1).map_err(|e| format!("Impossible de décompresser Nagi : {e}"))?;
    let status = Command::new(dir.join("Nagi-Setup-0.3.3.exe"))
        .current_dir(&dir)
        .status()
        .map_err(|e| format!("Impossible de lancer l'assistant d'installation : {e}"))?;
    let _ = fs::remove_dir_all(&dir);
    if !status.success() {
        return Err(format!("L'assistant d'installation s'est terminé avec le code {}.", status.code().unwrap_or(-1)));
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() { show_error(&error); }
}




