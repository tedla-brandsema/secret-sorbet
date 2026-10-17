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
| [P06](ROADMAP.md#p06) | Record explicit product decision on opt-in NFC, new profile/version and recovery migration; preserve v1. | P02/P03 |
| [P07](ROADMAP.md#p07) | Build mobile core harnesses, then execute fixtures on iOS/Android simulators/emulators and named physical devices; keep app availability claims separate. | P02–P05 |
| [P08](ROADMAP.md#p08) | Complete required gates, independent review, documentation consistency, evidence-based support claims, and changelog preparation for a later authorized release. | P01–P07 |
| [R01–R03](ROADMAP.md#v010-development-source-release) | Prepare, verify, review and publish the separately authorized v0.1.0 development source release. | P01–P05 and release authorization |

The authorized P01–P05 delivery through Phase 2 is complete. P06 and P07 remain
deferred at the user's request; P08 remains the broader portability gate.

P06 requires a decision record. Implementing an NFC profile requires explicit
approval and a subsequent plan; this TODO does not approve a compatibility
change. P07 remains future work while mobile tooling/devices are unavailable;
record specific partial evidence and remaining work instead of closing it.

R02 is active for the authorized [v0.1.0 source release](plans/v0.1.0-release.md).
Its separate verification and publication gate does not close P08 or waive its
missing mobile requirements. Record new accepted unfinished work here and link
it to the roadmap.
