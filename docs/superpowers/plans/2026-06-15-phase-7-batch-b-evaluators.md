# Phase 7 Batch B Evaluators Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement P7-T03~P7-T05 formal backend evaluators (`stage_evaluator`, `evidence_evaluator`, `understanding_evaluator`, `view_evaluator`, `qa_evaluator`) and wire them into `QualityReporter`, replacing the Batch A baseline reporter checks while preserving all safety and scope constraints.

**Architecture:** Split the monolithic `reporter.rs` baseline checks into focused evaluator modules, each responsible for one dimension. A shared `issue_builder` module provides deterministic `QualityIssue` construction helpers. `QualityReporter` becomes a coordinator: build input sets, call evaluators, assign deterministic IDs, attach dimension refs, build `QualityRunSummary`, and compute `QualityAcceptanceStatus`. All evaluators remain read-only, deterministic, and free of audit verdicts.

**Tech Stack:** Rust (Tauri v2 backend), existing `evidence`/`understanding`/`views`/`trace`/`models` crates.

---

## File Structure

| File | Responsibility |
|------|----------------|
| `src-tauri/src/quality/issue_builder.rs` | Shared `make_issue`, `make_guardrail`, `trace_ref_ok`, `is_noisy`, `is_label_sane`, `sanitize_scope`. |
| `src-tauri/src/quality/stage_evaluator.rs` | `StageEvaluator` / `StageEvaluatorInput` → `StageEvaluationTarget` + `stage_identification_mismatch` issues. |
| `src-tauri/src/quality/evidence_evaluator.rs` | `EvidenceEvaluator` / `EvidenceEvaluatorInput` → `EvidenceQualityReport` + `missing_evidence`/`noisy_evidence`/`wrong_source_kind` issues. |
| `src-tauri/src/quality/understanding_evaluator.rs` | `UnderstandingEvaluator` / `UnderstandingEvaluatorInput` → `UnderstandingQualityReport` + `unsupported_claim`/`hallucinated_claim_blocked`/`weak_summary` issues. |
| `src-tauri/src/quality/view_evaluator.rs` | `ViewEvaluator` / `ViewEvaluatorInput` → `ViewQualityReport` + `empty_or_unhelpful_view` issues. |
| `src-tauri/src/quality/qa_evaluator.rs` | `QaEvaluator` / `QaEvaluatorInput` → `QaQualityReport` + `qa_answer_without_valid_citation`/`qa_unanswered_when_evidence_exists` issues. |
| `src-tauri/src/quality/reporter.rs` | Coordinator: call evaluators, assign IDs, dimension refs, summary, acceptance. |
| `src-tauri/src/quality/mod.rs` | Declare new modules and re-export public evaluator types. |

---

## Task 1: Shared Issue Builder

**Files:**
- Create: `src-tauri/src/quality/issue_builder.rs`
- Modify: `src-tauri/src/quality/mod.rs`

- [ ] **Step 1: Create `issue_builder.rs`**

Move existing helpers from `reporter.rs` into shared public helpers:

```rust
use std::collections::HashSet;
use crate::evidence::models::{EvidenceItem, LineRange};
use crate::models::enums::{Language, SourceKind};
use crate::quality::models::{
    ArtifactKind, DetectionMethod, IssueStatus, QualityIssue, QualityIssueKind,
    QualityIssuePolarity, QualitySeverity,
};

#[allow(clippy::too_many_arguments)]
pub fn make_issue(...)

pub fn make_guardrail(...)

pub fn trace_ref_ok(...)

pub fn is_noisy(summary: &str) -> bool

pub fn is_label_sane(item: &EvidenceItem) -> bool

pub fn sanitize_scope(sample_id: &str) -> String
```

- [ ] **Step 2: Expose from `mod.rs`**

Add `pub mod issue_builder;` and re-export helpers if useful.

---

## Task 2: Stage Evaluator

**Files:**
- Create: `src-tauri/src/quality/stage_evaluator.rs`

- [ ] **Step 1: Define input and evaluator**

```rust
pub struct StageEvaluatorInput<'a> { ... }
pub struct StageEvaluator;
impl StageEvaluator {
    pub fn evaluate(input: &StageEvaluatorInput<'_>) -> (StageEvaluationTarget, Vec<QualityIssue>)
}
```

- [ ] **Step 2: Implement status comparison**

