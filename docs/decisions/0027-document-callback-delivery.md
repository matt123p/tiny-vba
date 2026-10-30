# ADR 0027: host-authorized document callback delivery

Status: adopted for the M5 implementation slice, 2026-10-29.

The VM owns verified VBA document-module procedures and their default instances. The embedding owns
workbook/worksheet identity, trust, event ordering and captured host arguments. Import remains inert;
a host may schedule a handler only after authorizing its event. Delivery uses the existing bounded
callback queue and ordinary VM/debugger frames after the active invocation reaches a terminal
boundary. A first document callback lazily allocates its default instance and queues a zero-argument
`Class_Initialize` before the requested handler. Reset and close follow existing session callback
lifecycle rules.

When the IDE receives a host-authorized callback in Design mode, it compiles the authored project
with a temporary, inert standard-module entry stub and constructs an idle verified session. The
private document procedure is still resolved and queued by code name through the callback API; it
does not become a public compiler entry and the project's selected entry procedure is not launched.

The VM default instance remains a VBA class instance for its authored fields and procedures. A
host may bind a validated host object as its document owner. Calls to members declared by the VBA
class stay VM-owned; unresolved members on a document receiver forward through the existing
authorized foreign-call path to that bound owner. This preserves both VBA module state and the
host's actual document object identity. A binding must belong to the current host identity and is
retained in project-state snapshots. Host dispatch remains responsible for stale-object errors.

GridCraft binds workbook/sheet roots by code name and scopes unqualified Application
Range/Cells/Rows/Columns construction to the callback's worksheet owner without changing active
selection. Its host catalogue still determines which document members work. Event ordering,
EnableEvents, mutation sources, trust and undo/recalculation coordination remain application-fork
responsibilities. The declared M5 profile was accepted on 2026-10-30; see the [M5 acceptance map](../m5-acceptance.md).
