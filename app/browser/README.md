# Secret Sorbet Browser Extensions

This directory is reserved for browser extensions.

No extension or browser binding is implemented here. Future bindings must pass
the [shared compatibility fixtures](../../test-vectors/README.md) and preserve
the [v1 byte and Unicode contract](../../docs/support.md), including input
preparation and profile metadata. Browser execution is not currently verified.

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
