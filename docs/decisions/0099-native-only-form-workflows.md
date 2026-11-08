# ADR 0099: Native-only form file workflows

Date: 2026-11-08. Status: adopted; M7A.6 COMPLETE; shared and pinned-host gates pass.

Owner direction in ADR 0038 requires removing the custom form format. M7A.5
supplies the replacement native encoders and host package-copy workflows. This
change removes the `.tvform` codec/module, public `import_form`/`export_form` APIs,
source-import branches, Save/Export/remove-export fallbacks, browser/host file
filters and the shipped custom example. There is no deprecated importer, hidden
compatibility path, converter or replacement custom format.

`vba-ide` still owns the shared designer/runtime. Insert UserForm now attaches an
inert, empty native `.frm` template with ordinary VB_Name metadata, the existing
300x200 point defaults and code page 1252, matching the current shell import
profile. Source and native resources remain separate; new forms are modified until
saved. Unsupported source characters refuse at export instead of being replaced.
No locale/VM/host authority or limit changes are introduced. Legacy-only
ordinary fields that collide with native built-in properties (BackColor/ForeColor)
now receive the same pre-execution refusal as imported native forms; this is not
an alternative class of custom UserForm.

All form Save/Export actions and export-before-remove route through the existing
native snapshot request. Plain `.bas`/`.cls` source transfer remains separate;
`.frm`/`.frx` use the native bundle path. Current component names also appear in
standalone native export headers, file names and VB_Name attributes. The format
encoder has an explicit named-export API for this existing rename workflow;
fixed embedded project component identity continues to refuse rename/insertion
at the project writeback boundary. Imported originals are never rebased/mutated.

The CustomerForm demonstration is rebuilt from original Rust designer operations
and authored VBA into native files. Native imports still validate before mutation;
Save validates source/design/project snapshots and acknowledges only successful
IO. Browser downloads and macro-enabled copy export do not clear original editor
dirty state. Runtime callbacks, Reset, debugger waits and host trust remain intact.

Useful codec-era tests now use native file fixtures; source, layout/tab order,
atomic malformed-resource refusal, stale receipts, actual host callbacks and
pause/resume expectations are retained. Native grouping export still explicitly
refuses the unsupported resource representation. A repository gate prevents the
retired codec, product references and custom example extension from returning.
Historical ADR/M2 acceptance records remain evidence of their original profiles;
their format choice/support is superseded by ADRs 0038, 0098 and this decision.
M7A.7 final combined acceptance and M8.5 fresh Office comparisons remain separate;
no fresh Office observation or additional P9 behavior is claimed here.
