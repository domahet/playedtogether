use clap::{CommandFactory, Parser};
use riven::RiotApi;
use std::error::Error;
use std::env;
use riven::consts::RegionalRoute;
use colored::Colorize;

mod cli;
mod config;
mod riot_id;
mod api_client;
mod utils;

use cli::{Cli, UserFacingRegion};
use config::Config;
use riot_id::RiotId;
use api_client::run_query;
use utils::print_in_box;

use serde_json;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut config = Config::load()?;
    let cli = Cli::parse();

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
            Cli::command().print_help()?;
            return Ok(());
        }
        1 => {
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
            player1_riot_id = cli.riot_ids[0].clone();
            player2_riot_id = cli.riot_ids[1].clone();
        }
        _ => {
            Cli::command().print_help()?;
            return Ok(());
        }
    }

    let user_selected_region: Option<UserFacingRegion> = cli.region.or(cli.default_region.clone());

    let regional_route = user_selected_region.as_ref()
                                               .map(|r| r.to_regional_route())
                                               .unwrap_or(RegionalRoute::EUROPE);

    let api_key = env::var("RGAPI_KEY")
        .expect("RGAPI_KEY environment variable not found. Please set it.");
    let riot_api = RiotApi::new(api_key);

    let output = run_query(
        &riot_api,
        player1_riot_id,
        player2_riot_id,
        regional_route,
        user_selected_region,
        cli.number,
    ).await?;

    if cli.json {
        let json_output = serde_json::to_string_pretty(&output)?;
        println!("{}", json_output);
    } else {

        let summary = &output.query_summary;
        let found_matches = &output.found_matches;

        if cli.silent || !cli.verbose {
            println!("\n--- Query Summary ---");
            println!(
                "Checked {} matches for {}#{}.",
                summary.checked_matches_count, summary.player1.game_name, summary.player1.tag_line
            );
            println!(
                "Found {} matches where {}#{} and {}#{} played together.",
                summary.matches_played_together_count,
                summary.player1.game_name, summary.player1.tag_line,
                summary.player2.game_name, summary.player2.tag_line
            );
            println!(
                "Of those, {} games were won by {}#{}.",
                summary.player1_wins_together_count, summary.player1.game_name, summary.player1.tag_line
            );

            if cli.silent {
                println!("\n--- Found Game Links ---");
                if found_matches.is_empty() {
                    println!("No games found together.");
                } else {
                    for match_detail in found_matches {
                        if let Some(link) = &match_detail.league_of_graphs_link {
                            println!("{}", link);
                        }
                    }
                }
            }
        }

        if cli.verbose {
            println!("\n--- Detailed Match Information ---");
            if found_matches.is_empty() {
                println!("No detailed matches found.");
            } else {
                for match_detail in found_matches {
                    let mut lines_of_text: Vec<String> = Vec::new();
                    lines_of_text.push(format!(
                        "Players {}#{} and {}#{} played together in Match ID: {}",
                        summary.player1.game_name, summary.player1.tag_line,
                        summary.player2.game_name, summary.player2.tag_line,
                        match_detail.match_id
                    ));

                    lines_of_text.push(format!("Date: {}", match_detail.game_date_utc));
                    lines_of_text.push(format!(
                        "Game Mode: {}, Game Type: {}",
                        match_detail.game_mode,
                        match_detail.game_type.as_deref().unwrap_or("N/A")
                    ));

                    if let Some(link) = &match_detail.league_of_graphs_link {
                        lines_of_text.push(link.clone());
                    }

                    lines_of_text.push("--- Participant Details ---".to_string());
                    lines_of_text.push(format!("{}:", summary.player1.game_name));

                    lines_of_text.push(format!("  Champion: {}", match_detail.player1_details.champion));
                    lines_of_text.push(format!("  Role: {}", match_detail.player1_details.role));
                    lines_of_text.push(format!(
                        "  KDA: {}/{}/{}",
                        match_detail.player1_details.kills, match_detail.player1_details.deaths, match_detail.player1_details.assists
                    ));


                    lines_of_text.push(format!("{}:", summary.player2.game_name));

                    lines_of_text.push(format!("  Champion: {}", match_detail.player2_details.champion));
                    lines_of_text.push(format!("  Role: {}", match_detail.player2_details.role));
                    lines_of_text.push(format!(
                        "  KDA: {}/{}/{}",
                        match_detail.player2_details.kills, match_detail.player2_details.deaths, match_detail.player2_details.assists
                    ));

                    lines_of_text.push("--- Match Outcome ---".to_string());
                    let outcome_text = if match_detail.player1_details.outcome == "Victory" {
                        "Victory".green().to_string()
                    } else {
                        "Defeat".red().to_string()
                    };
                    lines_of_text.push(format!("  Outcome: {}", outcome_text));

                    print_in_box(
                        &lines_of_text
                            .iter()
                            .map(String::as_str)
                            .collect::<Vec<&str>>(),
                    );
                    println!();
                }
            }
        }
    }
    Ok(())
}