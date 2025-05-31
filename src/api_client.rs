use riven::consts::RegionalRoute;
use riven::RiotApi;
use std::collections::HashSet;
use std::error::Error;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use chrono::{TimeZone, Utc};

use crate::riot_id::RiotId;
use crate::utils::print_in_box;
use crate::cli::UserFacingRegion;

pub async fn run_query(
    riot_api: &RiotApi,
    player1_riot_id: RiotId,
    player2_riot_id: RiotId,
    regional_route: RegionalRoute,
    user_selected_region: Option<UserFacingRegion>,
    verbose: bool,
    silent: bool,
) -> Result<(), Box<dyn Error>> {
    let player1_game_name = &player1_riot_id.game_name;
    let player1_tag_line = &player1_riot_id.tag_line;

    let player2_game_name = &player2_riot_id.game_name;
    let player2_tag_line = &player2_riot_id.tag_line;

    if verbose {
        println!(
            "Fetching PUUID for {}#{}",
            player1_game_name, player1_tag_line
        );
    }
    let account1 = riot_api
        .account_v1()
        .get_by_riot_id(regional_route, player1_game_name, player1_tag_line)
        .await?
        .ok_or_else(|| {
            format!(
                "Error: Player 1 Riot ID '{}' not found on regional route '{:?}'. Please check spelling, tag line, and ensure the account exists and is active in this region.",
                player1_riot_id, regional_route
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
        .get_by_riot_id(regional_route, player2_game_name, player2_tag_line)
        .await?
        .ok_or_else(|| {
            format!(
                "Error: Player 2 Riot ID '{}' not found on regional route '{:?}'. Please check spelling, tag line, and ensure the account exists and is active in this region.",
                player2_riot_id, regional_route
            )
        })?;
    let puuid2 = &account2.puuid;
    if verbose {
        println!("Player 2 PUUID: {}", puuid2);
    }

    let one_month_ago = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(30 * 24 * 60 * 60))
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);

    if verbose {
        println!(
            "Fetching match IDs for Player 1 (last 100 matches, roughly last 30 days if available)..."
        );
    }
    let match_ids = riot_api
        .match_v5()
        .get_match_ids_by_puuid(
            regional_route,
            puuid1,
            Some(100),
            None,
            None,
            one_month_ago,
            None,
            None,
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
    let mut found_game_links: Vec<String> = Vec::new();

    let total_matches_to_check = match_ids.len();

    for match_id in match_ids {
        checked_matches += 1;
        if verbose {
            println!(
                "Checking match {} ({} of {})...",
                match_id,
                checked_matches,
                total_matches_to_check
            );
        }

        let match_data_option = riot_api
            .match_v5()
            .get_match(regional_route, &match_id)
            .await?;

        if let Some(match_data) = match_data_option {
            let info = match_data.info;
            let participants_puuids: HashSet<&str> =
                info.participants.iter().map(|p| p.puuid.as_str()).collect();

            if participants_puuids.contains(&puuid2.as_str()) {
                found_together_count += 1;
                let mut lines_of_text: Vec<String> = Vec::new();

                let game_start_datetime =
                    Utc.timestamp_millis_opt(info.game_start_timestamp)
                       .single()
                       .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                       .unwrap_or_else(|| "Unknown Date".to_string());


                lines_of_text.push(format!(
                    "Players {} and {} played together in Match ID: {}",
                    player1_riot_id, player2_riot_id, match_id
                ));
                lines_of_text.push(format!("Date: {}", game_start_datetime));

                lines_of_text.push(format!(
                    "Game Mode: {:?}, Game Type: {:?}",
                    info.game_mode, info.game_type
                ));

                let mut leagueofgraphs_link = None;
                if let Some((_region_id, stripped_match_id)) = match_id.split_once('_') {
                    let log_region = user_selected_region
                        .as_ref()
                        .map(|r| r.to_log_string())
                        .unwrap_or("eune");

                    leagueofgraphs_link =
                        Some(format!("https://www.leagueofgraphs.com/match/{}/{}", log_region, stripped_match_id));
                    lines_of_text.push(leagueofgraphs_link.clone().unwrap());
                }

                let player1_participant = info
                    .participants
                    .iter()
                    .find(|p| p.puuid == *puuid1);

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
                        lines_of_text.push("--- Participant Details ---".to_string());

                        lines_of_text.push(format!("{}:", player1_game_name));
                        lines_of_text.push(format!("  Champion: {}", p1_data.champion_name));
                        lines_of_text.push(format!("  Role: {}", p1_data.team_position));
                        lines_of_text.push(format!(
                            "  KDA: {}/{}/{}",
                            p1_data.kills, p1_data.deaths, p1_data.assists
                        ));

                        lines_of_text.push(format!("{}:", player2_game_name));
                        lines_of_text.push(format!("  Champion: {}", p2_data.champion_name));
                        lines_of_text.push(format!("  Role: {}", p2_data.team_position));
                        lines_of_text.push(format!(
                            "  KDA: {}/{}/{}",
                            p2_data.kills, p2_data.deaths, p2_data.assists
                        ));

                        lines_of_text.push("--- Match Outcome ---".to_string());
                        lines_of_text.push(format!(
                            "  Won the game?: {}",
                            if p1_data.win { "YES" } else { "NO" }
                        ));

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

    if silent || !verbose {
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