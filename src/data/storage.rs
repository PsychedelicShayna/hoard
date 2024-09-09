use serde_json as sj;

use std::collections::HashMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use super::models::Command;

use anyhow as ah;

pub struct CommandDatabase {
    file_path: String,
    commands: HashMap<String, Command>,
}

// An in-memory representation of the stored commands.
// When a command is added or updated, it is done so by telling an instance
// of this struct to update itself with the new command.
//
// If the command is not already in the database, it is added.
// If it already in the database, it is updated.
//
// The file path is determined by the environment variable `VIHOARD_DATABASE`.
// or defaults to `~/.local/share/vihoard/commands.json

const DEFAULT_DATABASE_DIR_PATH: &str = ".local/share/vihoard";
const DEFAULT_DATABASE_FILE_NAME: &str = "commands.json";
const DATABASE_ENV_VAR: &str = "VIHOARD_DATABASE";

impl CommandDatabase {
    pub fn write(&self) -> ah::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file_path)
            .expect("Could not open file for writing.");

        let mut writer = BufWriter::new(&mut file);
        sj::to_writer_pretty(&mut writer, &self.commands).expect("Could not write to file.");
        Ok(())
    }

    pub fn get_commands(&self) -> Vec<Command> {
        self.commands.values().cloned().collect()
    }

    pub fn read(&mut self) -> ah::Result<()> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let commands: HashMap<String, Command> = sj::from_reader(reader)?;
        self.commands = commands;
        Ok(())
    }

    pub fn update(&mut self, command: Command) -> ah::Result<()> {
        self.commands.insert(command.binary.clone(), command);
        self.write()?;
        Ok(())
    }

    pub fn update_multiple(&mut self, commands: Vec<Command>) -> ah::Result<()> {
        for command in commands {
            self.commands.insert(command.binary.clone(), command);
        }

        self.write()?;
        Ok(())
    }

    pub fn new() -> ah::Result<Self> {
        let home_dir = env::var("HOME")?;
        let database_dir_path = format!("{}/{}", home_dir, DEFAULT_DATABASE_DIR_PATH);

        let database_file_path: String = match env::var(DATABASE_ENV_VAR) {
            Ok(path) => path,
            Err(_) => {
                let home = env::var("HOME").unwrap();
                format!("{}/{}", database_dir_path, DEFAULT_DATABASE_FILE_NAME)
            }
        };

        let database_dir_path: PathBuf = PathBuf::from(database_dir_path);

        if !database_dir_path.is_dir() {
            fs::create_dir_all(&database_dir_path)?;

            if !database_dir_path.is_dir() {
                ah::bail!(
                    "Could not create database directory at {}",
                    database_dir_path.display()
                );
            }
        }

        let database_file_path_pb: PathBuf = PathBuf::from(&database_file_path);

        if !database_file_path_pb.is_file() {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&database_file_path_pb)?;

            // Default to an empty JSON object
            file.write_all(b"{}")
                .expect("Could not write empty JSON object when creating database file.");
        }

        let mut command_database = CommandDatabase {
            file_path: database_file_path,
            commands: HashMap::new(),
        };

        // Ensure everything went smoothly, and we can read/write through the
        // functions that will be used to interact with the database.
        command_database.read()?;
        command_database.write()?;

        Ok(command_database)
    }
}
