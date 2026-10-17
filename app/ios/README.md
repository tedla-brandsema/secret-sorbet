# Secret Sorbet iOS App

This directory is reserved for the native iOS application.

No iOS binding or application is implemented here. Mobile fixture execution
(P07) is deferred; see the [roadmap](../../docs/ROADMAP.md#p07) and the shared
[v1 input contract](../../docs/support.md). Desktop evidence does not verify iOS.

The iOS app should use SwiftUI and bind to the shared Rust crates through a
stable FFI layer, likely UniFFI or a C ABI. Platform-specific functionality such
as Keychain, LocalAuthentication, autofill, and secure clipboard behavior belongs
here rather than in `lib/core`.
