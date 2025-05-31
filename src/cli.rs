use clap::Parser;
use crate::riot_id::RiotId;
use riven::consts::RegionalRoute; // Keep this import for conversion

/// Represents the user-facing regional routes
#[derive(Debug, Clone)]
pub enum UserFacingRegion {
    BR,   // Brazil -> AMERICAS
    EUNE, // Europe Nordic & East -> EUROPE
    EUW,  // Europe West -> EUROPE
    JP,   // Japan -> ASIA
    KR,   // Korea -> ASIA
    LAN,  // Latin America North -> AMERICAS
    LAS,  // Latin America South -> AMERICAS
    ME,   // Middle East -> EUROPE (Note: RIOT API may have specific ME RegionalRoute in future)
    NA,   // North America -> AMERICAS
    OCE,  // Oceania -> AMERICAS (Note: RIOT API may have specific OCE RegionalRoute in future)
    RU,   // Russia -> EUROPE
    SEA,  // Southeast Asia -> ASIA
    TR,   // Turkey -> EUROPE (This is PlatformRoute::TR, maps to RegionalRoute::EUROPE)
    TW,   // Taiwan -> ASIA
    VN,   // Vietnam -> ASIA
}

impl std::str::FromStr for UserFacingRegion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BR" => Ok(UserFacingRegion::BR),
            "EUNE" => Ok(UserFacingRegion::EUNE),
            "EUW" => Ok(UserFacingRegion::EUW),
            "JP" => Ok(UserFacingRegion::JP),
            "KR" => Ok(UserFacingRegion::KR),
            "LAN" => Ok(UserFacingRegion::LAN),
            "LAS" => Ok(UserFacingRegion::LAS),
            "ME" => Ok(UserFacingRegion::ME),
            "NA" => Ok(UserFacingRegion::NA),
            "OCE" => Ok(UserFacingRegion::OCE),
            "RU" => Ok(UserFacingRegion::RU),
            "SEA" => Ok(UserFacingRegion::SEA),
            "TR" => Ok(UserFacingRegion::TR),
            "TW" => Ok(UserFacingRegion::TW),
            "VN" => Ok(UserFacingRegion::VN),
            _ => Err(format!("Invalid region: {}. Supported regions are BR, EUNE, EUW, JP, KR, LAN, LAS, ME, NA, OCE, RU, SEA, TR, TW, VN", s)),
        }
    }
}

impl UserFacingRegion {
    /// Converts a UserFacingRegion to the corresponding riven::consts::RegionalRoute.
    pub fn to_regional_route(&self) -> RegionalRoute {
        match self {
            UserFacingRegion::BR => RegionalRoute::AMERICAS,
            UserFacingRegion::EUNE => RegionalRoute::EUROPE,
            UserFacingRegion::EUW => RegionalRoute::EUROPE,
            UserFacingRegion::JP => RegionalRoute::ASIA,
            UserFacingRegion::KR => RegionalRoute::ASIA,
            UserFacingRegion::LAN => RegionalRoute::AMERICAS,
            UserFacingRegion::LAS => RegionalRoute::AMERICAS,
            UserFacingRegion::ME => RegionalRoute::EUROPE, // RIOT API sometimes maps ME to EUROPE
            UserFacingRegion::NA => RegionalRoute::AMERICAS,
            UserFacingRegion::OCE => RegionalRoute::AMERICAS, // RIOT API sometimes maps OCE to AMERICAS
            UserFacingRegion::RU => RegionalRoute::EUROPE,
            UserFacingRegion::SEA => RegionalRoute::ASIA,
            UserFacingRegion::TR => RegionalRoute::EUROPE,
            UserFacingRegion::TW => RegionalRoute::ASIA,
            UserFacingRegion::VN => RegionalRoute::ASIA,
        }
    }
}


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets the "self" Riot ID for subsequent calls (e.g., MainingYourMom#4444)
    #[clap(long = "self", value_name = "RIOT_ID")]
    pub set_self: Option<RiotId>,

    /// Check if two Riot IDs played together.
    /// player1: The Riot ID whose match history will be checked.
    /// player2: The Riot ID to search for in player1's match history.
    #[clap(value_parser, num_args = 0..=2)]
    pub riot_ids: Vec<RiotId>,

    #[clap(
        long, 
        value_name = "REGION_OVERRIDE",
        help = "Specify the Regional Route to use for the query. Supported values: \nBR, EUNE, EUW, JP, KR, LAN, LAS, ME, NA, OCE, RU, SEA, TR, TW, VN",
    )]
    pub region: Option<UserFacingRegion>, // <-- Change type to UserFacingRegion

    #[clap(
        long, 
        value_name = "DEFAULT_REGION",
        help = "Set a default Regional Route to use if --region is not specified. Defaults to EUROPE if neither --region nor --default-region are specified. Supported values: \nBR, EUNE, EUW, JP, KR, LAN, LAS, ME, NA, OCE, RU, SEA, TR, TW, VN"
    )]
    pub default_region: Option<UserFacingRegion>, // <-- Change type to UserFacingRegion

    /// Enable verbose output, showing full match details.
    #[clap(short, long)]
    pub verbose: bool,

    /// Enable silent output, only printing links and a summary.
    #[clap(short, long, conflicts_with = "verbose")]
    pub silent: bool,
}