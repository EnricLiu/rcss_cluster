use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
#[repr(u8)]
pub enum ClientKind {
    Player = 0,
    OlCoach = 1,
    Trainer = 2,
}

impl Default for ClientKind { 
    fn default() -> Self {
        ClientKind::Player
    }
}

impl TryFrom<u8> for ClientKind {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ClientKind::Player),
            1 => Ok(ClientKind::OlCoach),
            2 => Ok(ClientKind::Trainer),
            _ => Err(()),
        }
    }
}

impl Serialize for ClientKind {
    fn serialize<S: Serializer>(
        &self, serializer: S
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for ClientKind {
    fn deserialize<D: Deserializer<'de>>(des: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(des)?;
        let kind = ClientKind::try_from(value)
            .map_err(|_| serde::de::Error::custom(format!("Invalid client kind: {}", value)))?;

        Ok(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClientKind::*;

    #[test]
    fn test_serialize() {
        let kind = [Player, Trainer, OlCoach];
        for kind in kind {
            let serialized = serde_json::to_string(&kind).unwrap();
            assert_eq!(serialized, format!("{}", kind as u8));
        }
    }

    #[test]
    fn test_deserialize() {
        let kind = [Trainer, Player, OlCoach];
        let serialized = kind.iter().map(|kind| *kind as u8);
        for (serialized, kind) in serialized.zip(kind) {
            let des: ClientKind = serde_json::from_str(&serialized.to_string()).unwrap();
            assert_eq!(des, kind);
        }
    }

    #[test]
    fn test_ser_des() {
        let raw = vec![Player, Trainer, OlCoach];
        let ser = serde_json::to_string(&raw).unwrap();
        let des: Vec<ClientKind> = serde_json::from_str(&ser).unwrap();
        assert_eq!(raw, des, "Serialization and deserialization failed");
    }
}