//! Tools to serialize `tracing-finetune` objects for use in web servers.
//!
//! See the documentation on [`serializable_callsite`] for details.
use serde::{
    Serialize, Serializer,
    ser::{SerializeSeq, SerializeStruct},
};
use tracing_finetune::OverrideStatus;

/// Helper to serialize a [`tracing_finetune::Callsite`]. Returns an opaque type that implements
/// the [`serde::Serialize`] trait.
///
/// # Examples
///
/// ```
///# use tracing_subscriber::prelude::*;
///# use tracing_finetune::FinetuneFilter;
///# let layer = tracing_subscriber::fmt::layer();
///# let filter = tracing_subscriber::filter::LevelFilter::WARN;
///# let finetune = FinetuneFilter::new(filter);
///# let finetune_handle = finetune.handle();
///# let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));
///# {
///# let _guard = tracing::subscriber::set_default(subscriber);
/// use finetune_server::ser::serializable_callsite;
///
/// tracing::info!("an event!");
///
/// let callsites = finetune_handle.callsites();
/// let serialized_callsite =
/// serde_json::to_string(&serializable_callsite(callsites[0].clone())).unwrap();
///
/// // Not an empty object
/// assert!(serialized_callsite.len() > 2);
///# }
/// ```
#[must_use]
pub fn serializable_callsite(callsite: tracing_finetune::Callsite) -> impl serde::Serialize {
    CallsiteWrapper::new(callsite)
}

#[derive(Debug)]
pub(crate) struct CallsiteWrapper {
    callsite: tracing_finetune::Callsite,
}

impl CallsiteWrapper {
    pub(crate) fn new(callsite: tracing_finetune::Callsite) -> Self {
        Self { callsite }
    }
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
struct InterestWrapper<'a> {
    interest: &'a tracing::subscriber::Interest,
}

impl Serialize for InterestWrapper<'_> {
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
struct MetadataWrapper {
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
struct LevelWrapper<'a> {
    level: &'a tracing::Level,
}

impl Serialize for LevelWrapper<'_> {
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
struct FieldSetWrapper<'a> {
    field_set: &'a tracing::field::FieldSet,
}

impl Serialize for FieldSetWrapper<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_seq(Some(self.field_set.len()))?;

        for field in self.field_set {
            state.serialize_element(field.name())?;
        }

        state.end()
    }
}

#[derive(Debug)]
struct KindWrapper {
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
                serializer.serialize_unit_variant("OverrideStatus", 2, "ExplicitlyDisabled")
            }
        }
    }
}
