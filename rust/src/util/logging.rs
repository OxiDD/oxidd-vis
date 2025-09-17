use wasm_bindgen::prelude::*;

// First up let's take a look of binding `console.log` manually, without the
// help of `web_sys`. Here we're writing the `#[wasm_bindgen]` annotations
// manually ourselves, and the correctness of our program relies on the
// correctness of these annotations!

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = time)]
    pub fn time(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = timeEnd)]
    pub fn time_end(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

macro_rules! log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ({
        use crate::util::logging::log;
        log(&format_args!($($t)*).to_string())}
    )
}
macro_rules! time  {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ({
        use crate::util::logging::time;
        time(&format_args!($($t)*).to_string())}
    )
}
macro_rules! time_end {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ({
        use crate::util::logging::time_end;
        time_end(&format_args!($($t)*).to_string())}
    )
}

// Next let's define a macro that's like `println!`, only it works for
// `console.log`. Note that `println!` doesn't actually work on the wasm target
// because the standard library currently just eats all output. To get
// `println!`-like behavior in your app you'll likely want a macro like this.
pub mod console {

    pub(crate) use log;
    pub(crate) use time;
    pub(crate) use time_end;
}
