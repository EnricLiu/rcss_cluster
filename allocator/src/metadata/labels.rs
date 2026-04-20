use std::sync::OnceLock;
use std::collections::{BTreeMap, HashMap};

use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use arcstr::ArcStr;

use common::errors::{BuilderError, BuilderResult};
use common::types::Side;

use crate::declaration::{PlayerDeclaration, Unum};
use super::label_serdes;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerLabel {
    #[serde(flatten)]
    pub player: PlayerDeclaration,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Labels {
    pub left: HashMap<Unum, PlayerLabel>, // p.l.1, p.l.2 = str
    pub right: HashMap<Unum, PlayerLabel>, // p.r.1, p.r.2 = str

    #[serde(skip)]
    pub left_num: u8, // p.l.n = u8
    #[serde(skip)]
    pub right_num: u8, // p.r.n = u8
    
    #[serde(skip)]
    buf_map: OnceLock<HashMap<String, String>>,
    #[serde(skip)]
    buf_ordered_map: OnceLock<BTreeMap<String, String>>,
    #[serde(skip)]
    buf_hash: OnceLock<ArcStr>,
}

const LABELS_HASH_LEN: usize = 48;
const LABELS_HASH_CHAR_SET: &[char; 32] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
    'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
    'u', 'v',
];

impl Labels {
    pub fn new(left: HashMap<Unum, PlayerLabel>, right: HashMap<Unum, PlayerLabel>) -> Self {
        let left_num = left.len() as u8;
        let right_num = right.len() as u8;
        let left = left.into_iter().collect();
        let right = right.into_iter().collect();

        Self {
            left, right,
            left_num, right_num,
            buf_map: OnceLock::new(),
            buf_hash: OnceLock::new(),
            buf_ordered_map: OnceLock::new(),
        }
    }
    
    pub fn validate(&self) -> BuilderResult<()> {
        self.try_as_map()?;
        Ok(())
    }
    
    pub fn from_map(map: HashMap<String, String>) -> BuilderResult<Self> {
        let mut left = HashMap::new();
        let mut right = HashMap::new();

        let keys: Vec<_> = map.keys()
            .map(|key| (key, key.split(".").collect::<Vec<_>>())).collect();

        let player_keys = keys.iter().filter_map(|(full, parts)| {
            if parts.len() != 3 { return None }
            if parts[0] != "p" { return None }

            let side = match parts[1] {
                "l" => Side::LEFT,
                "r" => Side::RIGHT,
                _ => return None,
            };

            let unum = parts[2].parse::<u8>().ok()?;
            Some((full, side, unum))
        });

        for (key, side, unum) in player_keys {
            let value = map.get(*key).unwrap();
            let unum = Unum::try_from(unum)?;

            let player_label = label_serdes::des::<PlayerLabel>(&(unum, value.clone()))
                .map_err(|e| -> BuilderError { e.into() })?;


            match side {
                Side::LEFT => left.insert(unum, player_label),
                Side::RIGHT => right.insert(unum, player_label),
                _ => unreachable!(),
            };
        }

        Ok(Self::new(left, right))
    }

    pub fn try_into_map(self) -> BuilderResult<HashMap<String, String>> {
        let mut map = HashMap::new();

        map.insert("p.l.n".to_string(), self.left_num.to_string());
        map.insert("p.r.n".to_string(), self.right_num.to_string());

        for (unum, label) in self.left {
            let encoded = label_serdes::ser(&label)?;
            map.insert(format!("p.l.{}", unum), encoded);
        }
        for (unum, label) in self.right {
            let encoded = label_serdes::ser(&label)?;
            map.insert(format!("p.r.{}", unum), encoded);
        }
        Ok(map)
    }
    
    pub fn try_as_map(&self) -> BuilderResult<&HashMap<String, String>> {
        if let Some(ret) = self.buf_map.get() {
            return Ok(ret);
        }

        let map = self.clone().try_into_map()?;
        Ok(self.buf_map.get_or_init(|| map))
    }

    pub fn try_as_ordered_map(&self) -> BuilderResult<&BTreeMap<String, String>> {
        if let Some(ret) = self.buf_ordered_map.get() {
            return Ok(ret);
        }

        let ordered = self.try_as_map()?
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(self.buf_ordered_map.get_or_init(|| ordered))
    }
    
    pub fn as_hash(&self) -> BuilderResult<&ArcStr> {
        if let Some(ret) = self.buf_hash.get() {
            return Ok(ret);
        }

        let ordered = self.try_as_ordered_map()?;
        let hash = Self::gen_hash(ordered);
        
        Ok(self.buf_hash.get_or_init(|| ArcStr::from(hash)))
    }

    fn gen_hash(map: &BTreeMap<String, String>) -> String {
        let encoded = serde_json::to_vec(map)
            .expect("serializing ordered labels for hashing should not fail");
        let digest = Sha256::digest(encoded);

        //  5 * u8 =  8 * u5,
        // 48 * u5 = 30 * u8, encode 30 bytes into 48 chars
        assert_eq!(LABELS_HASH_LEN, 48);
        assert_eq!(LABELS_HASH_CHAR_SET.len(), 32);

        let mut ret = String::with_capacity(LABELS_HASH_LEN);
        for chunk in digest.chunks(5).take(LABELS_HASH_LEN / 8) {
            assert_eq!(chunk.len(), 5);

            let mut value = 0u64;
            for &byte in chunk {
                value = (value << 8) | byte as u64;
            }

            for i in (0..8).rev() {
                let index = (value >> (i * 5)) & 0b11111;
                ret.push(LABELS_HASH_CHAR_SET[index as usize]);
            }
        }

        ret
    }
}

impl Eq for Labels {}
impl PartialEq for Labels {
    fn eq(&self, other: &Self) -> bool {
        let self_map = match self.try_as_map() {
            Ok(map) => map,
            Err(_) => return false,
        };
        
        let other_map = match other.try_as_map() {
            Ok(map) => map,
            Err(_) => return false,
        };
        
        self_map == other_map
    }
}

impl TryInto<Labels> for HashMap<String, String> {
    type Error = BuilderError;
    fn try_into(self) -> Result<Labels, Self::Error> {
        Labels::from_map(self)
    }
}
