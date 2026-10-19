//! A tracing filter that can be finetuned to enable or disable specific callsites.
//!
//! This crate provides [`FinetuneFilter`] which implements the [`Filter`] trait from
//! [`tracing-subscriber`]. It wraps another filter, but can override whether or not specific
//! callsites (places where events or spans are created) are recorded. This finetuning can be
//! performed at runtime.
//!
//! [`tracing-subscriber`]: tracing_subscriber
//!
//! ## Usage
//!
//! A finetune filter can wrap any other filter and then be added to filter a layer.
//!
//! ```
//! use tracing_subscriber::prelude::*;
//! use tracing_finetune::FinetuneFilter;
//!
//! let filter = tracing_subscriber::filter::LevelFilter::WARN;
//! let finetune = FinetuneFilter::new(filter);
//! let finetune_handle = finetune.handle();
//! let subscriber = tracing_subscriber::registry()
//!     .with(tracing_subscriber::fmt::layer().with_filter(finetune))
//!     .init();
//!
//! fn info_event() {
//!     tracing::info!("hi there!");
//! }
//!
//! fn warn_event() {
//!     tracing::warn!("important!");
//! }
//!
//! info_event(); // Doesn't match WARN filter: not emitted
//! warn_event(); // Matches WARN filter: emitted
//!
//! let callsites = finetune_handle.callsites();
//!
//! assert_eq!(callsites.len(), 2_usize);
//! // Callsites are returned in the order they are encounted during execution
//! let info_callsite = &callsites[0];
//! let warn_callsite = &callsites[1];
//!
//! // Here we explicitly enable the info callsite
//! finetune_handle.enable_callsite(info_callsite.id());
//! // and explicitly disable the warn callsite
//! finetune_handle.disable_callsite(warn_callsite.id());
//!
//!
//! info_event(); // Explictly enabled: emitted
//! warn_event(); // Explicitly disabled: not emitted
//! ```
//!
//! ## Supported Rust Versions
//!
//! `tracing-finetune` is built against the latest stable release. The minimum supported version is
//! 1.90 ([What's New in Rust 1.88 through 1.90]). The current version of `tracing-finetune` is not
//! guaranteed to build on Rust versions earlier than the minimum supported version.
//!
//! The Minimum Supported Rust Version (MSRV) policy is that it won't be updated to a version that
//! hasn't been discussed by Jon and Ben on the [Rustacean Station] podcast.
//!
//! [What's New in Rust 1.88 through 1.90]: https://rustacean-station.org/episode/rust-1.88-1.89-1.90/
//! [Rustacean Station]: https://rustacean-station.org
//!
//! ## License
//!
//! This project is licensed under the [MIT license].
//!
//! [MIT license]: https://github.com/hds/ari_subscriber/blob/main/LICENSE
//!
//! ### Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
//! in `tracing-finetune` by you, shall be licensed as MIT, without any additional terms or
//! conditions.
#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
use std::{
    marker::PhantomData,
    ptr,
    sync::{Arc, Mutex},
};

use arc_swap::ArcSwap;
use indexmap::IndexMap;
use tracing::{Metadata, Subscriber, subscriber::Interest};
use tracing_subscriber::layer::{Context, Filter};

/// Callsite
///
/// A representation of a tracing callsite for the [`FinetuneFilter`]. This can be used to retrieve
/// the callsite's metadata and the interest that the wrapped filter provided for it. It also
/// contains the finetune-internal [`CallsiteId`] which can be used to add an override to enable or
/// disable the callsite.
#[derive(Debug, Clone)]
pub struct Callsite {
    id: CallsiteId,
    meta: &'static Metadata<'static>,
    inner_filter_interest: Interest,
    /// The override status is always stored as `NoOverride` on the [`FinetuneFilter`], it only
    /// gets filled in when the list is exported via [`callsites`].
    ///
    /// [`callsites`]: FinetuneFilter::callsites
    override_status: OverrideStatus,
}

/// Callsite override status
///
/// This enum specified the override status of a specific callsite for a [`FinetuneFilter`].
#[derive(Debug, Clone, Copy)]
pub enum OverrideStatus {
    /// There is no override present
    NoOverride,
    /// The callsite is explicitly enabled
    ExplicitlyEnabled,
    /// The callsite is explicitly disabled
    ExplicitlyDisabled,
}

impl Callsite {
    fn new(
        id: CallsiteId,
        meta: &'static Metadata<'static>,
        inner_filter_interest: Interest,
    ) -> Self {
        Self {
            id,
            meta,
            inner_filter_interest,
            override_status: OverrideStatus::NoOverride,
        }
    }

    /// This callsite's finetune-internal Id.
    ///
    /// This Id can be used to add an override via [`enable_callsite`] or [`disable_callsite`] on a
    /// [`FinetuneHandle`].
    ///
    /// [`enable_callsite`]: FinetuneHandle::enable_callsite
    /// [`disable_callsite`]: FinetuneHandle::disable_callsite
    #[must_use]
    pub fn id(&self) -> CallsiteId {
        self.id
    }

