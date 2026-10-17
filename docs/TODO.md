# Canonical implementation TODO

This file records accepted unfinished implementation work. The
[portability plan](plans/portability-and-determinism.md) defines scope, ownership,
dependencies, verification, documentation, and suggested commits for each slice.
The [roadmap](ROADMAP.md) is authoritative for phase and slice progress, the checkbox
legend, and completion evidence. Writing a plan does not complete an
implementation item. Add future plans under `docs/plans/`.

Use the roadmap's statuses rather than adding parallel checkboxes here. Do not
mark configured CI or compilation as executed compatibility. Remove resolved
backlog items after recording their evidence and commit in the roadmap.

| ID | Accepted work | Depends on |
| --- | --- | --- |
| [P01](ROADMAP.md#p01) | Fix >256 free-position nontermination with u64 big-endian sampling above 256; preserve all successful v1 bytes, including long no-required policies; extend bounded regression/fuzz tests. | — |
| [P02](ROADMAP.md#p02) | Expand synthetic pinned full derivation and intermediate byte fixtures, raw/effective login fields, Unicode and invalid/boundary cases. | P01 for long policies |
| [P03](ROADMAP.md#p03) | Document std/heap, 64 MiB plus overhead, dependency/thread and 32/64-bit requirements; v1 Unicode baseline and toolchain drift. | P02; final evidence P04/P05 |
| [P04](ROADMAP.md#p04) | Execute debug/release fixtures on all six configured native desktop targets and retain evidence. | P02 |
| [P05](ROADMAP.md#p05) | Execute i686, ARMv7 and big-endian s390x vectors under emulation; label compilation separately. | P01/P02; coordinate P04 |
| [P06](ROADMAP.md#p06) | Record explicit product decision on opt-in NFC, new profile/version and recovery migration; preserve v1. | P02/P03 |
| [P07](ROADMAP.md#p07) | Build mobile core harnesses, then execute fixtures on iOS/Android simulators/emulators and named physical devices; keep app availability claims separate. | P02–P05 |
| [P08](ROADMAP.md#p08) | Complete required gates, independent review, documentation consistency, evidence-based support claims, and changelog preparation for a later authorized release. | P01–P07 |

P06 requires a decision record. Implementing an NFC profile requires explicit
approval and a subsequent plan; this TODO does not approve a compatibility
change. P07 remains future work while mobile tooling/devices are unavailable;
record specific partial evidence and remaining work instead of closing it.

Recommended next slice: **P01**, before expanding long-policy coverage. Missing
required execution keeps the owning slice and release phase partial in the
roadmap; record the exact remaining target/check here and do not treat documented
limits as a waiver.
