# ADR 0032: M6 sign-off and deferred Excel/Word comparison

Adopted 2026-11-01 by explicit owner direction: move comparisons against Excel
and Word to a later milestone, sign off M6 and proceed to the next milestone.

M6 is complete and accepted for its declared integration profile using its
recorded shared native/WASM/WASI, actual fork, UI/native/browser and full
application CI evidence. Existing unsupported features and host boundaries stay
explicit; sign-off does not expand implemented functionality.

The fresh Microsoft Excel/Word comparison requirement formerly associated with
M6.4 moves to M8.5, under expanded conformance and release validation. It remains
NOT_RUN while Office is unavailable. It is deferred, not deleted, passed or
replaced by public documentation, authored expectations or inherited evidence.
The applicable release comparison gate must be satisfied in M8.

M7 is unblocked. Its unchanged-document/package cases exercise the adopted
contracts and record their expectation basis and load/compile/run/save outcomes.
Fresh reference-build comparisons of those cases occur in M8.5. Nothing in this
decision authorizes unreviewed specimen execution, binary writeback without its
separate design, or unverified claims of Microsoft Office compatibility.

The remaining plan starts at M7; completed M5/M6 definitions are retained in the
progress record and the owning acceptance documents. The M6 acceptance report,
README and compatibility status must reflect this sign-off and the deferred gate.
