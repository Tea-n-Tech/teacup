extern crate machine_uid;
extern crate xdg;

use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalSettings {
    pub machine_id: i64,
}

pub async fn get_settings_filepath() -> PathBuf {
    // TODO make return type result and fail if not retrievable

    // xdg defines nicely where to store stuff on computers
    // we use this to either search or store our settings.
    let base_dir = xdg::BaseDirectories::with_prefix("teacup")
        .expect("Could not determine important OS base directories which are needed");

    base_dir
        .place_config_file("settings.json")
        .expect("Could not create the settings directory")
}

pub async fn load_settings(config_path: &PathBuf) -> LocalSettings {
    match fs::read_to_string(config_path) {
        Ok(contents) => {
            serde_json::from_str(&contents).expect("Error converting settings file to json")
        }
        Err(_) => {
            let settings = LocalSettings {
                machine_id: generate_machine_id().await,
            };
            fs::write(config_path, serde_json::to_string(&settings).unwrap())
                .expect("Could not write config file with required settings");

            settings
        }
    }
}

async fn generate_machine_id() -> i64 {
    match machine_uid::get() {
        Ok(id) => {
            // TODO investigate a more appropriate hashing algorithm
            let mut hasher = Sha256::new();
            hasher.update(id);
            let hash = hasher.finalize();
            i64::from_ne_bytes(
                // unwrapping is safe as the hash of sha256 is long enough
                hash.as_slice().split_at(8).0.try_into().unwrap(),
            )
        }
        Err(err) => {
            eprintln!(
                "Error getting machine id, will create an artifical one: {:?}",
                err
            );
            let mut rng = rand::thread_rng();
            rng.gen::<i64>()
        }
    }
}
