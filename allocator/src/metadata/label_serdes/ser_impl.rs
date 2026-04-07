use crate::declaration::PlayerDeclaration;

use super::{
    PlayerLabel,
    LabelSerialize,
    LabelSerializeError,
    LabelSerializeResult,
    FIELD_SEP, IMAGE_SEP,
    IMAGE_ORIG_SEP, K8S_LABEL_MAX_LEN,
};

/// Encoding format (all fields separated by `_`):
///
/// | Variant | Format                                          | Example                                               |
/// |---------|-------------------------------------------------|-------------------------------------------------------|
/// | Helios  | `h_{provider}.{model}_{goalie}{log}`            | `h_HELIOS.helios-base_00`                             |
/// | SSP     | `s_{provider}.{model}_{goalie}{log}_{host}_{port}` | `s_Cyrus2D.SoccerSimulationProxy_10_127.0.0.1_6657` |
///
/// - `kind`:  `h` = Helios, `s` = SSP
/// - `image`: original `/` replaced with `.`
/// - `goalie` / `log`: `1` = true, `0` = false
/// - `host`:  IPv4 address (dots are K8s-safe)
/// - `port`:  u16 decimal
impl LabelSerialize for PlayerLabel {
    type Raw = String;

    fn label_serialize(&self) -> LabelSerializeResult<Self::Raw> {
        let encoded = match &self.player {
            PlayerDeclaration::Helios { base } => {
                let image = base.image.raw.replace(IMAGE_ORIG_SEP, &IMAGE_SEP.to_string());
                let flags = format!(
                    "{}{}",
                    if base.goalie { '1' } else { '0' },
                    if base.log { '1' } else { '0' },
                );
                format!("h{FIELD_SEP}{image}{FIELD_SEP}{flags}")
            }
            PlayerDeclaration::Ssp { base, grpc } => {
                let image = base.image.raw.replace(IMAGE_ORIG_SEP, &IMAGE_SEP.to_string());
                let flags = format!(
                    "{}{}",
                    if base.goalie { '1' } else { '0' },
                    if base.log { '1' } else { '0' },
                );
                format!(
                    "s{FIELD_SEP}{image}{FIELD_SEP}{flags}{FIELD_SEP}{}{FIELD_SEP}{}",
                    grpc.host, grpc.port,
                )
            }
        };

        validate_label_value(&encoded)?;
        Ok(encoded)
    }
}

/// Validate that an encoded string is a legal K8s label value:
///   - at most 63 characters
///   - matches `(([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])?`
pub fn validate_label_value(value: &str) -> LabelSerializeResult<()> {
    if value.len() > K8S_LABEL_MAX_LEN {
        return Err(LabelSerializeError::ValueTooLong {
            encoded: value.to_string(),
            len: value.len(),
        });
    }
    if value.is_empty() {
        return Ok(());
    }
    for ch in value.chars() {
        if !matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.') {
            return Err(LabelSerializeError::InvalidCharacter {
                encoded: value.to_string(),
                ch,
            });
        }
    }
    Ok(())
}
