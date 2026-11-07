# ADR 0082: Native Font property assignment

Date: 2026-11-07. Status: implemented; acceptance in progress. M7A.4 remains open.

The IDE provider emits a typed `Property Set Font(ByVal value As MSForms.NewFont)`
for the UserForm and each existing Font-capable native control class. It uses
ordinary `Set VbaFont=value`; it introduces no VM/foreign special case, mutable
registry, font acquisition, additional interface claim or increased budget.
Root setters use the same lazy-load guard as root getters. Named/default form
access and retained control references keep their existing lifetime behavior.
Image, ScrollBar and SpinButton continue to have no Font member.

Explicit assignment is distinct from the independently copied container defaults
in ADR 0081. Assigned references share subsequent attributes; aliases of the
previous Font remain valid. New controls still copy their container's current
attributes into independent objects. Normal nominal argument checking rejects
unrelated objects with error 13 before mutation. Stop/Continue resumes once and
Reset retains original source and native resources. Inert inspection/rendering
reads the newly assigned canonical object without invoking getters or loading.

Microsoft's Forms interop reference exposes writable NewFont properties on both
UserForm and TextBox. Reference-sharing expectations here derive from that
signature and VBA's documented Set/reference contract; these are independently
authored expectations, not fresh Office observations. Font-specific Nothing,
external font realization, other assignment modes, ambient/toolbox defaults and
platform-specific behavior remain unaccepted comparison cases. No rejection or
partial profile closes the full P01 or M7A.4 checklist. Edited native font saving
remains M7A.5, legacy-format removal M7A.6 and fresh Office comparisons M8.5 NOT_RUN.

Ten model tests (five new cases, including 16 per-family subcases) cover typed
aliases, replacement, old aliases, root lazy reload, dynamic copied defaults,
Stop/Continue/Reset and the existing attribute/import profile. Four GPU cases
include pixel equality between direct attributes and an assigned Font. Native
browser and affected host acceptance are recorded in implementation progress.

References:

- [Forms UserForm Font property](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.userformclass.font?view=office-pia)
- [Forms TextBox Font property](https://learn.microsoft.com/en-us/dotnet/api/microsoft.vbe.interop.forms.textboxclass.font?view=office-pia)
- [VBA Set statement](https://learn.microsoft.com/en-us/office/vba/language/reference/user-interface-help/set-statement)
- [ADR 0081](0081-native-form-fonts.md)
