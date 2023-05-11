// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use league_client_connector::LeagueClientConnector;
use reqwest::IntoUrl;
use serde::Deserialize;
use thiserror::Error;

#[tauri::command]
async fn summoner_name() -> String {
    let client = NewGameClient::new();

    client.summoner_name().await.unwrap()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, summoner_name])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// NOTE(reno): A lot of this code is copied & lightly modified from the lol-game-client crate

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Failed to query the API. Is the game running ? '{}'", _0)]
    Reqwest(#[from] reqwest::Error), // An error of this type may suggests that the API specs as been updated and the crate is out-of-date. Please fill an issue !
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Summoner {
    pub account_id: u32,
    pub display_name: String,
}

struct NewGameClient {
    client: reqwest::Client,
    port: u32,
    password: String,
}

impl NewGameClient {
    pub fn new() -> Self {
        let lockfile = LeagueClientConnector::parse_lockfile().unwrap();
        NewGameClient {
            client: reqwest::ClientBuilder::new()
                .danger_accept_invalid_certs(true)
                .build()
                .unwrap(),
            port: lockfile.port,
            password: lockfile.password,
        }
    }

    async fn get_data<T: for<'de> Deserialize<'de>, U: IntoUrl>(
        &self,
        endpoint: String,
    ) -> Result<T, QueryError> {
        let data = self
            .client
            .get(String::from("https://localhost:") + &self.port.to_string() + "/" + &endpoint)
            .basic_auth("riot", Some(&self.password))
            .send()
            .await?
            .json::<T>()
            .await?;
        Ok(data)
    }

    pub async fn summoner_name(&self) -> Result<String, QueryError> {
        let base_endpoint = String::from("lol-summoner/v1/current_summoner");
        let summoner = self
            .get_data::<Summoner, String>(base_endpoint)
            .await
            .unwrap();

        Ok(summoner.display_name)
    }
}
