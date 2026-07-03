use serde_json::{Value, json};

pub const CYRUS_IMAGE: &str = "Cyrus2D/cyrus2024";
pub const HELIOS_IMAGE: &str = "HELIOS/helios2024";
pub const CYRUS_TEAM: &str = "CYRUS2024";
pub const HELIOS_TEAM: &str = "HELIOS2024";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideOrder {
    CyrusLeft,
    HeliosLeft,
}

impl SideOrder {
    pub fn for_match(global_match_index: u64) -> Self {
        if global_match_index % 2 == 0 {
            SideOrder::CyrusLeft
        } else {
            SideOrder::HeliosLeft
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            SideOrder::CyrusLeft => "cyrus_left",
            SideOrder::HeliosLeft => "helios_left",
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchConfigInput {
    pub global_match_index: u64,
    pub time_up: u16,
}

#[derive(Debug, Clone)]
pub struct GeneratedMatchConfig {
    pub request: Value,
    pub conf: Value,
    pub side_order: SideOrder,
}

pub fn build_allocate_request(input: &MatchConfigInput) -> GeneratedMatchConfig {
    let side_order = SideOrder::for_match(input.global_match_index);

    let (left, right) = match side_order {
        SideOrder::CyrusLeft => (
            team(CYRUS_TEAM, CYRUS_IMAGE),
            team(HELIOS_TEAM, HELIOS_IMAGE),
        ),
        SideOrder::HeliosLeft => (
            team(HELIOS_TEAM, HELIOS_IMAGE),
            team(CYRUS_TEAM, CYRUS_IMAGE),
        ),
    };

    let conf = json!({
        "log": true,
        "referee": { "enable": true },
        "stopping": { "time_up": input.time_up },
        "teams": {
            "left": left,
            "right": right,
        }
    });

    let request = json!({
        "version": 1,
        "mode": "gs",
        "conf": conf,
    });

    GeneratedMatchConfig {
        request,
        conf,
        side_order,
    }
}

fn team(name: &str, image: &str) -> Value {
    let players = (1u8..=11)
        .map(|unum| {
            json!({
                "unum": unum,
                "goalie": unum == 1,
                "policy": {
                    "kind": "bot",
                    "image": image,
                },
            })
        })
        .collect::<Vec<_>>();

    json!({
        "name": name,
        "players": players,
        "coach": {
            "policy": {
                "kind": "bot",
                "image": image,
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(global_match_index: u64) -> MatchConfigInput {
        MatchConfigInput {
            global_match_index,
            time_up: 6000,
        }
    }

    #[test]
    fn generates_full_11v11_cyrus_left_match() {
        let generated = build_allocate_request(&input(0));
        assert_eq!(generated.side_order, SideOrder::CyrusLeft);

        let conf = generated.conf;
        assert_eq!(conf["teams"]["left"]["name"], CYRUS_TEAM);
        assert_eq!(conf["teams"]["right"]["name"], HELIOS_TEAM);
        assert_eq!(
            conf["teams"]["left"]["players"].as_array().unwrap().len(),
            11
        );
        assert_eq!(
            conf["teams"]["right"]["players"].as_array().unwrap().len(),
            11
        );
        assert_eq!(conf["teams"]["left"]["players"][0]["goalie"], true);
        assert_eq!(conf["teams"]["left"]["players"][1]["goalie"], false);
        assert_eq!(
            conf["teams"]["left"]["coach"]["policy"]["image"],
            CYRUS_IMAGE
        );
        assert_eq!(
            conf["teams"]["right"]["coach"]["policy"]["image"],
            HELIOS_IMAGE
        );
        assert!(conf["teams"]["left"].get("trainer").is_none());
        assert!(conf["teams"]["right"].get("trainer").is_none());
    }

    #[test]
    fn swaps_sides_by_match_index() {
        let generated = build_allocate_request(&input(1));
        assert_eq!(generated.side_order, SideOrder::HeliosLeft);
        assert_eq!(generated.conf["teams"]["left"]["name"], HELIOS_TEAM);
        assert_eq!(generated.conf["teams"]["right"]["name"], CYRUS_TEAM);
    }

    #[test]
    fn does_not_override_gameserver_log_path() {
        let generated = build_allocate_request(&input(2));
        assert!(generated.conf.get("env").is_none());
    }
}
