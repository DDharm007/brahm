# BRAHM Evidence-Based Evaluation Rating

**Overall BRAHM Score:** **82.0 / 100**  
**Rating Band:** **Strong** (`75–89`)  
**Evaluation Date:** 2026-11-27  
**Machine-Readable Scorecard:** [BRAHM_RATING.json](./BRAHM_RATING.json) | [BRAHM_SCORECARD.json](./BRAHM_SCORECARD.json)

---

## 1. Score Summary by Weighted Dimension

| # | Dimension | Weight | Score | Percentage | Primary Evidence |
| :-: | :--- | ---: | ---: | ---: | :--- |
| 1 | Architecture Discovery Accuracy | 15.0 | **14.0** | 93.3% | `brahm arch` parsed 26/26 TS files in 25ms (`20/20` routes, `10/10` SQL/migration models, `87` symbols). `-1.0` for mounted subrouter prefix propagation limits. |
| 2 | Decoy Realism & Structural Fit | 15.0 | **9.0** | 60.0% | `11/11` zero-conflict, TypeScript-compatible synthetic overlays. `-6.0` because `.decoy.` / `BRAHM_CANARY_ANCHOR` markers allow white-box semantic identification. |
| 3 | RE Confusion & Effort Amplification | 15.0 | **7.0** | 46.7% | Expanded inspected workspace files `37 -> 48` (`+29.73%`) and wall time `155s -> 211s` (`+36.13%`), but white-box LLM agent kept `100%` genuine route/table recall (`0` false beliefs). |
| 4 | Canary Detection & Telemetry Effectiveness | 15.0 | **14.0** | 93.3% | `11/11` accessed decoy markers fired `CanaryAccessed` events (`23` total SQLite events, `High` severity `R01_REPEATED_CANARY_ACCESS`), sandbox traversal blocked cleanly. `-1.0` for relying on mediated workspace telemetry rather than OS kernel hooks. |
| 5 | Production Safety & Repository Integrity | 15.0 | **15.0** | 100.0% | Zero production repo mutation, stored `BRAHM_BASELINE.json` tamper detection (`E2E-18`), symlink & secret protection, clean ownership-verified rollback (`100%`). |
| 6 | Operational Usability & Automation Integration | 15.0 | **13.0** | 86.7% | Single-command `brahm apply` (`protect`), 20/20 CLI steps + 13/13 security gates, clean JSON/stderr separation, authenticated localhost API. `-2.0` for standalone CLI directory conventions (`ISSUE-004`) and Windows UNC display paths (`ISSUE-009`). |
| 7 | Deterministic Execution & Verification | 10.0 | **10.0** | 100.0% | Pure Rust engine (0 AI/LLM/Python dependencies), `134/134` tests passing, 0 clippy warnings (`-D warnings`). |
| — | **Total** | **100.0** | **82.0** | **82.0%** | **Rating Band: Strong (`75–89`)** |

---

## 2. Rating Scale Definitions

- **90 – 100 (`Exemplary`):** Strong anti-RE confusion even under determined white-box agents, plus complete safety/telemetry/accuracy.
- **75 – 89 (`Strong`):** Production-grade deterministic analysis, isolated inspection deception, 100% canary trapping, strong integrity guarantees, and measurable inspection overhead — with transparent limits against full white-box semantic analysis.
- **60 – 74 (`Moderate`):** Core functionality works with meaningful coverage gaps or stability defects.
- **0 – 59 (`Foundational`):** Early prototype or unverified pipeline.

---

## 3. Justification & Honest Scientific Assessment

BRAHM earns **82.0 / 100 (`Strong`)** strictly from empirical observation rather than optimistic assumptions:
- **Verified Engineering Strengths (`52 / 55` across Discovery, Telemetry, Safety, Determinism):** Stages 01–06 deliver fast, deterministic static intelligence, isolated zero-contamination inspection mirrors, 100% canary capture when workspace accesses flow through BRAHM telemetry, and strict cryptographic baseline/tamper protection verified across `134/134` automated tests and `20/20` CLI E2E + `13/13` failure-path checks.
- **Empirical Deception Ceiling (`16 / 30` combined across Realism & Confusion):** When tested adversarially with an unrestricted white-box inspecting agent reading all 48 workspace files, BRAHM's current `.decoy.` / `BRAHM-CANARY-*` naming strategy forced `+29.73%` extra file reads and `+36.13%` longer inspection time and triggered `11/11` telemetry traps, but did not cause false architectural reconstruction (`20/20` routes and `9/9` tables accurately recovered). Improving AST-level plausible dead-branch injection or selective code redaction/obfuscation policies inside mirrors (without violating developer safety) represents the clearest engineering path above `90+`.
