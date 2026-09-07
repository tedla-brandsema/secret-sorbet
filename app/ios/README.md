# Secret Sorbet iOS App

This directory is reserved for the native iOS application.

The iOS app should use SwiftUI and bind to the shared Rust crates through a
stable FFI layer, likely UniFFI or a C ABI. Platform-specific functionality such
as Keychain, LocalAuthentication, autofill, and secure clipboard behavior belongs
here rather than in `lib/core`.

