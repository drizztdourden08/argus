//! The cases of the Node `test/notify.test.mjs` that are not the toast XML (those live in
//! `argus-native`), run against fakes. No real toast is ever shown.
//!
//! Not ported, because the notifier is native code and no PowerShell is involved: the `toastScript`
//! group (a constant script that carries no input, and its parse check) and the `createOsToast`
//! cases of a fake PowerShell (the spawn arguments, a failing PowerShell, the guard against a
//! real toast in a test run). The failing platform is `a_failing_platform_is_an_error_not_a_panic`
//! in `argus-native`, and the XML loading into the real `WinRT` document is `toast_winrt.rs`.

mod auto;
mod delivery;
mod harness;
mod levels;
mod limits;
mod records;
mod settings;
