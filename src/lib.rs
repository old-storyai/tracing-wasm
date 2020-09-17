use core::fmt::{self, Write};
use core::sync::atomic::AtomicUsize;

use tracing::field::{Field, Visit};
use tracing::Subscriber;
use tracing_subscriber::layer::*;
use tracing_subscriber::registry::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = performance)]
    fn mark(a: &str);
    #[wasm_bindgen(js_namespace = performance)]
    fn measure(name: &str, startMark: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log1(message: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log2(message1: &str, message2: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log3(message1: &str, message2: &str, message3: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log4(message1: &str, message2: &str, message3: &str, message4: &str);
}

pub struct WASMLayerConfig {
    /// Log events will be marked and measured so they appear in performance Timings
    pub report_logs_in_timings: bool,
    /// Log events will be logged to the browser console
    pub report_logs_in_console: bool,
    /// Only relevant if report_logs_in_console is true, this will use color style strings in the console.
    pub use_console_color: bool,
}

impl core::default::Default for WASMLayerConfig {
    fn default() -> Self {
        WASMLayerConfig {
            report_logs_in_timings: true,
            report_logs_in_console: true,
            use_console_color: true,
        }
    }
}

/// Implements [tracing_subscriber::layer::Layer] which uses [wasm_bindgen] for marking and measuring with `window.performance`
pub struct WASMLayer {
    last_event_id: AtomicUsize,
    config: WASMLayerConfig,
}

impl WASMLayer {
    pub fn new(config: WASMLayerConfig) -> Self {
        WASMLayer {
            last_event_id: AtomicUsize::new(0),
            config,
        }
    }
}

impl core::default::Default for WASMLayer {
    fn default() -> Self {
        WASMLayer::new(WASMLayerConfig::default())
    }
}

fn mark_name(id: &tracing::Id) -> String {
    format!("t{:x}", id.into_u64())
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for WASMLayer {
    fn new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
        let mut new_debug_record = StringRecorder::new();
        attrs.record(&mut new_debug_record);

        if let Some(span_ref) = ctx.span(id) {
            span_ref
                .extensions_mut()
                .insert::<StringRecorder>(new_debug_record);
        }
    }

    /// doc: Notifies this layer that a span with the given Id recorded the given values.
    fn on_record(&self, id: &tracing::Id, values: &tracing::span::Record<'_>, ctx: Context<'_, S>) {
        if let Some(span_ref) = ctx.span(id) {
            if let Some(debug_record) = span_ref.extensions_mut().get_mut::<StringRecorder>() {
                values.record(debug_record);
            }
        }
    }

    // /// doc: Notifies this layer that a span with the ID span recorded that it follows from the span with the ID follows.
    // fn on_follows_from(&self, _span: &tracing::Id, _follows: &tracing::Id, ctx: Context<'_, S>) {}
    /// doc: Notifies this layer that an event has occurred.
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if self.config.report_logs_in_timings || self.config.report_logs_in_console {
            let mut recorder = StringRecorder::new();
            event.record(&mut recorder);
            let meta = event.metadata();
            let level = meta.level();
            if self.config.report_logs_in_console {
                let origin = meta
                    .file()
                    .and_then(|file| meta.line().map(|ln| format!("{}:{}", file, ln)))
                    .unwrap_or_default();
                if self.config.use_console_color {
                    log4(
                        &format!("%c{}%c {}%c{}", level, origin, recorder),
                        match *level {
                            tracing::Level::TRACE => "color: dodgerblue; background: #444",
                            tracing::Level::DEBUG => "color: lawngreen; background: #444",
                            tracing::Level::INFO => "color: whitesmoke; background: #444",
                            tracing::Level::WARN => "color: orange; background: #444",
                            tracing::Level::ERROR => "color: red; background: #444",
                        },
                        "color: gray; font-style: italic",
                        "color: inherit",
                    );
                } else {
                    log1(&format!("{} {}{}", level, origin, recorder));
                }
            }
            if self.config.report_logs_in_timings {
                let mark_name = format!(
                    "c{:x}",
                    self.last_event_id
                        .fetch_add(1, core::sync::atomic::Ordering::Relaxed)
                );
                // mark and measure so you can see a little blip in the profile
                mark(&mark_name);
                measure(
                    &format!(
                        "{} {}{}",
                        level,
                        meta.module_path().unwrap_or("..."),
                        recorder
                    ),
                    &mark_name,
                );
            }
        }
    }
    /// doc: Notifies this layer that a span with the given ID was entered.
    fn on_enter(&self, id: &tracing::Id, _ctx: Context<'_, S>) {
        mark(&mark_name(id));
    }
    /// doc: Notifies this layer that the span with the given ID was exited.
    fn on_exit(&self, id: &tracing::Id, ctx: Context<'_, S>) {
        if let Some(span_ref) = ctx.span(id) {
            let meta = span_ref.metadata();
            if let Some(debug_record) = span_ref.extensions().get::<StringRecorder>() {
                measure(
                    &format!(
                        "\"{}\" {} {}",
                        meta.name(),
                        meta.module_path().unwrap_or("..."),
                        debug_record,
                    ),
                    &mark_name(id),
                )
            } else {
                measure(
                    &format!(
                        "\"{}\" {}",
                        meta.name(),
                        meta.module_path().unwrap_or("..."),
                    ),
                    &mark_name(id),
                )
            }
        }
    }
    // /// doc: Notifies this layer that the span with the given ID has been closed.
    // /// We can dispose of any data for the span we might have here...
    // fn on_close(&self, _id: tracing::Id, ctx: Context<'_, S>) {}
    // /// doc: Notifies this layer that a span ID has been cloned, and that the subscriber returned a different ID.
    // /// I'm not sure if I need to do something here...
    // fn on_id_change(&self, _old: &tracing::Id, _new: &tracing::Id, ctx: Context<'_, S>) {}
}

/// Set the global default with [tracing::subscriber::set_global_default]
pub fn set_as_global_default() {
    tracing::subscriber::set_global_default(
        Registry::default().with(WASMLayer::new(WASMLayerConfig::default())),
    )
    .expect("default global");
}

/// Set the global default with [tracing::subscriber::set_global_default]
pub fn set_as_global_default_with_config(config: WASMLayerConfig) {
    tracing::subscriber::set_global_default(Registry::default().with(WASMLayer::new(config)))
        .expect("default global");
}

struct StringRecorder(String, bool);
impl StringRecorder {
    fn new() -> Self {
        StringRecorder(String::new(), false)
    }
}

impl Visit for StringRecorder {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            if !self.0.is_empty() {
                self.0 = format!("{:?}\n{}", value, self.0)
            } else {
                self.0 = format!("{:?}", value)
            }
        } else {
            if self.1 {
                // following args
                writeln!(self.0, "").unwrap();
            } else {
                // first arg
                write!(self.0, " ").unwrap();
                self.1 = true;
            }
            write!(self.0, "{} = {:?};", field.name(), value).unwrap();
        }
    }
}

impl core::fmt::Display for StringRecorder {
    fn fmt(&self, mut f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if !self.0.is_empty() {
            write!(&mut f, " {}", self.0)
        } else {
            Ok(())
        }
    }
}

impl core::default::Default for StringRecorder {
    fn default() -> Self {
        StringRecorder::new()
    }
}
