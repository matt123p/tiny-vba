# ADR 0018 — M10 classic tools and debugger policy

Date: 2026-10-28. Status: adopted.

## Decision and ownership

Extend the reusable M8 IDE in place. `vba-ide` owns source history, scoped literal
search, project operations, metadata and evaluation policy. `vba-ui-egui` renders
commands, classic procedure/split views, docking choices and toolbar customization.
The application shell owns native/browser source transfer and preference storage.
Office objects, document packaging, trust and events remain in the forks.

Source history is shared by editor views, bounded to 256 states and 8 MiB. Edits
invalidate compiled source; Undo does not resurrect a live VM. Navigation and
bookmarks retain stable source identities and remap unchanged anchors on edits.
Rename validates project identity; export-before-remove does not remove on cancel.
Browser imports are explicit UTF-8 `.bas` transfers and never execute code.
Downloads do not prove a successful filesystem save or clear dirty state.

Host reference/constant/help metadata is bounded and declarative. HTTPS help links
are validated. This is not a DLL/COM loader or executable external-type binding;
M15 retains that separate contract. Code aids use scoped source/catalogue metadata.

## Execution policy

The VM exposes read-only inspection at the exact current checkpoint visit,
including session identity. Conditional watches use this path without calls or
properties. Limits are 64 watches, 64 KiB per result and 100,000 predicate
evaluations per run. Exhaustion pauses once with an explicit disabled diagnostic;
Continue can proceed. Context filters apply to the actual frame/procedure.

Set Next Statement validates the current pause, source, frame and statement before
changing the instruction pointer. The supported profile is an ordinary checkpoint
in the current straight-line procedure with an empty operand stack. Branches,
handlers, faults, ancestor frames and stale handles are refused. Relocation grants
no continuation permit and executes nothing; it creates a new pause generation.

Watches and default Immediate evaluation remain read-only. Explicit
`EvaluationPolicy` may permit isolated calls and host effects while paused. This
compiles a bounded child function using owned local snapshots and the same host
identity/capability checks; it does not inject code into paused project state.
Private source functions are accessible in their source module. Child limits are
10,000 instructions, depth 32, 1,024 locations, stack 4,096, 32 host calls and
8 MiB payload. Child waits are cancelled exactly once; callbacks are not run.
Host effects are real. The opt-in is neither default nor persisted as a preference.

## Optional external debugger

`vba-dap` wraps the IDE session rather than duplicating its compile/lifecycle state.
It depends on `vba-ide` and pinned `serde_json` 1.0.154 (MIT OR Apache-2.0).
JSON stays in the optional DAP frontend; native UI/host/debugger calls remain Rust.
The CLI enables stdio transport with `--features dap`. Framing is ASCII
Content-Length plus UTF-8 JSON, with 4 KiB headers and 1 MiB bodies. Input runs
separately from bounded VM pumping so pause/disconnect remain responsive.

Capabilities describe implemented behavior only. Unsupported conditional/hit/log
breakpoints are unverified, not silently converted. Frame/value IDs belong to a
real pause and expire on resume; stack queries do not change the selected frame.
Disconnect, EOF and termination close the session. This is a single-session
adapter, not arbitrary attach, remote debugging or an editor extension package.

## Verification and limits

The [M10 acceptance map](../ui/m10-implementation.md) records authored core,
interaction, process, WASI, real browser and native capture evidence. M8 evidence
remains historical and its regressions still run. Exact Office geometry remains
unverified; this decision neither implements M11 forms/classes nor broadens the
Excel source-compatibility claims in M15–M19.
