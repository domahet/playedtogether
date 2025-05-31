use riven::consts::RegionalRoute;
use riven::RiotApi;
use std::collections::HashSet;
use std::error::Error;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use chrono::{TimeZone, Utc};

use crate::riot_id::RiotId;
use crate::utils::print_in_box;

pub async fn run_query(
    riot_api: &RiotApi,
    player1_riot_id: RiotId,
    player2_riot_id: RiotId,
    regional_route: RegionalRoute, // <-- Add this parameter
    verbose: bool,
    silent: bool,
) -> Result<(), Box<dyn Error>> {
    let player1_game_name = &player1_riot_id.game_name;
    let player1_tag_line = &player1_riot_id.tag_line;
    // Removed player1_regional_route and player2_regional_route variables

    let player2_game_name = &player2_riot_id.game_name;
    let player2_tag_line = &player2_riot_id.tag_line;

    // 1. Get PUUIDs for both players
    if verbose {
        println!(
            "Fetching PUUID for {}#{}",
            player1_game_name, player1_tag_line
        );
    }
    let account1 = riot_api
        .account_v1()
        .get_by_riot_id(regional_route, player1_game_name, player1_tag_line) // <-- Use regional_route
        .await?
        .ok_or_else(|| {
            format!(
                "Error: Player 1 Riot ID '{}' not found on regional route '{:?}'. Please check spelling, tag line, and ensure the account exists and is active in this region.",
                player1_riot_id, regional_route // <-- Use regional_route here
            )
        })?;
    let puuid1 = &account1.puuid;
    if verbose {
        println!("Player 1 PUUID: {}", puuid1);
    }

    if verbose {
        println!(
            "Fetching PUUID for {}#{}",
            player2_game_name, player2_tag_line
        );
    }
    let account2 = riot_api
        .account_v1()
        .get_by_riot_id(regional_route, player2_game_name, player2_tag_line) // <-- Use regional_route
        .await?
        .ok_or_else(|| {
            format!(
                "Error: Player 2 Riot ID '{}' not found on regional route '{:?}'. Please check spelling, tag line, and ensure the account exists and is active in this region.",
                player2_riot_id, regional_route // <-- Use regional_route here
            )
        })?;
    let puuid2 = &account2.puuid;
    if verbose {
        println!("Player 2 PUUID: {}", puuid2);
    }

    // This warning is no longer strictly necessary if both use the same regional_route parameter.
    // However, if you ever wanted to allow different regional routes for players, you'd re-add this.
    // For now, we assume both players are on the same region passed by the CLI.
    // if player1_regional_route != player2_regional_route {
    //     eprintln!("Warning: Players are on different regional routes. Match history search may be inconsistent or fail.");
    // }

    // 2. Get a list of recent match IDs for Player 1
    let one_month_ago = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(30 * 24 * 60 * 60)) // Approx 30 days
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);

    if verbose {
        println!(
            "Fetching match IDs for Player 1 (last 100 matches, roughly last 30 days if available)..."
        );
    }
    let match_ids = riot_api
        .match_v5()
        .get_match_ids_by_puuid(
            regional_route, // <-- Use regional_route
            puuid1,
            Some(100), // Max number of matches to retrieve
            None,      // end_time: None (up to now)
            None,      // queue: None (any queue type)
            one_month_ago, // start_time: roughly a month ago
            None,      // start: None (start from beginning of list)
            None,      // type: None (any match type)
        )
        .await?;

    if match_ids.is_empty() {
        println!(
            "No recent matches found for {}#{}",
            player1_game_name, player1_tag_line
        );
        return Ok(());
    }

    if verbose {
        println!("Found {} recent matches for Player 1.", match_ids.len());
    }

    let mut found_together_count = 0;
    let mut games_won_count = 0;
    let mut checked_matches = 0;
    let mut found_game_links: Vec<String> = Vec::new(); // To store links for silent mode

    // 3. For each match ID, retrieve the full match details
    for match_id in match_ids {
        checked_matches += 1;
        if verbose {
            println!(
                "Checking match {} ({} of {})...",
                match_id,
                checked_matches,
                100 // Hardcoded max matches from get_match_ids_by_puuid
            );
        }

        let match_data_option = riot_api
            .match_v5()
            .get_match(regional_route, &match_id) // <-- Use regional_route
            .await?;

        if let Some(match_data) = match_data_option {
            // 4. Check if the second player's PUUID is among the participants
            let info = match_data.info;
            let participants_puuids: HashSet<&str> =
                info.participants.iter().map(|p| p.puuid.as_str()).collect();

            if participants_puuids.contains(&puuid2.as_str()) {
                found_together_count += 1;
                let mut lines_of_text: Vec<String> = Vec::new();

                // Format the game start timestamp
                let game_start_datetime =
                    Utc.timestamp_millis_opt(info.game_start_timestamp)
                       .single()
                       .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                       .unwrap_or_else(|| "Unknown Date".to_string());


                // Line 1: Players and Match ID
                lines_of_text.push(format!(
                    "Players {} and {} played together in Match ID: {}",
                    player1_riot_id, player2_riot_id, match_id
                ));
                lines_of_text.push(format!("Date: {}", game_start_datetime));

                // Line 2: Game Mode and Game Type
                lines_of_text.push(format!(
                    "Game Mode: {:?}, Game Type: {:?}",
                    info.game_mode, info.game_type
                ));

                let mut leagueofgraphs_link = None;
                // Note: League of Graphs link currently hardcodes EUNE.
                // To make this dynamic, you'd need a mapping from RegionalRoute to LoG's region codes.
                if let Some((_region_id, stripped_match_id)) = match_id.split_once('_') {
                    // Example: if regional_route maps to "EUW" or "EUNE" or "NA" etc.
                    // For now, it remains EUNE as it's a fixed part of the URL.
                    leagueofgraphs_link =
                        Some(format!("https://www.leagueofgraphs.com/match/EUNE/{}", stripped_match_id));
                    lines_of_text.push(leagueofgraphs_link.clone().unwrap());
                }

                // Find participant data for Player 1
                let player1_participant = info
                    .participants
                    .iter()
                    .find(|p| p.puuid == *puuid1);

                // Find participant data for Player 2
                let player2_participant = info
                    .participants
                    .iter()
                    .find(|p| p.puuid == *puuid2);

                if let (Some(p1_data), Some(p2_data)) = (player1_participant, player2_participant)
                {
                    if p1_data.win {
                        games_won_count += 1;
                    }
                    if verbose {
                        // Participant Details Header
                        lines_of_text.push("--- Participant Details ---".to_string());

                        // Player 1 Details
                        lines_of_text.push(format!("{}:", player1_game_name));
                        lines_of_text.push(format!("  Champion: {}", p1_data.champion_name));
                        lines_of_text.push(format!("  Role: {}", p1_data.team_position));
                        lines_of_text.push(format!(
                            "  KDA: {}/{}/{}",
                            p1_data.kills, p1_data.deaths, p1_data.assists
                        ));

                        // Player 2 Details
                        lines_of_text.push(format!("{}:", player2_game_name));
                        lines_of_text.push(format!("  Champion: {}", p2_data.champion_name));
                        lines_of_text.push(format!("  Role: {}", p2_data.team_position));
                        lines_of_text.push(format!(
                            "  KDA: {}/{}/{}",
                            p2_data.kills, p2_data.deaths, p2_data.assists
                        ));

                        // Match Outcome Header
                        lines_of_text.push("--- Match Outcome ---".to_string());
                        // Match Outcome Detail
                        lines_of_text.push(format!(
                            "  Won the game?: {}",
                            if p1_data.win { "YES" } else { "NO" }
                        ));

                        // Print the lines in a box
                        if !lines_of_text.is_empty() {
                            print_in_box(
                                &lines_of_text
                                    .iter()
                                    .map(String::as_str)
                                    .collect::<Vec<&str>>(),
                            );
                        } else {
                            println!("No detailed information available for this match.");
                        }
                        println!("\n");
                    }
                    if let Some(link) = leagueofgraphs_link {
                        found_game_links.push(link);
                    }
                } else {
                    eprintln!(
                        "Error: Could not find participant data for one or both players in match {}.",
                        match_id
                    );
                }
            }
        } else {
            eprintln!("Warning: Match {} not found or accessible.", match_id);
        }
    }

    // Final Summary
    if silent || !verbose {
        // Always print summary in silent, and in default (non-verbose) mode.
        println!("\n--- Query Summary ---");
        println!("Checked {} matches for {}.", checked_matches, player1_riot_id);
        println!(
            "Found {} matches where {} and {} played together.",
            found_together_count, player1_riot_id, player2_riot_id
        );
        println!("Of those, {} games were won by {}.", games_won_count, player1_riot_id);

        if silent {
            println!("\n--- Found Game Links ---");
            if found_game_links.is_empty() {
                println!("No games found together.");
            } else {
                for link in found_game_links {
                    println!("{}", link);
                }
            }
        }
    }

    Ok(())
}