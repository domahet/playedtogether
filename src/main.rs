use riven::consts::RegionalRoute;
use riven::RiotApi;
use std::env;
use std::error::Error;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Read the API key from an environment variable at RUNTIME.
    let api_key = env::var("RGAPI_KEY")
        .expect("RGAPI_KEY environment variable not found. Please set it.");
    let riot_api = RiotApi::new(api_key);

    // --- Player 1 (the one whose match history we will iterate) ---
    let player1_game_name = "MainingYourMom"; // Replace with actual game name
    let player1_tag_line = "4444";       // Replace with actual tag line
    let player1_regional_route = RegionalRoute::EUROPE; // Or EUROPE, ASIA etc.

    // --- Player 2 (the one we are checking for) ---
    let player2_game_name = "Piciúr"; // Replace with actual game name
    let player2_tag_line = "ontop";       // Replace with actual tag line
    let player2_regional_route = RegionalRoute::EUROPE; // Must be the same regional route as player 1

    // 1. Get PUUIDs for both players
    println!(
        "Fetching PUUID for {}#{}",
        player1_game_name, player1_tag_line
    );
    let account1 = riot_api
        .account_v1()
        .get_by_riot_id(player1_regional_route, player1_game_name, player1_tag_line)
        .await?
        .expect("Player 1 Riot ID not found.");
    let puuid1 = &account1.puuid;
    println!("Player 1 PUUID: {}", puuid1);

    println!(
        "Fetching PUUID for {}#{}",
        player2_game_name, player2_tag_line
    );
    let account2 = riot_api
        .account_v1()
        .get_by_riot_id(player2_regional_route, player2_game_name, player2_tag_line)
        .await?
        .expect("Player 2 Riot ID not found.");
    let puuid2 = &account2.puuid;
    println!("Player 2 PUUID: {}", puuid2);

    // Ensure they are on the same regional route for match history lookup consistency
    if player1_regional_route != player2_regional_route {
        eprintln!("Warning: Players are on different regional routes. Match history search may be inconsistent or fail.");
        // Depending on your logic, you might want to exit here or handle this differently.
    }


    // 2. Get a list of recent match IDs for Player 1
    // We'll limit to the last 100 matches to stay within typical API caps and avoid long runtimes.
    // The `count` parameter limits the number of matches returned (max 100 per call).
    // The `start_time` parameter is useful for limiting the search to recent games
    // (matches list started storing timestamps on June 16, 2021).
    let one_month_ago = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(30 * 24 * 60 * 60)) // Approx 30 days
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64);

    println!(
        "Fetching match IDs for Player 1 (last 100 matches, roughly last 30 days if available)..."
    );
    let match_ids = riot_api
        .match_v5()
        .get_match_ids_by_puuid(
            player1_regional_route,
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
        println!("No recent matches found for {}#{}", player1_game_name, player1_tag_line);
        return Ok(());
    }

    println!("Found {} recent matches for Player 1.", match_ids.len());

    let mut found_together = false;
    let mut checked_matches = 0;

    // 3. For each match ID, retrieve the full match details
    for match_id in match_ids {
        checked_matches += 1;
        println!("Checking match {} ({} of {})...", match_id, checked_matches, 100);

        let match_data_option = riot_api
            .match_v5()
            .get_match(player1_regional_route, &match_id) // Use the same regional route as match_ids
            .await?; // Use ? to propagate network/API errors, but allows Option to be handled

        if let Some(match_data) = match_data_option {
            // 4. Check if the second player's PUUID is among the participants
            let info = match_data.info;
            let participants_puuids: Vec<&str> =
                info.participants.iter().map(|p| p.puuid.as_str()).collect();

            if participants_puuids.contains(&puuid2.as_str()) {
                    // Initialize a Vec<String> to hold each line
            let mut lines_of_text: Vec<String> = Vec::new();

            // Line 1: Players and Match ID
            lines_of_text.push(format!(
                "Players {}#{} and {}#{} played together in Match ID: {}",
                player1_game_name, player1_tag_line,
                player2_game_name, player2_tag_line,
                match_id
            ));

            // Line 2: Game Mode and Game Type
            lines_of_text.push(format!(
                "Game Mode: {:?}, Game Type: {:?}",
                info.game_mode, info.game_type
            ));

            // Find participant data for Player 1
            let player1_participant = info.participants.iter()
                .find(|p| p.puuid == *puuid1);

            // Find participant data for Player 2
            let player2_participant = info.participants.iter()
                .find(|p| p.puuid == *puuid2);

            if let (Some(p1_data), Some(p2_data)) = (player1_participant, player2_participant) {
                // Participant Details Header
                lines_of_text.push("--- Participant Details ---".to_string());

                // Player 1 Details
                lines_of_text.push(format!("{}:", player1_game_name));
                lines_of_text.push(format!("  Champion: {}", p1_data.champion_name));
                lines_of_text.push(format!("  Role: {}", p1_data.team_position));
                lines_of_text.push(format!("  KDA: {}/{}/{}", p1_data.kills, p1_data.deaths, p1_data.assists));

                // Player 2 Details
                lines_of_text.push(format!("{}:", player2_game_name));
                lines_of_text.push(format!("  Champion: {}", p2_data.champion_name));
                lines_of_text.push(format!("  Role: {}", p2_data.team_position));
                lines_of_text.push(format!("  KDA: {}/{}/{}", p2_data.kills, p2_data.deaths, p2_data.assists));

                // Match Outcome Header
                lines_of_text.push("--- Match Outcome ---".to_string());
                // Match Outcome Detail
                lines_of_text.push(format!("  Won the game?: {}", if p2_data.win { "YES" } else { "NO" }));

                // Print the lines in a box
                if !lines_of_text.is_empty() {
                    print_in_box(&lines_of_text.iter().map(String::as_str).collect::<Vec<&str>>());
                } else {
                    println!("No detailed information available for this match.");

                println!("\n")
    }
                } else {
                    eprintln!("Error: Could not find participant data for one or both players in match {}.", match_id);
                }


                found_together = true;
                // If you only need to find one game, you can break here
                // break;
            }
            
        } else {
            eprintln!("Warning: Match {} not found or accessible.", match_id);
        }
    }

    if !found_together {
        println!("\n{}#{} and {}#{} do not appear to have played together in the last {} matches checked.",
            player1_game_name, player1_tag_line,
            player2_game_name, player2_tag_line,
            checked_matches
        );
    }

    Ok(())
}

fn print_in_box(lines: &[&str]) {
    // 1. Calculate the maximum line length
    let max_len = lines.iter().map(|s| s.len()).max().unwrap_or(0);

    // 2. Determine box width (2 for padding + 2 for borders)
    let box_width = max_len + 4;

    // 3. Print the top border
    println!("{}", "-".repeat(box_width));

    // 4. Print each line, padded and enclosed
    for line in lines {
        // Calculate padding needed for the current line
        let padding = max_len - line.len();
        println!("| {} {} |", line, " ".repeat(padding));
    }

    // 5. Print the bottom border
    println!("{}", "-".repeat(box_width));
}