# 0105 — Host permissions bounded by the program

Date: 2026-11-21. Status: adopted for issue #14.

`HostBinding::for_program` authorizes every foreign site of a verified program.
It stopped at a fixed cap, `MAX_STATIC_PERMISSIONS` (4,096, ADR 0065). Each
UserForm's generated support code adds about 630 sites, and the IDE builds a
binding whenever a form requests the `VbaFormsGate` root. A project with seven
small forms therefore could not run any macro (`VBA0034 host permission limit`).

The fixed cap is removed. A derived table has one entry per foreign site, and
the verifier already bounds the program's site tables (`limits.instructions` per
procedure and the metadata byte budget in total). `Session::new` refuses a
supplied table with more entries than the program has sites, and it still
refuses any entry that does not name a site of the program. So a table can
never authorize more than the program's own sites.

Which calls are authorized does not change: every host call still needs its own
site in the table. Executed-call, work, object, location and memory limits are
unchanged.

Rejected: raising the cap again, which a project with more forms reaches again;
generating fewer sites per form, which is a larger change to form generation
(related to issue #11) and can follow independently.
