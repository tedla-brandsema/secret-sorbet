# Secret Sorbet Android App

This directory is reserved for the native Android application.

The Android app should use Kotlin/Compose and bind to the shared Rust crates
through a stable FFI layer, likely UniFFI or JNI. Platform-specific functionality
such as Android Keystore, BiometricPrompt, autofill, and secure clipboard
behavior belongs here rather than in `lib/core`.

