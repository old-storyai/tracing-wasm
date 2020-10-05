# `tracing-wasm`

Leverage performance profiling with your browser tools with the [tracing crate](https://crates.io/crates/tracing).

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![APACHE licensed][apache-2-badge]][apache-2-url]

[crates-badge]: https://img.shields.io/crates/v/tracing-wasm.svg
[crates-url]: https://crates.io/crates/tracing-wasm
[docs-badge]: https://docs.rs/tracing-wasm/badge.svg
[docs-url]: https://docs.rs/tracing-wasm
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE-MIT
[apache-2-badge]: https://img.shields.io/badge/license-APACHE%202.0-blue.svg
[apache-2-url]: LICENSE-APACHE

![Screenshot of performance reported using the `tracing-wasm` Subscriber](./2020-07-10-devtools-demo-screenshot.png)

Note: `tracing_wasm` uses the global JavaScript `console` and `performance` objects. It will not work in environments where one or both of these are not available, such as Node.js or Cloudflare Workers.

## Usage

For the simplest out of the box set-up, you can simply set `tracing_wasm` as your default tracing Subscriber in wasm_bindgen(start)

We have this declared in our `./src/lib.rs`

```rust
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    // This is not needed for tracing_wasm to work, but it is a common tool for getting proper error line numbers for panics.
    console_error_panic_hook::set_once();

    // Add this line:
    tracing_wasm::set_as_global_default();

    Ok(())
}
```