If `expected_status` is `Some(non-empty)` and differs from `recognized_status`, emit `stage_identification_mismatch` with `artifact_kind=Stage`, severity `High`.

- [ ] **Step 3: Add tests**

`status_match_no_issue`, `status_mismatch_emits_stage_identification_mismatch`, `missing_expected_status_no_issue`.

---

## Task 3: Evidence Evaluator

**Files:**
- Create: `src-tauri/src/quality/evidence_evaluator.rs`

- [ ] **Step 1: Define input**

```rust
pub struct EvidenceEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub collection: &'a EvidenceCollection,
    pub expected_source_paths: Option<&'a [String]>,
}
pub struct EvidenceEvaluator;
```

- [ ] **Step 2: Implement checks**

- Empty collection → `missing_evidence`.
- `file_coverage_ratio`: use `expected_source_paths` if provided; otherwise fallback (`0.0` if empty, `1.0` otherwise) and only populate `uncovered_files` when expected paths are given.
- `line_range_accuracy`: count items with `start >= 1 && end >= start`.
- `label_sanity_ratio`: use `is_label_sane`.
- `noisy_evidence` / `wrong_source_kind` issue generation with `evidence_id`/`source_path`/`line_range`.

- [ ] **Step 3: Add tests**

`empty_evidence_emits_missing_evidence`, `invalid_line_range_affects_accuracy`, `wrong_source_kind_emits_issue`, `noisy_evidence_emits_issue`, `expected_source_paths_uncovered_file_emits_missing_evidence`, `no_expected_files_uses_fallback_without_fake_uncovered_files`.

---

## Task 4: Understanding Evaluator

**Files:**
- Create: `src-tauri/src/quality/understanding_evaluator.rs`

- [ ] **Step 1: Define input**

```rust
pub struct UnderstandingEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub understanding: &'a ImplementationUnderstanding,
    pub evidence_id_set: &'a HashSet<String>,
}
pub struct UnderstandingEvaluator;
```

- [ ] **Step 2: Implement checks**

- Claim `evidence_refs` existence check; missing refs without `evidence_gap` → `unsupported_claim`.
- `unknown` claim + `evidence_gap` + no refs → `hallucinated_claim_blocked` positive guardrail.
- `weak_summary` when short empty or detailed < 10 chars.
- Compute ratios deterministically.

- [ ] **Step 3: Add tests**

`unsupported_claim_without_gap_emits_issue`, `unknown_claim_with_gap_emits_positive_guardrail`, `unknown_with_fake_evidence_ref_emits_problem`, `weak_summary_emits_issue`, `claim_existence_ratio_uses_existing_evidence_ids`.

---

## Task 5: View Evaluator

**Files:**
- Create: `src-tauri/src/quality/view_evaluator.rs`

- [ ] **Step 1: Define input**

```rust
pub struct ViewEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub view: &'a ViewGraph,
    pub evidence_id_set: &'a HashSet<String>,
    pub claim_id_set: &'a HashSet<String>,
}
pub struct ViewEvaluator;
```

- [ ] **Step 2: Implement checks**

- Empty graph → `empty_or_unhelpful_view`, ratio `0.0`.
- Per node/edge `trace_refs` presence and resolvability; missing/unresolvable → issue.
- `trace_resolvable_ratio = resolvable_artifacts / total_artifacts`.
- `isolated_node_count`, `suspected_misconnection_count` placeholder with TODO.

- [ ] **Step 3: Add tests**

`empty_view_ratio_zero`, `node_without_trace_emits_issue`, `edge_without_trace_emits_issue`, `invalid_trace_ref_lowers_ratio`, `valid_node_edge_trace_ratio_one`, `isolated_node_count_detected`.

---

## Task 6: Q&A Evaluator

**Files:**
- Create: `src-tauri/src/quality/qa_evaluator.rs`

- [ ] **Step 1: Define input**

```rust
pub struct QaEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub answer: &'a GroundedAnswer,
    pub evidence_id_set: &'a HashSet<String>,
    pub claim_id_set: &'a HashSet<String>,
    pub question_set: Option<&'a QaEvaluationQuestionSet>,
}
pub struct QaEvaluator;
```

- [ ] **Step 2: Implement checks**

- Citation existence check for `evidence_id` and `claim_id`.
- With `question_set`: match first question for stage; `answerable` + degraded/unknown/no valid citations → `qa_unanswered_when_evidence_exists`; `not_answerable` + unknown + no fabricated citations → no problem.
- Without `question_set`: documented fallback using confidence proxy.

