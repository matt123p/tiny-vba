# 0002 — Editor and debugger UI owned by tiny-vba

Date: 2026-10-26. Status: adopted on explicit user direction.
Amends [ADR 0001](0001-repository-scope.md) and takes precedence over supplied design snapshots.

## Decision

This repository owns the VBA source editor, graphical debugger, reusable UI components and a
standalone IDE application. These are required product deliverables. Gridcraft and Wordcraft
own their document UI, Office compatibility layers and integration with this editor/debugger.

Use egui for embeddable panels and eframe for the standalone shell, aligned with Gridcraft's
reviewed dependency revisions and design conventions. At adoption, Gridcraft declares egui/eframe
0.37 and patches the egui family to revision 6d518fc799681571963842f95b55a6b68431b15a.
This is a local compatibility reference, not a dependency added by this decision. Recheck the
host manifests before implementation and coordinate pin updates across consuming workspaces.

The reusable UI must use the embedding application's existing egui context and accept its theme,
fonts, icons and action routing. It must also work independently with an original default theme
and assets. Neither application fork is a build dependency of tiny-vba.

## Boundaries and delivery

- Compiler, VM and debugger controller remain toolkit independent and usable headlessly.
- Planned vba-ide owns editor/project/debug-session state and actions; vba-ui-egui renders it.
- Planned vba-ide-app provides the standalone native/web shell and platform IO integration.
- Native UI controls call Rust debugger APIs directly. DAP is optional for external clients.
- Host adapters supply Office objects, document lifecycle, trust decisions and document edits.
  Saving source files does not imply writing a VBA binary project or macro-enabled Office package.

The first executable UI slice follows the M3 VM/debugger proof. M8 requires a usable visual editor
and debugger in this repository. Gridcraft embedding is verified in the fork without duplicating
these panels. See [the UI contract](../editor-debugger-ui.md) for acceptance criteria.

## Consequences

UI maintenance, original assets, font handling, rendering checks and accessibility checks become
part of tiny-vba's scope. GUI packages stay optional for headless consumers. Introduce real Cargo
packages and graph entries with implementation, not documentation-only placeholders. Shared design
conventions may be followed without copying Gridcraft source or introducing sibling-path dependencies.
