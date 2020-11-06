use std::{env, include_str};
use std::path::{Path, PathBuf};
use std::fs::{create_dir, remove_file, remove_dir, File};
use std::io::{copy, Write};
use chrono::{DateTime, Utc};
use serde_json::{from_reader, Value};
use walkdir::{DirEntry, WalkDir};
use zip::write::{ZipWriter, FileOptions};

fn main() {
    // Include all new mod files
    let info_json_template = include_str!("new_mod_example/info.json");
    let changelog_template = include_str!("new_mod_example/changelog.txt");

    let mut args = env::args().skip(1);

    match args.next().unwrap().as_str() {
        "new" => new_mod(args.next().unwrap(), info_json_template, changelog_template),
        "build" => build_mod(),
        "run" => println!("[TODO] Build mod and run a game with it"),
        _ => println!("[TODO] No action specified")
    }
}

// Create new mod project
fn new_mod(mod_name: String, info_json_template: &str, changelog_template: &str) {
    let mod_path = Path::new(&mod_name);    
    
    let current_time: DateTime<Utc> = Utc::now();

    create_dir(&mod_path).expect("Failed to create project dir");
    create_dir(mod_path.join("prototypes")).expect("Failed to create prototypes dir");

    // TODO: mod author
    let info_json_content = info_json_template
        .replace("mod_name", &mod_name)
        .replace("mod_title", &mod_name)
        .replace("mod_author", "TODO")
        .replace("mod_desc", &mod_name);

    // TODO: date
    let changelog_content = changelog_template
        .replace("blank_date", &format!("{}", current_time.format("%d.%m.%Y")));

    let mut info_json_file = File::create(mod_path.join("info.json")).expect("failed to create info.json");
    let mut changelog_file = File::create(mod_path.join("changelog.txt")).expect("failed to create changelog.txtx");

    info_json_file.write_all(info_json_content.as_bytes()).expect("failed to write info.json");
    changelog_file.write_all(changelog_content.as_bytes()).expect("failed to write changelog.txt");

    let mut data_lua_file = File::create(mod_path.join("data.lua"))
        .expect("Failed to create data.lua");
    data_lua_file.write_all("-- Here goes prototype data".as_bytes())
        .expect("Failed to write data.lua");

    let mut control_lua_file = File::create(mod_path.join("control.lua"))
        .expect("Failed to create control.lua");
    control_lua_file.write_all("-- Here goes all runtime game scripts. For more info, consult https://lua-api.factorio.com".as_bytes())
        .expect("Failed to write control.lua");

    println!("Succesfully created project {}", mod_name);
}

// Function to filter all files we don't want to add to archive
fn is_hidden(entry: &DirEntry, zip_file_name: &String) -> bool {
    entry.file_name().to_str().unwrap() == zip_file_name || (entry.file_name().to_str().unwrap() != "." &&  entry.file_name().to_str().unwrap().starts_with("."))
}

// Build mod. Repurposed from rfmp
fn build_mod() {
    // Open info.json and parse it
    let info_file = File::open("info.json").expect("Error opening info.json");
    let info: Value = from_reader(info_file).expect("Error parsing info.json");

    // Get mod name/id and version
    let mod_name = info["name"].as_str().unwrap();
    let mod_version = info["version"].as_str().unwrap();
    
    let mut zip_file_path = PathBuf::from(".build");
    if !zip_file_path.exists() {
        create_dir(&zip_file_path).unwrap();
    }

    // Mod file name
    let zip_file_name = format!("{}_{}.zip", mod_name, mod_version);
    zip_file_path.push(&zip_file_name);

    // Walkdir iter, filtered
    let walkdir = WalkDir::new(".");
    let it = walkdir.into_iter().filter_entry(|e| !is_hidden(e, &zip_file_name));

    // Delete existing file
    if zip_file_path.exists() {
        println!("{} exists, removing.", zip_file_path.to_str().unwrap());
        if zip_file_path.is_file() {
            remove_file(&zip_file_path).unwrap();
        } else if zip_file_path.is_dir() { // Is this even possible?
            remove_dir(&zip_file_path).unwrap();
        }
    }

    // Create mod file
    let zip_file = File::create(zip_file_path).unwrap();

    // Archive options. Deflated is best combination of speed and compression (for zip)
    // It would be cool if Factorio allowed other compression formats, like zstd
    let zip_options = FileOptions::default();

    // Create writer
    let mut zipwriter = ZipWriter::new(zip_file);  

    // Let the zipping begin!
    for entry in it {
        let entry = entry.unwrap();
        let name = entry.path();
        if name == Path::new(".") {continue;}
        //name.strip_prefix(Path::new("./")).unwrap();
        let zipped_name = Path::new(&format!("{}_{}", mod_name, mod_version)).join(Path::new(name.to_str().unwrap().strip_prefix("./").unwrap()));

        if name.is_file() {
            //println!("adding file {:?}", name);
            zipwriter.start_file(zipped_name.to_str().unwrap(), zip_options).unwrap();
            let mut f = File::open(name).unwrap();

            copy(&mut f, &mut zipwriter).unwrap();
        } else if name.as_os_str().len() != 0 {
            //println!("adding dir  {:?}", name);
            zipwriter.add_directory(zipped_name.to_str().unwrap(), zip_options).unwrap();
        }
    }

    // Finish writing
    zipwriter.finish().unwrap();
}
