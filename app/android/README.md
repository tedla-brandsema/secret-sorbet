# Secret Sorbet Android App

This directory is reserved for the native Android application.

No Android binding or application is implemented here. Mobile fixture execution
(P07) is deferred; see the [roadmap](../../docs/ROADMAP.md#p07) and the shared
[v1 input contract](../../docs/support.md). Desktop evidence does not verify Android.

The Android app should use Kotlin/Compose and bind to the shared Rust crates
through a stable FFI layer, likely UniFFI or JNI. Platform-specific functionality
such as Android Keystore, BiometricPrompt, autofill, and secure clipboard
behavior belongs here rather than in `lib/core`.
