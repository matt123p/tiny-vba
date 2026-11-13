---
name: tiny-vba-editor-ui
description: Build or review the tiny-vba VBA editor and graphical debugger, including egui panels, standalone shell, Gridcraft embedding, design tokens, accessibility and interaction. Use for visual or IDE state/action changes.
---

# tiny-vba editor ui

Read [UI contract](../../../docs/editor-debugger-ui.md),
[embedding](../../../docs/ui/embedding.md) and [forms](../../../docs/forms.md).

Align the reviewed egui/eframe family and consuming-root Cargo patches. Keep IDE
state/actions toolkit independent; render into caller-owned Ui with independent
widget IDs, injected theme/fonts/scaling and action routing. Do not replace global
styles/fonts. Panels issue commands rather than mutating the VM/document directly;
file pickers/IO and prompts belong to shells/hosts.

Preserve exact source/line endings and hidden metadata through the view projection.
Save acknowledgements are revision-checked. Browser download requests do not prove
successful file commits. Form design/source/resources survive VM Reset; native
runtime widgets inspect canonical fields without a second writable state model.
Executable input follows authorization and callback/default continuation barriers.

Use accessible labels and keyboard access, with glyphs as well as color for state.
Verify 4.5:1 text and 3:1 functional glyph contrast. Use original/licensed assets.
After visual changes inspect light/dark, narrow/scaled/focus and injected-host-theme
renders. Exercise actual edit/run/break/step/inspect/resume and relevant native/
browser workflows; preserve headless builds. Placeholder screenshots are not
execution evidence. See [testing](../../../docs/testing.md) for gates.
