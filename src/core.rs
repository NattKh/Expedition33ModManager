use std::error::Error;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use walkdir;

const UE4SS_URL: &str = "https://github.com/UE4SS-RE/RE-UE4SS/releases/download/experimental-latest/zDEV-UE4SS_v3.0.1-394-g437a8ff.zip";

pub fn install_ue4ss(target_dir: &str) -> Result<(), Box<dyn Error>> {
    println!("Downloading UE4SS from {}...", UE4SS_URL);
    let resp = reqwest::blocking::get(UE4SS_URL)?;
    if !resp.status().is_success() {
        return Err(format!("Failed to download UE4SS: HTTP {}", resp.status()).into());
    }
    let bytes = resp.bytes()?;
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };
        println!("[DEBUG] Zip entry: {}", outpath.display());
        // Only extract files/folders under UE4SS/
        let mut components = outpath.components();
        if let Some(first) = components.next() {
            if first.as_os_str().to_ascii_lowercase() != "ue4ss" {
                continue;
            }
        } else {
            continue;
        }
        // Strip the UE4SS folder from the path
        let relative_path: std::path::PathBuf = components.collect();
        if relative_path.as_os_str().is_empty() {
            continue;
        }
        let dest_path = Path::new(target_dir).join(&relative_path);
        if file.is_dir() {
            match fs::create_dir_all(&dest_path) {
                Ok(_) => println!("[DEBUG] Created directory: {}", dest_path.display()),
                Err(e) => {
                    println!("[ERROR] Failed to create directory {}: {}", dest_path.display(), e);
                    return Err(e.into());
                }
            }
        } else {
            if let Some(parent) = dest_path.parent() {
                match fs::create_dir_all(parent) {
                    Ok(_) => println!("[DEBUG] Created parent directory: {}", parent.display()),
                    Err(e) => {
                        println!("[ERROR] Failed to create parent directory {}: {}", parent.display(), e);
                        return Err(e.into());
                    }
                }
            }
            match fs::File::create(&dest_path) {
                Ok(mut outfile) => {
                    match std::io::copy(&mut file, &mut outfile) {
                        Ok(_) => println!("[DEBUG] Wrote file: {}", dest_path.display()),
                        Err(e) => {
                            println!("[ERROR] Failed to write file {}: {}", dest_path.display(), e);
                            return Err(e.into());
                        }
                    }
                }
                Err(e) => {
                    println!("[ERROR] Failed to create file {}: {}", dest_path.display(), e);
                    return Err(e.into());
                }
            }
        }
    }
    println!("UE4SS contents installed to {}!", target_dir);
    Ok(())
}

/// Install a mod from a zip file by extracting it into the Mods folder
pub fn install_mod_from_zip(zip_path: &str, win64_dir: &str) -> Result<(), Box<dyn Error>> {
    let mods_dir = Path::new(win64_dir).join("Mods");
    println!("[DEBUG] Installing mod from zip: {} to Mods folder: {:?}", zip_path, mods_dir);
    if !mods_dir.exists() {
        println!("[DEBUG] Mods folder does not exist, creating...");
        fs::create_dir_all(&mods_dir)?;
    }
    let zip_data = fs::read(zip_path).map_err(|e| {
        println!("[ERROR] Failed to read zip file: {}", e);
        e
    })?;
    let mut zip = zip::ZipArchive::new(Cursor::new(zip_data)).map_err(|e| {
        println!("[ERROR] Failed to open zip archive: {}", e);
        e
    })?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| {
            println!("[ERROR] Failed to access file in zip: {}", e);
            e
        })?;
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => {
                println!("[DEBUG] Skipping file with invalid path in zip");
                continue;
            }
        };
        let dest_path = mods_dir.join(outpath);
        if file.is_dir() {
            if let Err(e) = fs::create_dir_all(&dest_path) {
                println!("[ERROR] Failed to create directory {:?}: {}", dest_path, e);
                return Err(e.into());
            }
        } else {
            if let Some(parent) = dest_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    println!("[ERROR] Failed to create parent directory {:?}: {}", parent, e);
                    return Err(e.into());
                }
            }
            let mut outfile = match fs::File::create(&dest_path) {
                Ok(f) => f,
                Err(e) => {
                    println!("[ERROR] Failed to create file {:?}: {}", dest_path, e);
                    return Err(e.into());
                }
            };
            if let Err(e) = std::io::copy(&mut file, &mut outfile) {
                println!("[ERROR] Failed to write file {:?}: {}", dest_path, e);
                return Err(e.into());
            }
        }
    }
    println!("[DEBUG] Mod installed successfully from {}!", zip_path);
    Ok(())
}

/// List installed mods by returning the names of all subfolders in the Mods directory
pub fn list_installed_mods(win64_dir: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mods_path = Path::new(win64_dir).join("Mods");
    let mut mods = Vec::new();
    if mods_path.exists() && mods_path.is_dir() {
        for entry in fs::read_dir(mods_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    mods.push(name.to_string());
                }
            }
        }
    }
    Ok(mods)
}

/// Recursively list all files and directories under a given root directory.
pub fn list_all_files_and_dirs<P: AsRef<std::path::Path>>(root: P) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();
    let root = root.as_ref();
    if !root.exists() {
        return Ok(entries);
    }
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry?;
        if entry.path().is_dir() { // Only include directories
            let rel_path = entry.path().strip_prefix(root).unwrap_or(entry.path());
            if rel_path.as_os_str().is_empty() {
                continue;
            }
            entries.push(rel_path.display().to_string());
        }
    }
    Ok(entries)
} 