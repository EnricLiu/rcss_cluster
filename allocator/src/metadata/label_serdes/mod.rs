mod ser_impl;
mod des_impl;
mod error;

use super::labels::PlayerLabel;

pub use error::{
    LabelDeserializeError, LabelDeserializeResult,
    LabelSerializeError, LabelSerializeResult,
};

pub use ser_impl::validate_label_value;

/// Field separator used between encoded segments.
/// Chosen because K8s label values allow `_` and it does not appear in
/// image provider/model names (enforced by [`Image::try_from`]: only `[A-Za-z0-9-]`).
pub const FIELD_SEP: char = '_';

/// Separator that replaces `/` inside image strings so the value stays
/// K8s-label-safe.  `provider/model` → `provider.model`
///
/// `.` is forbidden in image segments (enforced by [`Image::try_from`]),
/// so the first `.` in the encoded image always corresponds to the original `/`.
pub const IMAGE_SEP: char = '.';

/// Original separator in image strings.
pub const IMAGE_ORIG_SEP: char = '/';

/// Maximum length of a K8s label value.
pub const K8S_LABEL_MAX_LEN: usize = 63;

pub trait LabelSerialize {
    type Raw;
    fn label_serialize(&self) -> LabelSerializeResult<Self::Raw>;
}

pub trait LabelDeserialize: Sized {
    type Raw;
    fn label_deserialize(raw: &Self::Raw) -> LabelDeserializeResult<Self>;
}

pub fn ser<L: LabelSerialize>(l: &L) -> LabelSerializeResult<L::Raw> {
    l.label_serialize()
}

pub fn des<L: LabelDeserialize>(raw: &L::Raw) -> LabelDeserializeResult<L> {
    L::label_deserialize(raw)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_too_long() {
        let long = "a".repeat(K8S_LABEL_MAX_LEN+1);
        assert!(validate_label_value(&long).is_err());
    }

    #[test]
    fn validate_rejects_invalid_chars() {
        assert!(validate_label_value("hello{world}").is_err());
        assert!(validate_label_value("a/b").is_err());
        assert!(validate_label_value(r#"{"k":"v"}"#).is_err());
    }

    #[test]
    fn validate_accepts_good_values() {
        assert!(validate_label_value("").is_ok());
        assert!(validate_label_value("h_HELIOS.helios-base_00").is_ok());
        assert!(validate_label_value("s_Cyrus2D.SoccerSimulationProxy_10_127.0.0.1_6657").is_ok());
    }

    #[test]
    fn roundtrip_helios() {
        use crate::declaration::{PlayerBaseDeclaration, PlayerDeclaration, Unum};
        use crate::declaration::image::Image;

        let unum = Unum::try_from(3u8).unwrap();
        let base = PlayerBaseDeclaration {
            unum,
            image: Image::try_from("HELIOS/helios-base").unwrap(),
            goalie: false,
            log: false,
        };
        let label = PlayerLabel {
            player: PlayerDeclaration::Helios { base },
        };

        let encoded = ser(&label).expect("serialize");
        assert!(validate_label_value(&encoded).is_ok());

        let decoded: PlayerLabel = des(&(unum, encoded.clone())).expect("deserialize");
        let re_encoded = ser(&decoded).expect("re-serialize");
        assert_eq!(encoded, re_encoded);
    }

    #[test]
    fn roundtrip_ssp() {
        use std::net::Ipv4Addr;
        use crate::declaration::{HostPort, PlayerBaseDeclaration, PlayerDeclaration, Unum};
        use crate::declaration::image::Image;

        let unum = Unum::try_from(1u8).unwrap();
        let base = PlayerBaseDeclaration {
            unum,
            image: Image::try_from("Cyrus2D/SoccerSimulationProxy").unwrap(),
            goalie: true,
            log: false,
        };
        let label = PlayerLabel {
            player: PlayerDeclaration::Ssp {
                base,
                grpc: HostPort {
                    host: Ipv4Addr::new(127, 0, 0, 1).into(),
                    port: 6657,
                },
            },
        };

        let encoded = ser(&label).expect("serialize");
        assert!(validate_label_value(&encoded).is_ok());
        assert!(encoded.len() <= K8S_LABEL_MAX_LEN);

        let decoded: PlayerLabel = des(&(unum, encoded.clone())).expect("deserialize");
        let re_encoded = ser(&decoded).expect("re-serialize");
        assert_eq!(encoded, re_encoded);
    }
}
