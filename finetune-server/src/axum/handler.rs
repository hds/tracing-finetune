use std::sync::Arc;

use ::axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Serialize, Serializer, ser::SerializeStruct};

use tracing_finetune::FinetuneHandle;

use crate::ser::CallsiteWrapper;

#[derive(Debug)]
pub(crate) struct CallsiteResponse {
    callsites: Vec<CallsiteWrapper>,
}

impl Serialize for CallsiteResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CallsitesResponse", 1)?;

        state.serialize_field("callsites", &self.callsites)?;

        state.end()
    }
}

pub(crate) struct NotFound {
    callsite_id: u64,
}

struct NotFoundBody {
    callsite_id: u64,
}

impl IntoResponse for NotFound {
    fn into_response(self) -> Response {
        (
            StatusCode::NOT_FOUND,
            Json(NotFoundBody {
                callsite_id: self.callsite_id,
            }),
        )
            .into_response()
    }
}

impl Serialize for NotFoundBody {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NotFound", 2)?;

        state.serialize_field("error", "Callsite not found for Id")?;
        state.serialize_field("callsite_id", &self.callsite_id)?;

        state.end()
    }
}

pub(crate) async fn callsites(
    State(finetune_handle): State<Arc<FinetuneHandle>>,
) -> Json<CallsiteResponse> {
    let callsites = finetune_handle
        .callsites()
        .into_iter()
        .map(|c| CallsiteWrapper::new(c))
        .collect::<Vec<_>>();

    Json(CallsiteResponse { callsites })
}

pub(crate) async fn enable_callsite(
    State(finetune_handle): State<Arc<FinetuneHandle>>,
    Path(callsite_id): Path<u64>,
) -> Result<Json<CallsiteWrapper>, NotFound> {
    match finetune_handle.enable_callsite(callsite_id.into()) {
        Some(callsite) => Ok(Json(CallsiteWrapper::new(callsite))),
        None => Err(NotFound { callsite_id }),
    }
}

pub(crate) async fn disable_callsite(
    State(finetune_handle): State<Arc<FinetuneHandle>>,
    Path(callsite_id): Path<u64>,
) -> Result<Json<CallsiteWrapper>, NotFound> {
    match finetune_handle.disable_callsite(callsite_id.into()) {
        Some(callsite) => Ok(Json(CallsiteWrapper::new(callsite))),
        None => Err(NotFound { callsite_id }),
    }
}

pub(crate) async fn clear_callsite_override(
    State(finetune_handle): State<Arc<FinetuneHandle>>,
    Path(callsite_id): Path<u64>,
) -> Result<Json<CallsiteWrapper>, NotFound> {
    match finetune_handle.clear_callsite_override(callsite_id.into()) {
        Some(callsite) => Ok(Json(CallsiteWrapper::new(callsite))),
        None => Err(NotFound { callsite_id }),
    }
}
