# 0005 — Bounded source-preserving M1 frontend

Date: 2026-10-26. Status: adopted and implemented for M1's reviewed subset.

`vba-syntax` owns a pure-Rust lexer, a recursive-descent declaration/statement parser and Pratt
expressions. Reuse diagnostics' SourceFile/Span/StatementId; keep original text and every token,
including trivia, in the module. ASCII case-insensitive keys never replace display spelling.
Lexical/parser diagnostics prevent syntax success even when recovery produces a partial AST.
Document supported forms and hard limits in [frontend](../frontend.md). Expand from independent
fixtures, not another interpreter's implementation. No external Rust dependencies are introduced.

Application syntax remains unresolved until binding. Explicit/implicit calls, parenthesized arguments,
Let/Set and named/omitted/optional forms retain their distinct syntax even when IR 1 rejects them.
There is no inference of default members, array semantics, coercion or host capabilities in syntax.
Statement IDs follow source traversal rather than physical lines. Control headers and terminators
have separate sites; compiler checkpoint roles/SiteIds remain M2 work under ADR 0004.

SourceFile.position now handles lone CR as well as CRLF/LF, matching the frontend's newline rules.
This refines ADR 0003's deferred lone-CR behavior; UTF-8 spans and Unicode scalar columns remain the
same contract. UTF-16 editor coordinates are still future work.

Add the development CLI's direct syntax dependency to the reviewed DAG to expose `parse <path>`.
No compiler/IR/VM dependency reversals or new crates are needed. Libraries remain headless and WASM
portable. CLI IO is bounded, and parsing never executes an entrypoint, host operation or downloaded
script. The optional corpus example reads five reviewed MIT examples as syntax data only; original
inventory execution statuses stay NOT_RUN. Review and results are in the M1 corpus report.

M1 completion is this documented subset and its positive/negative/source/budget tests, runnable CLI,
external syntax evaluation and native/WASM compilation gates. It is not complete VBA grammar support.
M2 next implements scopes, Option Explicit, direct signature/name binding and deterministic IR 1
lowering/verifier/disassembly. Execution remains M3.
