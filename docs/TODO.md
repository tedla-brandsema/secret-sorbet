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
| [P03](ROADMAP.md#p03) | Document std/heap, 64 MiB plus overhead, dependency/thread and 32/64-bit requirements; v1 Unicode baseline and toolchain drift. | P02; final evidence P04/P05 |
| [P06](ROADMAP.md#p06) | Record explicit product decision on opt-in NFC, new profile/version and recovery migration; preserve v1. | P02/P03 |
| [P07](ROADMAP.md#p07) | Build mobile core harnesses, then execute fixtures on iOS/Android simulators/emulators and named physical devices; keep app availability claims separate. | P02–P05 |
| [P08](ROADMAP.md#p08) | Complete required gates, independent review, documentation consistency, evidence-based support claims, and changelog preparation for a later authorized release. | P01–P07 |

Current authorization covers P01–P05 through Phase 2. P06 and P07 are deferred
at the user's request; P08 is outside this delivery boundary.

P06 requires a decision record. Implementing an NFC profile requires explicit
approval and a subsequent plan; this TODO does not approve a compatibility
change. P07 remains future work while mobile tooling/devices are unavailable;
record specific partial evidence and remaining work instead of closing it.

Active slice: **P03**, publishing support documentation with verified P04/P05
execution evidence. Missing
required execution keeps the owning slice and release phase partial in the
roadmap; record the exact remaining target/check here and do not treat documented
limits as a waiver.
