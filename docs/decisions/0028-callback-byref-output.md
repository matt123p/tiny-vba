# ADR 0028: return ByRef outputs from host-scheduled callbacks

Status: adopted for the M6 host-integration slice, 2026-10-30.

Host-scheduled callbacks receive captured argument values, but Word and other event hosts need to
observe `ByRef` parameters after a handler returns. A host cannot safely retain VM locations or
borrow a mutable host object across an asynchronous callback. The VM therefore snapshots each
`ByRef` parameter's final value when its scheduled callback completes normally. The result records
the original zero-based parameter index and is consumed once by the embedding.

Snapshots are FIFO for completed callbacks and share the session's payload/work budget and callback
limit. Callbacks without `ByRef` parameters retain no output snapshot. A failed or cancelled callback
produces no result for that invocation. Reset clears pending snapshots. Embeddings can clear any
unconsumed results before delivering an independent host event.

This is copy-back at the callback boundary, not an alias into host memory: event handlers get normal
VBA `ByRef` behavior within the procedure, and the host applies the returned value when the handler
has completed. A host must still define event trust, ordering, cancellation and side-effect policy.
