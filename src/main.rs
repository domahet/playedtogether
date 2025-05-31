use clap::{CommandFactory, Parser};
use riven::RiotApi;
use std::error::Error;
use std::env;
use riven::consts::RegionalRoute; // Keep this import for the final RegionalRoute type

// Declare our modules
mod cli;
mod config;
mod riot_id;
mod api_client;
mod utils;

use cli::Cli; // <-- Import UserFacingRegion
use config::Config;
use riot_id::RiotId;
use api_client::run_query;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut config = Config::load()?;
    let cli = Cli::parse();

    // Handle --self flag
    if let Some(riot_id_to_store) = cli.set_self {
        config.self_riot_id = Some(riot_id_to_store.clone().into());
        config.save()?;
        println!("Stored '{}' as your self Riot ID.", riot_id_to_store);
        return Ok(());
    }

    let player1_riot_id: RiotId;
    let player2_riot_id: RiotId;

    match cli.riot_ids.len() {
        0 => {
            // No arguments: print help text
            Cli::command().print_help()?;
            return Ok(());
        }
        1 => {
            // One argument: use stored self_riot_id as player1, provided arg as player2
            if let Some(self_id_stored) = config.self_riot_id.map(RiotId::from) {
                player1_riot_id = self_id_stored;
                player2_riot_id = cli.riot_ids[0].clone();
            } else {
                eprintln!("Error: No 'self' Riot ID stored. Please set it using `--self <RIOT_ID>` or provide two Riot IDs as arguments.");
                Cli::command().print_help()?;
                return Err("Missing 'self' Riot ID".into());
            }
        }
        2 => {
            // Two arguments: use provided args as player1 and player2
            player1_riot_id = cli.riot_ids[0].clone();
            player2_riot_id = cli.riot_ids[1].clone();
        }
        _ => {
            // Should not happen due to num_args = 0..=2, but good for completeness
            Cli::command().print_help()?;
            return Ok(());
        }
    }

    // Determine the regional route with the new fallback logic and conversion
    let regional_route = cli.region
                            .map(|r| r.to_regional_route()) // Convert if --region is provided
                            .or(cli.default_region.map(|r| r.to_regional_route())) // Convert if --default-region is provided
                            .unwrap_or(RegionalRoute::EUROPE); // Ultimate fallback

    // Read the API key from an environment variable at RUNTIME.
    let api_key = env::var("RGAPI_KEY")
        .expect("RGAPI_KEY environment variable not found. Please set it.");
    let riot_api = RiotApi::new(api_key);

    // Call the modularized API logic
    run_query(
        &riot_api,
        player1_riot_id,
        player2_riot_id,
        regional_route,
        cli.verbose,
        cli.silent,
    ).await?;

    Ok(())
}