use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    response::IntoResponse,
    routing::{get, put},
};
use tracing_finetune::FinetuneFilter;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(finetune))
        .init();

    let callsite_routes = Router::new()
        .route("/", get(handlers::callsites))
        .route("/{callsite_id}/enable", put(handlers::enable_callsite))
        .route("/{callsite_id}/disable", put(handlers::disable_callsite))
        .route(
            "/{callsite_id}/clear",
            put(handlers::clear_callsite_override),
        )
        .with_state(Arc::new(finetune_handle));

    let app = Router::new()
        .route(
            "/",
            get(|| async {
                {
                    let _span_guard =
                        tracing::info_span!("work-span", work_type = "busy").entered();
                    tracing::debug!("Do some work");
                }

                tracing::info!("Hello, World!");

                "Hello, World!\n"
            }),
        )
        .fallback(default_fallback)
        .nest("/api/callsites", callsite_routes);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3007").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn default_fallback(request: Request) -> impl IntoResponse {
    tracing::warn!(uri = %request.uri(), "Not Found");

    "Not Found\n"
}

pub mod handlers {
    use std::sync::Arc;

    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::{IntoResponse, Json, Response},
    };
    use serde::{
        Serialize, Serializer,
        ser::{SerializeSeq, SerializeStruct},
    };
    use tracing_finetune::{FinetuneHandle, OverrideStatus};

    #[derive(Debug)]
    pub struct CallsiteResponse {
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

    #[derive(Debug)]
    pub struct CallsiteWrapper {
        callsite: tracing_finetune::Callsite,
    }

    impl Serialize for CallsiteWrapper {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut state = serializer.serialize_struct("Callsite", 4)?;
            state.serialize_field("id", &self.callsite.id().as_u64())?;
            state.serialize_field(
                "interest",
                &InterestWrapper {
                    interest: &self.callsite.interest(),
                },
            )?;
            state.serialize_field(
                "metadata",
                &MetadataWrapper {
                    meta: self.callsite.metadata(),
                },
            )?;
            state.serialize_field(
                "status",
                &OverrideStatusWrapper {
                    override_status: self.callsite.status(),
                },
            )?;

            state.end()
        }
    }

    #[derive(Debug)]
    pub struct InterestWrapper<'a> {
        interest: &'a tracing::subscriber::Interest,
    }

    impl<'a> Serialize for InterestWrapper<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if self.interest.is_never() {
                serializer.serialize_unit_variant("Interest", 0, "Never")
            } else if self.interest.is_sometimes() {
                serializer.serialize_unit_variant("Interest", 1, "Sometimes")
            } else {
                // is_always
                serializer.serialize_unit_variant("Interest", 2, "Always")
            }
        }
    }

    #[derive(Debug)]
    pub struct MetadataWrapper {
        meta: &'static tracing::Metadata<'static>,
    }

    impl Serialize for MetadataWrapper {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut state = serializer.serialize_struct("Metadata", 8)?;

            state.serialize_field("name", self.meta.name())?;
            state.serialize_field("target", self.meta.target())?;
            state.serialize_field(
                "level",
                &LevelWrapper {
                    level: self.meta.level(),
                },
            )?;
            state.serialize_field("module_path", &self.meta.module_path())?;
            state.serialize_field("file", &self.meta.file())?;
            state.serialize_field("line", &self.meta.line())?;
            state.serialize_field(
                "field_names",
                &FieldSetWrapper {
                    field_set: self.meta.fields(),
                },
            )?;
            state.serialize_field("kind", &KindWrapper { meta: self.meta })?;

            state.end()
        }
    }

    #[derive(Debug)]
    pub struct LevelWrapper<'a> {
        level: &'a tracing::Level,
    }

    impl<'a> Serialize for LevelWrapper<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match *self.level {
                tracing::Level::ERROR => serializer.serialize_unit_variant("Level", 4, "Error"),
                tracing::Level::WARN => serializer.serialize_unit_variant("Level", 3, "Warn"),
                tracing::Level::INFO => serializer.serialize_unit_variant("Level", 2, "Info"),
                tracing::Level::DEBUG => serializer.serialize_unit_variant("Level", 1, "Debug"),
                // This should never happen (tracing uses an enum underneath), but we'll
                // default to `Trace` just in case.
                _ => serializer.serialize_unit_variant("Level", 0, "Trace"),
            }
        }
    }

    #[derive(Debug)]
    pub struct FieldSetWrapper<'a> {
        field_set: &'a tracing::field::FieldSet,
    }

    impl<'a> Serialize for FieldSetWrapper<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut state = serializer.serialize_seq(Some(self.field_set.len()))?;

            for field in self.field_set.iter() {
                state.serialize_element(field.name())?;
            }

            state.end()
        }
    }

    #[derive(Debug)]
    pub struct KindWrapper {
        meta: &'static tracing::Metadata<'static>,
    }

    impl Serialize for KindWrapper {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            if self.meta.is_event() {
                serializer.serialize_unit_variant("Kind", 0, "Event")
            } else if self.meta.is_span() {
                serializer.serialize_unit_variant("Kind", 1, "Span")
            } else {
                // this comes from `is_enabled!`
                serializer.serialize_unit_variant("Kind", 2, "Unknown")
            }
        }
    }

    #[derive(Debug)]
    struct OverrideStatusWrapper {
        override_status: OverrideStatus,
    }

    impl Serialize for OverrideStatusWrapper {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self.override_status {
                OverrideStatus::NoOverride => {
                    serializer.serialize_unit_variant("OverrideStatus", 0, "NoOverride")
                }
                OverrideStatus::ExplicitlyEnabled => {
                    serializer.serialize_unit_variant("OverrideStatus", 1, "ExplicitlyEnabled")
                }
                OverrideStatus::ExplicitlyDisabled => {
                    serializer.serialize_unit_variant("OverrideStatus", 2, "ExplicityDisabled")
                }
            }
        }
    }

    pub struct NotFound {
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

    pub async fn callsites(
        State(finetune_handle): State<Arc<FinetuneHandle>>,
    ) -> Json<CallsiteResponse> {
        let callsites = finetune_handle
            .callsites()
            .into_iter()
            .map(|c| CallsiteWrapper { callsite: c })
            .collect::<Vec<_>>();

        Json(CallsiteResponse { callsites })
    }

    pub async fn enable_callsite(
        State(finetune_handle): State<Arc<FinetuneHandle>>,
        Path(callsite_id): Path<u64>,
    ) -> Result<Json<CallsiteWrapper>, NotFound> {
        match finetune_handle.enable_callsite(callsite_id.into()) {
            Some(callsite) => Ok(Json(CallsiteWrapper { callsite })),
            None => Err(NotFound { callsite_id }),
        }
    }

    pub async fn disable_callsite(
        State(finetune_handle): State<Arc<FinetuneHandle>>,
        Path(callsite_id): Path<u64>,
    ) -> Result<Json<CallsiteWrapper>, NotFound> {
        match finetune_handle.disable_callsite(callsite_id.into()) {
            Some(callsite) => Ok(Json(CallsiteWrapper { callsite })),
            None => Err(NotFound { callsite_id }),
        }
    }

    pub async fn clear_callsite_override(
        State(finetune_handle): State<Arc<FinetuneHandle>>,
        Path(callsite_id): Path<u64>,
    ) -> Result<Json<CallsiteWrapper>, NotFound> {
        match finetune_handle.clear_callsite_override(callsite_id.into()) {
            Some(callsite) => Ok(Json(CallsiteWrapper { callsite })),
            None => Err(NotFound { callsite_id }),
        }
    }
}