- [ ] **Step 3: Add tests**

`invalid_evidence_citation_emits_issue`, `invalid_claim_citation_emits_issue`, `answerable_question_unknown_emits_unanswered_issue`, `not_answerable_unknown_without_citation_no_problem`, `answerable_hit_ratio_with_question_set`, `fallback_without_question_set_is_documented`.

---

## Task 7: Reporter Integration

**Files:**
- Modify: `src-tauri/src/quality/reporter.rs`

- [ ] **Step 1: Remove baseline `evaluate_*` functions**

Delete `evaluate_evidence`, `evaluate_understanding`, `evaluate_view`, `evaluate_qa`, `trace_ref_ok`, `is_noisy`, `is_label_sane`, `sanitize_scope`, `make_issue`, `make_guardrail` from `reporter.rs`.

- [ ] **Step 2: Import evaluators and helpers**

```rust
use super::issue_builder::{make_guardrail, make_issue, sanitize_scope};
use super::stage_evaluator::{StageEvaluator, StageEvaluatorInput};
use super::evidence_evaluator::{EvidenceEvaluator, EvidenceEvaluatorInput};
use super::understanding_evaluator::{UnderstandingEvaluator, UnderstandingEvaluatorInput};
use super::view_evaluator::{ViewEvaluator, ViewEvaluatorInput};
use super::qa_evaluator::{QaEvaluator, QaEvaluatorInput};
```

- [ ] **Step 3: Wire evaluators in `evaluate_stage`**

Build `evidence_id_set` and `claim_id_set`; call each evaluator; collect reports and issues.

- [ ] **Step 4: Preserve deterministic IDs and summary**

`assign_issue_ids` and `build_run_summary` remain; continue using `as_str()` for snake_case keys.

- [ ] **Step 5: Add integration tests**

`reporter_calls_formal_evaluators`, `deterministic_output_stable_after_evaluator_split`, `run_summary_still_uses_snake_case_keys`, `positive_guardrail_not_counted_as_problem`.

---

## Task 8: Module Registration

**Files:**
- Modify: `src-tauri/src/quality/mod.rs`

- [ ] **Step 1: Declare modules**

```rust
pub mod issue_builder;
pub mod stage_evaluator;
pub mod evidence_evaluator;
pub mod understanding_evaluator;
pub mod view_evaluator;
pub mod qa_evaluator;
```

- [ ] **Step 2: Re-export evaluator types**

Add public evaluator input structs and evaluator structs to `pub use` block.

---

## Task 9: Verification

- [ ] **Step 1: Run focused tests**

```bash
cd src-tauri && cargo test --lib quality::
```

Expected: all new evaluator + reporter tests pass.

- [ ] **Step 2: Run full Rust test suite**

```bash
cd src-tauri && cargo test --lib
```

Expected: all tests pass.

- [ ] **Step 3: Run cargo check**

```bash
cd src-tauri && cargo check
```

Expected: zero warnings.

- [ ] **Step 4: Run frontend build**

```bash
npm run build
```

Expected: build succeeds.

- [ ] **Step 5: Run rg boundary checks**

Execute the five rg commands from the task spec and document results.

---

## Task 10: Commit and Push

- [ ] **Step 1: Stage files**

```bash
git add src-tauri/src/quality/
git add docs/planning/phase-7-implementation-plan.md  # if boundary note updated
```

- [ ] **Step 2: Commit**

```bash
git commit -m "feat(quality): implement phase 7 batch b evaluators"
```

- [ ] **Step 3: Push**

```bash
git push github main
```

---

## Spec Coverage Self-Review

| Spec Requirement | Task |
|------------------|------|
| Stage evaluator (status mismatch, snake_case) | Task 2 |
| Evidence evaluator (coverage, line_range, source_kind, noisy, uncovered_files) | Task 3 |
| Understanding evaluator (claim existence, unsupported, guardrail, weak_summary, ratios) | Task 4 |
| View evaluator (empty graph, trace_refs, isolated nodes, ratio) | Task 5 |
| Q&A evaluator (citation validity, answerable/unanswerable, question_set, fallback) | Task 6 |
| Reporter integration (deterministic IDs, snake_case summary, stable output) | Task 7 |
| No Tauri command / no UI / no LLM / no FS write / no audit verdicts | All tasks (verified by rg in Task 9) |

---

*Plan created with the `writing-plans` skill.*
