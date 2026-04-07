use std::net::Ipv4Addr;

use crate::declaration::{PlayerDeclaration, PlayerBaseDeclaration, HostPort, Unum};
use crate::declaration::image::Image;

use super::{
    PlayerLabel,
    LabelDeserialize,
    LabelDeserializeResult,
    LabelDeserializeError,
    FIELD_SEP, IMAGE_SEP, IMAGE_ORIG_SEP,
};

/// Decodes a K8s-safe label value back into a [`PlayerLabel`].
///
/// Expected formats (fields separated by `_`):
///   - Helios: `h_{provider}.{model}_{goalie}{log}`
///   - SSP:    `s_{provider}.{model}_{goalie}{log}_{host}_{port}`
impl LabelDeserialize for PlayerLabel {
    type Raw = (Unum, String);

    fn label_deserialize(raw: &Self::Raw) -> LabelDeserializeResult<Self> {
        let parts: Vec<&str> = raw.splitn(5, FIELD_SEP).collect();

        if parts.len() < 3 {
            return Err(LabelDeserializeError::InvalidFormat {
                raw: raw.clone(),
                reason: "expected at least 3 fields: kind, image, flags",
            });
        }

        let kind = parts[0];
        let image_encoded = parts[1];
        let flags = parts[2];

        // ── Parse image ─────────────────────────────────────────────────
        let image_raw = image_encoded.replacen(
            IMAGE_SEP,
            &IMAGE_ORIG_SEP.to_string(),
            1, // only replace the first occurrence (provider.model → provider/model)
        );
        let image = Image::try_from(image_raw.clone()).map_err(|e| {
            LabelDeserializeError::InvalidField {
                raw: raw.clone(),
                field: "image",
                detail: format!("{e}"),
            }
        })?;

        // ── Parse flags ─────────────────────────────────────────────────
        if flags.len() != 2 {
            return Err(LabelDeserializeError::InvalidFormat {
                raw: raw.clone(),
                reason: "flags field must be exactly 2 characters (goalie + log)",
            });
        }
        let goalie = parse_bool_flag(flags.as_bytes()[0], raw, "goalie")?;
        let log = parse_bool_flag(flags.as_bytes()[1], raw, "log")?;

        // ── Build the player declaration by kind ────────────────────────
        // unum is stored in the label *key* (p.l.{unum}), not the value,
        // so we set a placeholder here; the caller will override it.
        let unum = Unum::default();

        match kind {
            "h" => {
                if parts.len() != 3 {
                    return Err(LabelDeserializeError::InvalidFormat {
                        raw: raw.clone(),
                        reason: "Helios label must have exactly 3 fields",
                    });
                }
                let base = PlayerBaseDeclaration { unum, image, goalie, log };
                Ok(PlayerLabel {
                    player: PlayerDeclaration::Helios { base },
                })
            }
            "s" => {
                // remaining after splitn(5, _) at index 3 may be "host_port"
                // We need host and port as two more fields.
                if parts.len() < 4 {
                    return Err(LabelDeserializeError::InvalidFormat {
                        raw: raw.clone(),
                        reason: "SSP label must have 5 fields: kind, image, flags, host, port",
                    });
                }
                // parts[3] is everything after the third `_`, e.g. "127.0.0.1_6657"
                let rest = parts[3];
                // Split rest by the *last* underscore to separate host from port
                let last_sep = rest.rfind(FIELD_SEP).ok_or_else(|| {
                    LabelDeserializeError::InvalidFormat {
                        raw: raw.clone(),
                        reason: "SSP label must have host and port separated by '_'",
                    }
                })?;
                let host_str = &rest[..last_sep];
                let port_str = &rest[last_sep + 1..];

                let host: Ipv4Addr = host_str.parse().map_err(|e| {
                    LabelDeserializeError::InvalidField {
                        raw: raw.clone(),
                        field: "grpc.host",
                        detail: format!("{e}"),
                    }
                })?;
                let port: u16 = port_str.parse().map_err(|e| {
                    LabelDeserializeError::InvalidField {
                        raw: raw.clone(),
                        field: "grpc.port",
                        detail: format!("{e}"),
                    }
                })?;

                let base = PlayerBaseDeclaration { unum, image, goalie, log };
                Ok(PlayerLabel {
                    player: PlayerDeclaration::Ssp {
                        base,
                        grpc: HostPort { host, port },
                    },
                })
            }
            other => Err(LabelDeserializeError::UnknownKind {
                raw: raw.clone(),
                kind: other.to_string(),
            }),
        }
    }
}

fn parse_bool_flag(byte: u8, raw: &str, field: &'static str) -> LabelDeserializeResult<bool> {
    match byte {
        b'1' => Ok(true),
        b'0' => Ok(false),
        _ => Err(LabelDeserializeError::InvalidField {
            raw: raw.to_string(),
            field,
            detail: format!("expected '0' or '1', got '{}'", byte as char),
        }),
    }
}
