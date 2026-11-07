# ADR 0089: Restricted WASI callback forwarding

Date: 2026-11-07. Status: adopted runner change; full shared/browser/Node 18/22 acceptance passed, host acceptance in progress.

The unchanged default-stack debug form-instance module exits with SIGSEGV under
Node 22.23.4 when linked directly to native WASI imports. The identified designer
case passes alone; the full suite fails after earlier cases. Node 18 passes the
module. The same module passes all 99 cases on Node 22 when each native import is
called through an ordinary JavaScript forwarding function. Disabling WASM tier-up
also crashes, so it is not an accepted remedy. The exact engine/native fault is
not established; do not attribute this to a particular V8 optimization.

The test-only tools/run_wasi.cjs now supplies those forwarding functions. They
return each native result unchanged and let errors/traps propagate. Guest bytes,
memory/stack sizes, bytecode/VM limits, static permissions and host APIs do not
change. WASI still receives args=[], env={}, preopens={} and returnOnExit=true.
The diagnostic module exporting its stack pointer is not used by normal gates.
This is a runner interoperation workaround, not another execution engine.

The existing form-instance suite retains the failing native-form construction
case and its preceding context. Five independent tiny WASM guests additionally
verify nonzero exit status, unsuppressed traps, empty arguments/environment/no
preopened directory, memory growth, native writes, i64 arguments and nonzero WASI
errors. The write fixture uses one empty iovec, supported by both Node versions.
A null-prototype dictionary rejects inherited non-WASI import names; its regression
fails with an ordinary object and passes with the restricted dictionary. They run before the normal native/WASI gate in cargo xtask wasm-run. Node 18/22
receipts and the full authored suites are recorded in implementation progress.

No Office behavior or new M7A.4 completion row is inferred from this harness fix.
Fresh Office observations remain M8.5 NOT_RUN; the full milestone stays open.
