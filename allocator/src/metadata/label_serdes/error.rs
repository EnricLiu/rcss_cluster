use common::errors::BuilderError;

#[derive(thiserror::Error, Debug)]
pub enum LabelSerializeError {
    #[error("label value too long ({len} > {K8S_LABEL_MAX_LEN}): {encoded}")]
    ValueTooLong { encoded: String, len: usize },
    #[error("label value contains invalid character '{ch}': {encoded}, outside `[A-Za-z0-9_.-]`")]
    InvalidCharacter { encoded: String, ch: char },
}

impl From<LabelSerializeError> for BuilderError {
    fn from(e: LabelSerializeError) -> Self {
        BuilderError::InvalidField {
            field: "label",
            message: e.to_string(),
        }
    }
}

pub type LabelSerializeResult<T> = Result<T, LabelSerializeError>;

#[derive(thiserror::Error, Debug)]
pub enum LabelDeserializeError {
    #[error("invalid label format: {reason} (raw: {raw})")]
    InvalidFormat { raw: String, reason: &'static str },
    #[error("unknown player kind '{kind}' in label (raw: {raw}), expect 'h' or 's'")]
    UnknownKind { raw: String, kind: String },
    #[error("failed to parse field '{field}': {detail} (raw: {raw})")]
    InvalidField { raw: String, field: &'static str, detail: String },
}

impl From<LabelDeserializeError> for BuilderError {
    fn from(e: LabelDeserializeError) -> Self {
        BuilderError::InvalidField {
            field: "label",
            message: e.to_string(),
        }
    }
}

pub type LabelDeserializeResult<T> = Result<T, LabelDeserializeError>;
