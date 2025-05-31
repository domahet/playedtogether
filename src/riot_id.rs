/// Represents a Riot ID (GameName#TagLine)
#[derive(Debug, Clone)]
pub struct RiotId {
    pub game_name: String,
    pub tag_line: String,
}

impl std::fmt::Display for RiotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.game_name, self.tag_line)
    }
}

impl std::str::FromStr for RiotId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('#').collect();
        if parts.len() == 2 {
            Ok(RiotId {
                game_name: parts[0].to_string(),
                tag_line: parts[1].to_string(),
            })
        } else {
            Err("Invalid Riot ID format. Expected 'GameName#TagLine'")
        }
    }
}