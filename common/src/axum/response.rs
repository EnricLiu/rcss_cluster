use std::sync::atomic::AtomicU32;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

static ID: AtomicU32 = AtomicU32::new(0);

fn get_id() -> u32 {
    ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    #[serde(skip)]
    status_code: StatusCode,

    id: u32,
    success: bool,
    payload: Value,
    created_at: DateTime<Utc>,
}

impl Response {
    pub fn new(id: u32, success: bool, status_code: StatusCode, payload: Value) -> Self {
        Self {
            status_code,

            id,
            success,
            payload,
            created_at: Utc::now(),
        }
    }
    
    pub fn ok() -> Self {
        Self::new(get_id(), true, StatusCode::OK, Value::Null)
    }

    pub fn success<T: Serialize>(payload: T) -> Self {
        Self::new(
            get_id(),
            true,
            StatusCode::OK,
            serde_json::to_value(payload).expect("Failed to serialize payload")
        )
    }

    fn success_value(payload: Value) -> Self {
        Self::new(get_id(), true, StatusCode::OK, payload)
    }

    pub fn code(status_code: StatusCode) -> Self {
        Self::new(get_id(), status_code == StatusCode::OK, status_code, Value::Null)
    }

    pub fn code_u16(status_code: u16) -> Self {
        Self::code(StatusCode::from_u16(status_code).unwrap())
    }

    pub fn fail<T: Serialize>(status_code: StatusCode, payload: T) -> Self {
        Self::new(
            get_id(),
            false,
            status_code,
            serde_json::to_value(payload).expect("Failed to serialize payload")
        )
    }

    pub fn error(err: &str, desc: &str) -> Self {
        Self::fail(StatusCode::OK, Some(json!({ "error": err, "desc": desc })))
    }

    pub fn with_status(self, status_code: StatusCode) -> Self {
        Self {
            status_code,
            ..self
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn payload(&self) -> &Value {
        &self.payload
    }
    
    pub fn into_payload(self) -> Value {
        self.payload
    }
}

pub struct GenericResponse<P: Serialize + for<'de> Deserialize<'de>> {
    pub id: u32,
    pub success: bool,
    pub payload: P,
    pub created_at: DateTime<Utc>,
}

impl<P: Serialize + for<'de> Deserialize<'de>> GenericResponse<P> {
    pub fn try_from(resp: Response) -> serde_json::Result<Self> {
        let payload = serde_json::from_value(resp.payload)?;
        Ok(Self {
            id: resp.id,
            success: resp.success,
            payload,
            created_at: resp.created_at,
        })
    }
}

impl Response {
    pub fn try_into_generic<P>(self) -> serde_json::Result<GenericResponse<P>>
    where P: Serialize + for<'de> Deserialize<'de>
    {
        GenericResponse::try_from(self)
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> AxumResponse {
        if self.status_code.is_success() {
            return (self.status_code, Json(self)).into_response();
        }
        (self.status_code, self.status_code.to_string()).into_response()
    }
}

impl<T, E> From<Result<T, E>> for Response
where
    T: Serialize,
    E: Into<Response>,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(v) => Response::success_value(serde_json::to_value(v).expect("Failed to serialize payload")),
            Err(e) => e.into(),
        }
    }
}
