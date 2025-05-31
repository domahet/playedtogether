use riven::consts::RegionalRoute;
use riven::RiotApi;
use std::collections::HashSet;
use std::error::Error;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use chrono::{TimeZone, Utc};

use serde::{Serialize};

use crate::riot_id::RiotId;
use crate::cli::UserFacingRegion;
use colored::Colorize;
use crate::utils::print_in_box;


// --- JSON Output Structures ---

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")] // Convert struct field names to camelCase for JSON keys
pub struct OverallOutput {
    pub query_summary: QuerySummary,
    pub found_matches: Vec<MatchDetails>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySummary {
    pub player1: PlayerIdentity,
    pub player2: PlayerIdentity,
    pub regional_route: String,
    pub checked_matches_count: u32,
    pub matches_played_together_count: u32,
    pub player1_wins_together_count: u32,
    pub player1_puuid_found: bool,
    pub player2_puuid_found: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerIdentity {
    pub game_name: String,
    pub tag_line: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchDetails {
    pub match_id: String,
    pub game_date_utc: String,
    pub game_mode: String,
    pub game_type: Option<String>,
    pub league_of_graphs_link: Option<String>,
    pub player1_details: ParticipantDetails,
    pub player2_details: ParticipantDetails,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantDetails {
    pub champion: String,
    pub role: String,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub outcome: String,
}

// --- End JSON Output Structures ---


pub async fn run_query(
    riot_api: &RiotApi,
    player1_riot_id: RiotId,
    player2_riot_id: RiotId,
    regional_route: RegionalRoute,
    user_selected_region: Option<UserFacingRegion>,
    number_of_matches: Option<i32>,
    verbose: bool,
    _silent: bool,
    json_output_enabled: bool,
) -> Result<OverallOutput, Box<dyn Error>> {
    let player1_game_name = player1_riot_id.game_name.clone();
    let player1_tag_line = player1_riot_id.tag_line.clone();

    let player2_game_name = player2_riot_id.game_name.clone();
    let player2_tag_line = player2_riot_id.tag_line.clone();

    if verbose {
        println!("Fetching PUUID for {}#{}", player1_game_name, player1_tag_line);
    }
    let player1_puuid_found = true;
    let account1 = riot_api
        .account_v1()
        .get_by_riot_id(regional_route, &player1_game_name, &player1_tag_line)
        .await?;

    let puuid1 = match account1 {
        Some(acc) => {
            if verbose {
                println!("Player 1 PUUID: {}", acc.puuid);
            }
            acc.puuid
        },
        _none => {
            return Err(format!(
                "Error: Player 1 Riot ID '{}' not found on regional route '{:?}'. Please check spelling, tag line, and ensure the account exists and is active in this region.",
                player1_riot_id, regional_route
            ).into());
        }
    };

    if verbose {
        println!("Fetching PUUID for {}#{}", player2_game_name, player2_tag_line);
    }
    let player2_puuid_found = true;
    let account2 = riot_api
        .account_v1()
        .get_by_riot_id(regional_route, &player2_game_name, &player2_tag_line)
        .await?;
    
    let puuid2 = match account2 {
        Some(acc) => {
            if verbose {
                println!("Player 2 PUUID: {}", acc.puuid);
            }
            acc.puuid
        },
        _none => {
            return Err(format!(
                "Error: Player 2 Riot ID '{}' not found on regional route '{:?}'. Please check spelling, tag line, and ensure the account exists and is active in this region.",
                player2_riot_id, regional_route
            ).into());
        }
    };

    if verbose {
        println!("Fetching match IDs for Player 1 (last {} matches, roughly last 30 days if available)...", number_of_matches.unwrap_or(100));
    }
    let one_month_ago = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(30 * 24 * 60 * 60))
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);

    let match_ids = riot_api
        .match_v5()
        .get_match_ids_by_puuid(
            regional_route,
            &puuid1,
            number_of_matches,
            None,
            None,
            one_month_ago,
            None,
            None,
        )
        .await?;

    if verbose {
        println!("Found {} recent matches for Player 1.", match_ids.len());
    }

    let mut found_together_count = 0;
    let mut player1_games_won_count = 0;
    let mut checked_matches_count = 0;
    let mut found_matches_details: Vec<MatchDetails> = Vec::new();

    let total_match_ids = match_ids.len();

    for match_id_str in match_ids {
        checked_matches_count += 1;
        if verbose {
            // Updated to use match_ids.len() for total count
            println!("Checking match {} ({} of {})...", match_id_str, checked_matches_count, total_match_ids);
        }

        let match_data_option = riot_api
            .match_v5()
            .get_match(regional_route, &match_id_str)
            .await?;

        if let Some(match_data) = match_data_option {
            let info = match_data.info;
            let participants_puuids: HashSet<&str> =
                info.participants.iter().map(|p| p.puuid.as_str()).collect();

            if participants_puuids.contains(&puuid2.as_str()) {
                found_together_count += 1;

                let game_start_datetime =
                    Utc.timestamp_millis_opt(info.game_start_timestamp)
                       .single()
                       .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                       .unwrap_or_else(|| "Unknown Date".to_string());

                let mut league_of_graphs_link = None; 
                if let Some((_region_id, stripped_match_id)) = match_id_str.split_once('_') {
                    let log_region = user_selected_region
                        .as_ref()
                        .map(|r| r.to_log_string())
                        .unwrap_or("eune");
                    league_of_graphs_link = Some(format!("https://www.leagueofgraphs.com/match/{}/{}", log_region, stripped_match_id));
                }
                
                let player1_participant = info
                    .participants
                    .iter()
                    .find(|p| p.puuid == puuid1);

                let player2_participant = info
                    .participants
                    .iter()
                    .find(|p| p.puuid == puuid2);

                if let (Some(p1_data), Some(p2_data)) = (player1_participant, player2_participant) {
                    if p1_data.win {
                        player1_games_won_count += 1;
                    }

                    let p1_outcome = if p1_data.win { "Victory" } else { "Defeat" }.to_string();
                    let p2_outcome = if p2_data.win { "Victory" } else { "Defeat" }.to_string();

                    // Create MatchDetails struct
                    let current_match_details = MatchDetails {
                        match_id: match_id_str.clone(),
                        game_date_utc: game_start_datetime,
                        game_mode: format!("{:?}", info.game_mode),
                        game_type: info.game_type.map(|gt| format!("{:?}", gt)),
                        league_of_graphs_link,
                        player1_details: ParticipantDetails {
                            champion: p1_data.champion_name.clone(),
                            role: p1_data.team_position.to_string(),
                            kills: p1_data.kills,
                            deaths: p1_data.deaths,
                            assists: p1_data.assists,
                            outcome: p1_outcome,
                        },
                        player2_details: ParticipantDetails {
                            champion: p2_data.champion_name.clone(),
                            role: p2_data.team_position.to_string(),
                            kills: p2_data.kills,
                            deaths: p2_data.deaths,
                            assists: p2_data.assists,
                            outcome: p2_outcome,
                        },
                    };

                    if verbose && !json_output_enabled {
                        let mut lines_of_text: Vec<String> = Vec::new();
                        lines_of_text.push(format!(
                            "Players {}#{} and {}#{} played together in Match ID: {}",
                            player1_game_name, player1_tag_line,
                            player2_game_name, player2_tag_line,
                            current_match_details.match_id
                        ));
                        lines_of_text.push(format!("Date: {}", current_match_details.game_date_utc));
                        lines_of_text.push(format!(
                            "Game Mode: {}, Game Type: {}",
                            current_match_details.game_mode,
                            current_match_details.game_type.as_deref().unwrap_or("N/A")
                        ));

                        if let Some(link) = &current_match_details.league_of_graphs_link {
                            lines_of_text.push(link.clone());
                        }

                        lines_of_text.push("--- Participant Details ---".to_string());
                        lines_of_text.push(format!("{}:", player1_game_name));
                        lines_of_text.push(format!("  Champion: {}", current_match_details.player1_details.champion));
                        lines_of_text.push(format!("  Role: {}", current_match_details.player1_details.role));
                        lines_of_text.push(format!(
                            "  KDA: {}/{}/{}",
                            current_match_details.player1_details.kills, current_match_details.player1_details.deaths, current_match_details.player1_details.assists
                        ));

                        lines_of_text.push(format!("{}:", player2_game_name));
                        lines_of_text.push(format!("  Champion: {}", current_match_details.player2_details.champion));
                        lines_of_text.push(format!("  Role: {}", current_match_details.player2_details.role));
                        lines_of_text.push(format!(
                            "  KDA: {}/{}/{}",
                            current_match_details.player2_details.kills, current_match_details.player2_details.deaths, current_match_details.player2_details.assists
                        ));

                        lines_of_text.push("--- Match Outcome ---".to_string());
                        let outcome_text = if current_match_details.player1_details.outcome == "Victory" {
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
                        println!(); // Add a newline after each box for spacing
                    }


                    found_matches_details.push(current_match_details); // Still collect for JSON output
                } else {
                    if verbose {
                        // Changed from `eprintln!("Warning: Participant data incomplete for match ID '{}'. Skipping this match.", match_id_str);`
                        // to the current message for clarity based on original output.
                        eprintln!("Warning: Could not find participant data for player 1 or player 2 in match '{}'. Skipping this match.", match_id_str);
                    }
                    continue;
                }
            }
        } else {
            if verbose {
                eprintln!("Warning: Match {} not found or accessible. Skipping.", match_id_str);
            }
        }
    }

    let query_summary = QuerySummary {
        player1: PlayerIdentity {
            game_name: player1_game_name,
            tag_line: player1_tag_line,
        },
        player2: PlayerIdentity {
            game_name: player2_game_name,
            tag_line: player2_tag_line,
        },
        regional_route: format!("{:?}", regional_route),
        checked_matches_count: checked_matches_count as u32,
        matches_played_together_count: found_together_count as u32,
        player1_wins_together_count: player1_games_won_count as u32,
        player1_puuid_found,
        player2_puuid_found,
    };

    Ok(OverallOutput {
        query_summary,
        found_matches: found_matches_details,
    })
}