    /// This callsite's [`Metadata`].
    ///
    /// Returns the `tracing` [`Metadata`] for this callsite.
    #[must_use]
    pub fn metadata(&self) -> &'static tracing::Metadata<'static> {
        self.meta
    }

    /// The [`Interest`] returned by the wrapped filter.
    ///
    /// This is the interest that the wrapped filter returned for this callsite.
    #[must_use]
    pub fn interest(&self) -> tracing::subscriber::Interest {
        self.inner_filter_interest.clone()
    }

    /// The override status of this callsite.
    ///
    /// This value provides the override status of the callsite represented by this [`Callsite`]
    /// object at the time it was created.
    ///
    /// Subsequent calls to [`enable_callsite`] or [`disable_callsite`] will not be reflected
    /// until the list of callsites is fetched again.
    ///
    /// [`enable_callsite`]: FinetuneHandle::enable_callsite
    /// [`disable_callsite`]: FinetuneHandle::disable_callsite
    #[must_use]
    pub fn status(&self) -> OverrideStatus {
        self.override_status
    }
}

/// Callsite identfier
///
/// Uniquely identifies a callsite within the [`FinetuneFilter`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct CallsiteId(u64);

impl CallsiteId {
    fn new_from_metadata(meta: &Metadata<'_>) -> Self {
        Self(ptr::from_ref(meta) as u64)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum OverrideAction {
    ExplicitlyEnabled,
    ExplicitlyDisabled,
}

#[derive(Debug, Clone, Copy)]
struct CallsiteOverride {
    id: CallsiteId,
    action: OverrideAction,
}

/// A tracing filter that can be finetuned to enable and disable specific callsites.
#[derive(Debug)]
pub struct FinetuneFilter<S, F>
where
    F: Filter<S>,
    S: Subscriber,
{
    inner_filter: F,
    handle: FinetuneHandle,
    _subscriber: PhantomData<S>,
}

impl<S, F> FinetuneFilter<S, F>
where
    S: Subscriber,
    F: Filter<S>,
{
    /// Create a new finetune filter wrapping another filter.
    pub fn new(inner_filter: F) -> Self {
        Self {
            inner_filter,
            handle: FinetuneHandle::default(),
            _subscriber: PhantomData {},
        }
    }

    /// The handle to this finetune filter to enable and disable callsites.
    pub fn handle(&self) -> FinetuneHandle {
        self.handle.clone()
    }
}

/// A handle to enable and disable callsites on the associated [`FinetuneFilter`].
#[derive(Debug, Default, Clone)]
pub struct FinetuneHandle {
    /// All callsites observed by the associated filter. It is effectively append only.
    callsites: Arc<Mutex<IndexMap<CallsiteId, Callsite>>>,

    callsite_overrides: Arc<ArcSwap<Vec<CallsiteOverride>>>,
}

impl FinetuneHandle {
    /// Enable the given callsite so that it is always recorded.
    ///
    /// Set an override on the finetune filter to always treat the span or event coming from the
    /// callsite as enabled.
    ///
    /// After this call, [`callsite_enabled`] will always return [`Interest::always()`] and
    /// [`enabled`] will always return `true`.
    ///
    /// The override can be removed with [`clear_callsite_override`].
    ///
    /// [`callsite_enabled`]: Filter::callsite_enabled
    /// [`enabled`]: Filter::enabled
    /// [`clear_callsite_override`]: Self::clear_callsite_override
    pub fn enable_callsite(&self, callsite_id: CallsiteId) {
        self.set_callsite_override(callsite_id, OverrideAction::ExplicitlyEnabled);
    }

    /// Disable the given callsite so that it is never recorded.
    ///
    /// Set an override on the finetune filter to always treat the span or event coming from the
    /// callsite as disabled.
    ///
    /// After this call, [`callsite_enabled`] will always return [`Interest::never()`] and
    /// [`enabled`] will always return `false`.
    ///
    /// The override can be removed with [`clear_callsite_override`].
    ///
    /// [`callsite_enabled`]: Filter::callsite_enabled
    /// [`enabled`]: Filter::enabled
    /// [`clear_callsite_override`]: Self::clear_callsite_override
    pub fn disable_callsite(&self, callsite_id: CallsiteId) {
        self.set_callsite_override(callsite_id, OverrideAction::ExplicitlyDisabled);
    }

    /// Remove any override present for this callsite.
    ///
    /// This method will remove an override previously set by [`enable_callsite`] or
    /// [`disable_callsite`].
    ///
    /// After this call, [`callsite_enabled`] and [`enabled`] will return the value provided by the
    /// wrapped filter.
    ///
    /// [`callsite_enabled`]: Filter::callsite_enabled
    /// [`enabled`]: Filter::enabled
    /// [`enable_callsite`]: Self::enable_callsite
    /// [`disable_callsite`]: Self::disable_callsite
    pub fn clear_callsite_override(&self, callsite_id: CallsiteId) {
        self.callsite_overrides.rcu(|callsite_overrides| {
            let mut callsite_overrides = Vec::<CallsiteOverride>::clone(callsite_overrides);
            for (idx, callsite_override) in callsite_overrides.iter().enumerate() {
                if callsite_override.id == callsite_id {
                    callsite_overrides.swap_remove(idx);
                    break;
                }
            }

            callsite_overrides
        });
        tracing_core::callsite::rebuild_interest_cache();
    }

    /// The callsites that his filter has observed.
    ///
    /// In `tracing`, callsites are observed by active subscribers (and their filters) when they
    /// are first recorded after the subscriber becomes active.
    ///
    /// The finetune filter records the callsites which have been observed and can then be
    /// explicitly enabled or disabled.
    ///
    /// # Guarantees
    ///
    /// The vector of callsites are stored and returned in the order in which they were recorded.
    /// As such, from one call to this method to the next, the size of the vector will remain the
    /// same or increase and the elements returned from the first call will be in the same position
    /// in the second call.
    ///
    /// # Panics
    ///
    /// This method will panic if the internal mutex becomes poisoned.
    #[must_use]
    pub fn callsites(&self) -> Vec<Callsite> {
        let callsites = self
            .callsites
            .lock()
            .expect("Callsite records are poisoned!");

        callsites
            .values()
            .map(|c| {
                let mut callsite = c.clone();
                let callsite_overrides = self.callsite_overrides.load();
                for callsite_override in callsite_overrides.iter() {
                    if callsite_override.id == callsite.id {
                        callsite.override_status = match callsite_override.action {
                            OverrideAction::ExplicitlyEnabled => OverrideStatus::ExplicitlyEnabled,
                            OverrideAction::ExplicitlyDisabled => {
                                OverrideStatus::ExplicitlyDisabled
                            }
                        };
                    }
                }
                callsite
            })
            .collect()
    }

    fn set_callsite_override(&self, callsite_id: CallsiteId, status: OverrideAction) {
        self.callsite_overrides.rcu(|callsite_overrides| {
            let mut found = false;
            let mut callsite_overrides = Vec::<CallsiteOverride>::clone(callsite_overrides);
            for callsite_override in &mut callsite_overrides {
                if callsite_override.id == callsite_id {
                    callsite_override.action = status;
                    found = true;
                    break;
                }
            }
            if !found {
                callsite_overrides.push(CallsiteOverride {
                    id: callsite_id,
                    action: status,
                });
            }

            callsite_overrides
        });
        tracing_core::callsite::rebuild_interest_cache();
    }

    fn upsert_callsite(
        &self,
        callsite_id: CallsiteId,
        meta: &'static Metadata<'static>,
        inner_filter_interest: Interest,
    ) {
        let mut callsites = self
            .callsites
            .lock()
            .expect("Callsite records are poisoned!");

        println!(
            "Upserting: {callsite_id:?} ({callsite_hex:#x})\n    - {meta:?}\n    - {inner_filter_interest:?}",
            callsite_hex = callsite_id.0
        );
        callsites
            .entry(callsite_id)
            .and_modify(|callsite| callsite.inner_filter_interest = inner_filter_interest.clone())
            .or_insert_with(|| Callsite::new(callsite_id, meta, inner_filter_interest));
    }

    fn callsite_status(&self, callsite_id: CallsiteId) -> Option<OverrideAction> {
        let callsite_overrides = self.callsite_overrides.load();
        for callsite_override in callsite_overrides.iter() {
            if callsite_override.id == callsite_id {
                return Some(callsite_override.action);
            }
        }

        None
    }
}

impl<S, F> Filter<S> for FinetuneFilter<S, F>
where
    S: Subscriber,
    F: Filter<S>,
{
    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        let callsite_id = CallsiteId::new_from_metadata(meta);

        let inner_filter_interest = self.inner_filter.callsite_enabled(meta);
        self.handle
            .upsert_callsite(callsite_id, meta, inner_filter_interest.clone());

        match self.handle.callsite_status(callsite_id) {
            Some(OverrideAction::ExplicitlyEnabled) => Interest::always(),
            Some(OverrideAction::ExplicitlyDisabled) => Interest::never(),
            None => inner_filter_interest,
        }
    }

    fn enabled(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool {
        match self
            .handle
            .callsite_status(CallsiteId::new_from_metadata(meta))
        {
            Some(OverrideAction::ExplicitlyEnabled) => true,
            Some(OverrideAction::ExplicitlyDisabled) => false,
            // Fallback to the inner filter
            None => self.inner_filter.enabled(meta, cx),
        }
    }
}
