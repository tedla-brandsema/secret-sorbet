# Secret Sorbet Browser Extensions

This directory is reserved for browser extensions.

First-class browser targets:

- Firefox
- Chrome and Chromium-based browsers
- Safari

The extensions should support deterministic generation, vault lookup, autofill,
and authenticator workflows where browser APIs allow it.

Browser extensions are part of the free client surface. Secret Sorbet-hosted
cloud synchronization is the only planned revenue mode.

The extensions should use the shared core implementation or generated bindings
rather than reimplementing deterministic derivation independently.

