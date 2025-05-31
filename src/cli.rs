use clap::Parser;
use crate::riot_id::RiotId;
use riven::consts::RegionalRoute;

/// Represents the user-facing regional routes
#[derive(Debug, Clone)]
pub enum UserFacingRegion {
    BR,
    EUNE,
    EUW,
    JP,
    KR,
    LAN,
    LAS,
    ME,
    NA,
    OCE,
    RU,
    SEA,
    TR,
    TW,
    VN,
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
            UserFacingRegion::ME => RegionalRoute::EUROPE,
            UserFacingRegion::NA => RegionalRoute::AMERICAS,
            UserFacingRegion::OCE => RegionalRoute::AMERICAS,
            UserFacingRegion::RU => RegionalRoute::EUROPE,
            UserFacingRegion::SEA => RegionalRoute::ASIA,
            UserFacingRegion::TR => RegionalRoute::EUROPE,
            UserFacingRegion::TW => RegionalRoute::ASIA,
            UserFacingRegion::VN => RegionalRoute::ASIA,
        }
    }

    /// Converts UserFacingRegion to its lowercase string representation for League of Graphs links.
    pub fn to_log_string(&self) -> &'static str {
        match self {
            UserFacingRegion::BR => "br",
            UserFacingRegion::EUNE => "eune",
            UserFacingRegion::EUW => "euw",
            UserFacingRegion::JP => "jp",
            UserFacingRegion::KR => "kr",
            UserFacingRegion::LAN => "lan",
            UserFacingRegion::LAS => "las",
            UserFacingRegion::ME => "me",
            UserFacingRegion::NA => "na",
            UserFacingRegion::OCE => "oce",
            UserFacingRegion::RU => "ru",
            UserFacingRegion::SEA => "sea",
            UserFacingRegion::TR => "tr",
            UserFacingRegion::TW => "tw",
            UserFacingRegion::VN => "vn",
        }
    }
}


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(
    about = "A CLI tool to check if two Riot IDs played together.",
    long_about = "PlayedTogether helps you quickly determine if two League of Legends or Valorant players\n\
                  have recently played in the same match.\n\n\
                  You can specify Riot IDs directly, set a default 'self' ID, and control output verbosity."
)]
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
        help = "Specify the Regional Route (e.g., EUW, NA, KR).\n\
                Overrides --default-region if both are specified.\n\
                Supported:\n    BR, EUNE, EUW, JP, KR, LAN, LAS, ME, NA, OCE, RU, SEA, TR, TW, VN"
    )]
    pub region: Option<UserFacingRegion>,

    #[clap(
        long,
        value_name = "DEFAULT_REGION",
        help = "Set a default Regional Route to use if --region is not specified.\n\
                Defaults to EUROPE if neither --region nor --default-region are specified.\n\
                Supported:\n    BR, EUNE, EUW, JP, KR, LAN, LAS, ME, NA, OCE, RU, SEA, TR, TW, VN"
    )]
    pub default_region: Option<UserFacingRegion>,

    /// Number of most recent games to check for player1.
    /// Default: 100 (Riven API max)
    #[clap(short, long, value_name = "COUNT")]
    pub number: Option<i32>,

    /// Enable verbose output, showing full match details.
    #[clap(short, long)]
    pub verbose: bool,

    /// Enable silent output, only printing links and a summary.
    #[clap(short, long, conflicts_with = "verbose")]
    pub silent: bool,
}