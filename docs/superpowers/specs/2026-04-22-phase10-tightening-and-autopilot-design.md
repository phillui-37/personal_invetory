# Phase 10 Design Spec — Backlog Tightening and Automatic Execution

**Date**: 2026-04-22  
**Author**: Copilot + Phil  
**Status**: Approved design

---

## 1. Goal

Review `tasks/phase10.md`, improve unclear or missing parts, then create an implementation plan that:

1. tightens the backlog into something honest and executable
2. separates ready work from externally blocked work
3. starts automatic execution with the highest-leverage ready tasks first

This design is for **Phase 10 planning and execution control**, not for implementing every Phase 10 task in one blind sweep.

---

## 2. Current-State Findings

The live repo shows these real conditions:

- `tasks/phase10.md` is the canonical remaining-work file.
- `P10-A0` is complete.
- connector tasks still contain real `TODO(network-inspection)` debt in:
  - `backend/plugins/src/ecosystem/dlsite.rs`
  - `backend/plugins/src/ecosystem/fanza.rs`
  - `backend/plugins/src/ecosystem/kindle.rs`
  - `backend/plugins/src/ecosystem/bookwalker.rs`
- search history is still in-memory only in `frontend/lib/services/search_history_service.dart`
- Android release config still uses example identity/debug signing in `frontend/android/app/build.gradle.kts`
- batch operations are still a raw comma-separated form flow in `frontend/lib/screens/batch_operations_screen.dart`
- the new `bin/` dispatcher work is now represented as `P10-E1`

---

## 3. Problems in the Current Phase 10 Backlog

The backlog is directionally right, but it still has three weak spots:

1. **Readiness is too implicit.** Some tasks are executable from repo state; others need external HAR/session/signing inputs that are not guaranteed to exist right now.
2. **Execution order is too optimistic.** It does not clearly distinguish dependency roots from downstream tasks.
3. **Verification language is uneven.** Some tasks describe cross-platform outcomes that cannot be fully validated from the current local environment.

---

## 4. Design

### 4.1 Backlog tightening model

Phase 10 should be revised to track two planning states:

- **Ready now** — executable from current repo/environment with no new outside artifacts
- **Blocked/manual-input** — depends on fresh HAR/session captures, signing assets, credentials, or host-specific tooling not guaranteed in-repo

This status belongs in the implementation plan and, where needed, in tightened task wording so later execution does not pretend blocked work is "ready".

### 4.2 Execution strategy

Use a **dependency-first with ready/blocker split** strategy:

1. tighten the backlog wording first
2. execute the highest-leverage ready tasks
3. prepare blocked tasks honestly without claiming impossible completion

### 4.3 First automatic execution wave

Recommended first wave:

1. `P10-E1` — cross-OS `bin/` task tooling
2. `P10-B1` — durable search history
3. `P10-B2` — search history replay UI
4. `P10-C1` — Android release identity/signing hardening
5. `P10-D2` — batch operations UX polish

### 4.4 Initially blocked-or-conditional tasks

These should be treated as blocked unless repo/local evidence proves otherwise during execution:

- `P10-A1` / `P10-A2` / `P10-A3` connector hardening if fresh sanitized HAR/session evidence is missing
- `P10-C2` iOS device-signing/export work if signing/export prerequisites are doc-only and not locally reproducible
- `P10-D1` performance work if meaningful profile measurements cannot be gathered honestly in the current environment

---

## 5. Planning Output

The implementation plan should:

1. record the Phase 10 review findings
2. list concrete backlog edits to make before implementation
3. define execution order and dependencies
4. mark tasks as ready vs blocked
5. choose an automatic execution starting set
6. define verification expectations for each planned execution slice

The plan should live in the session `plan.md` and be reflected into the SQL todo table for tracking.

---

## 6. Acceptance Criteria

This design is satisfied when:

1. `tasks/phase10.md` is reviewed for unclear or missing parts
2. the plan distinguishes ready work from blocked/manual-input work
3. the plan identifies the first automatic execution wave
4. the plan preserves truthful verification language
5. execution is proposed in a dependency-respecting order instead of a flat backlog sweep
