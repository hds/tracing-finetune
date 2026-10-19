use tracing::Level;
use tracing_mock::{expect, layer};
use tracing_subscriber::prelude::*;

use tracing_finetune::FinetuneFilter;

fn info_event(msg: &str) {
    tracing::info!(target: "finetune", "{}", msg);
}

fn other_info_event() {
    tracing::info!(target: "other", "other info");
}

fn warn_event(msg: &str) {
    tracing::warn!(target: "finetune", "{}", msg);
}

fn other_warn_event() {
    tracing::warn!(target: "other", "other warn");
}

/// This test replicates what is shown in the Usage section of the crate docs in lib.rs
#[test]
fn librs_usage_example() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::WARN,
            "first warn event",
        ))
        .event(expect_event_msg(
            "finetune",
            Level::INFO,
            "second info event",
        ))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        info_event("first info event");
        warn_event("first warn event");

        let callsites = finetune_handle.callsites();

        let info_callsite = get_callsite(&callsites, "finetune", Level::INFO);
        let warn_callsite = get_callsite(&callsites, "finetune", Level::WARN);
        finetune_handle.enable_callsite(info_callsite.id());
        finetune_handle.disable_callsite(warn_callsite.id());

        info_event("second info event");
        warn_event("second warn event");
    }

    handle.assert_finished();
}

#[test]
fn enable_callsite() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::INFO,
            "second info event",
        ))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        info_event("first info event");

        let callsites = finetune_handle.callsites();

        let info_callsite = get_callsite(&callsites, "finetune", Level::INFO);
        finetune_handle.enable_callsite(info_callsite.id());

        info_event("second info event");
    }

    handle.assert_finished();
}

#[test]
fn disable_callsite() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::WARN,
            "first warn event",
        ))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        warn_event("first warn event");

        let callsites = finetune_handle.callsites();

        let warn_callsite = get_callsite(&callsites, "finetune", Level::WARN);
        finetune_handle.disable_callsite(warn_callsite.id());

        warn_event("second warn event");
    }

    handle.assert_finished();
}

#[test]
fn enable_then_disable() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::INFO,
            "second info event",
        ))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        info_event("first info event");

        let callsites = finetune_handle.callsites();

        let info_callsite = get_callsite(&callsites, "finetune", Level::INFO);
        finetune_handle.enable_callsite(info_callsite.id());

        info_event("second info event");

        finetune_handle.disable_callsite(info_callsite.id());
        info_event("third info event");
    }

    handle.assert_finished();
}

#[test]
fn disable_then_enable() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::WARN,
            "first warn event",
        ))
        .event(expect_event_msg(
            "finetune",
            Level::WARN,
            "third warn event",
        ))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        warn_event("first warn event");

        let callsites = finetune_handle.callsites();

        let warn_callsite = get_callsite(&callsites, "finetune", Level::WARN);
        finetune_handle.disable_callsite(warn_callsite.id());

        warn_event("second warn event");

        finetune_handle.enable_callsite(warn_callsite.id());
        warn_event("third warn event");
    }

    handle.assert_finished();
}

#[test]
fn enable_then_clear() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::INFO,
            "second info event",
        ))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        info_event("first info event");

        let callsites = finetune_handle.callsites();

        let info_callsite = get_callsite(&callsites, "finetune", Level::INFO);
        finetune_handle.enable_callsite(info_callsite.id());

        info_event("second info event");

        finetune_handle.clear_callsite_override(info_callsite.id());
        info_event("third info event");
    }

    handle.assert_finished();
}

#[test]
fn disable_then_clear() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::WARN,
            "first warn event",
        ))
        .event(expect_event_msg(
            "finetune",
            Level::WARN,
            "third warn event",
        ))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        warn_event("first warn event");

        let callsites = finetune_handle.callsites();

        let warn_callsite = get_callsite(&callsites, "finetune", Level::WARN);
        finetune_handle.disable_callsite(warn_callsite.id());

        warn_event("second warn event");

        finetune_handle.clear_callsite_override(warn_callsite.id());
        warn_event("third warn event");
    }

    handle.assert_finished();
}

#[test]
fn enable_does_not_affect_other_callsites() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::INFO,
            "second info event",
        ))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        info_event("first info event");
        other_info_event();

        let callsites = finetune_handle.callsites();

        let info_callsite = get_callsite(&callsites, "finetune", Level::INFO);
        finetune_handle.enable_callsite(info_callsite.id());

        info_event("second info event");
        other_info_event();
    }

    handle.assert_finished();
}

#[test]
fn disable_does_not_affect_other_callsites() {
    let filter = tracing_subscriber::filter::LevelFilter::WARN;
    let finetune = FinetuneFilter::new(filter);
    let finetune_handle = finetune.handle();

    let (layer, handle) = layer::mock()
        .event(expect_event_msg(
            "finetune",
            Level::WARN,
            "first warn event",
        ))
        .event(expect_event_msg("other", Level::WARN, "other warn"))
        .event(expect_event_msg("other", Level::WARN, "other warn"))
        .only()
        .run_with_handle();

    let subscriber = tracing_subscriber::registry().with(layer.with_filter(finetune));

    {
        let _guard = tracing::subscriber::set_default(subscriber);

        warn_event("first warn event");
        other_warn_event();

        let callsites = finetune_handle.callsites();

        let warn_callsite = get_callsite(&callsites, "finetune", Level::WARN);
        finetune_handle.disable_callsite(warn_callsite.id());

        warn_event("second warn event");
        other_warn_event();
    }

    handle.assert_finished();
}

/// Find the first callsite in the list with the matching target and level
fn get_callsite<'a>(
    callsites: &'a [tracing_finetune::Callsite],
    target: &str,
    level: Level,
) -> &'a tracing_finetune::Callsite {
    callsites
        .iter()
        .find(|c| c.metadata().target() == target && *c.metadata().level() == level)
        .unwrap_or_else(|| panic!("could not find callsite with target={target} and level={level}"))
}

/// Create an [`ExpectedEvent`] with the provided level and message.
///
/// [`ExpectedEvent`]: tracing_mock::event::ExpectedEvent
fn expect_event_msg(target: &str, level: Level, msg: &str) -> tracing_mock::event::ExpectedEvent {
    expect::event()
        .with_target(target)
        .at_level(level)
        .with_fields(expect::msg(msg))
}
