# Architectural decisions

Decisions record adopted choices and their original evidence. Milestone/status
wording is historical; [compatibility](../compatibility-matrix.md) and the
[current guides](../README.md) describe the implemented system. Later decisions
supersede earlier choices where explicitly stated. Stable numbers support code
and evidence references.

## Repository and layering

- [0001 — Standalone interpreter and external application compatibility](0001-repository-scope.md)
- [0002 — Editor and debugger UI owned by tiny-vba](0002-editor-debugger-ui.md)
- [0003 — M0 source, scenario and foreign ownership contracts](0003-m0-source-scenarios-foreign.md)
- [0004 — Verified stack IR and explicit VM continuation contract](0004-bytecode-vm-contract.md)
- [0008 — Non-reused VM instance identity](0008-vm-instance-isolation.md)
- [ADR 0019 — Remaining plan scope and numbering](0019-remaining-plan-scope.md)
- [0035 — Align licensing with the ArtCraft applications](0035-dual-license.md)

## Frontend, compiler and runtime

- [0005 — Bounded source-preserving M1 frontend](0005-m1-frontend.md)
- [0006 — M2 binding, lowering and IR 1 structural target](0006-m2-binding-and-verification.md)
- [0007 — M3 explicit execution and generic host continuations](0007-m3-execution-and-debugger.md)
- [0009 — M4 typed scenario assertion runner and real-division widths](0009-m4-runner-and-numeric-widths.md)
- [0010 — Versioned M4 scalars and trustworthy verification](0010-m4-profile-and-verification-hardening.md)
- [ADR 0011: M4 language extensions](0011-m4-language-extension.md)
- [0017 — Preserve dispatch on imported object roots](0017-imported-root-members.md)
- [ADR 0020 — Explicit host reference snapshots](0020-explicit-host-reference-snapshots.md)
- [ADR 0021 — Nominal host object types](0021-nominal-host-object-types.md)
- [ADR 0022 — Generic default members and enumeration](0022-default-members-and-enumeration.md)
- [ADR 0023 — Project-local instances and form execution](0023-project-classes-and-forms.md)
- [ADR 0025 — Resolve getter aliases before member dispatch](0025-getter-alias-member-dispatch.md)
- [ADR 0026: preserve document-module code names in the IDE](0026-document-module-code-name-model.md)
- [ADR 0028: return ByRef outputs from host-scheduled callbacks](0028-callback-byref-output.md)
- [ADR 0029: explicit delivery to host-object WithEvents subscriptions](0029-host-event-subscriptions.md)
- [ADR 0030: owned VBA metadata and a lossless code view](0030-attribute-metadata-and-code-projection.md)
- [ADR 0031: scalar Variant For controls](0031-variant-for-counters.md)
- [ADR 0034: Intrinsic implementation framework](0034-intrinsic-implementation-framework.md)
- [ADR 0036 — bounded host calls into a waiting project](0036-suspended-host-project-calls.md)
- [ADR 0045 — class enumerators and native form collection iteration](0045-class-and-native-collection-enumerators.md)
- [ADR 0046 — lazy project-class variables](0046-lazy-project-class-variables.md)
- [ADR 0047 — native Forms nominal types and library access](0047-native-forms-nominal-types-and-library-access.md)
- [ADR 0064 — Native runtime TypeName](0064-native-runtime-type-names.md)
- [ADR 0066 — Foreign runtime type names](0066-foreign-runtime-type-names.md)
- [ADR 0067 — VarType and default value inspection](0067-vartype-default-value-inspection.md)
- [ADR 0068 — lazy project-class array elements](0068-lazy-project-class-array-elements.md)
- [ADR 0069 — UserForms without form components](0069-userforms-without-form-components.md)
- [ADR 0089: Restricted WASI callback forwarding](0089-restricted-wasi-callback-forwarding.md)
- [ADR 0101 — M8 conformance and reusable host-call payload space](0101-m8-conformance-and-host-payload.md)

## Host and project integration

- [ADR 0013: M5 native host integration contract](0013-m5-host-contract.md)
- [0014 — Bounded read-only project source decoding](0014-m6-project-decoding.md)
- [0015 — M7 bounded generic callbacks and lifecycle](0015-m7-callbacks-and-lifecycle.md)
- [ADR 0027: host-authorized document callback delivery](0027-document-callback-delivery.md)

## Editor and debugger

- [0016 — Embeddable Classic editor/debugger and executable UI harness](0016-m8-embeddable-editor.md)
- [ADR 0018 — M10 classic tools and debugger policy](0018-m10-classic-tools-and-debugger-policy.md)
- [ADR 0024 — Standard form execution and exchange](0024-standard-form-execution-and-exchange.md)

## Native forms

- [ADR 0038 — Native Office forms, removal of .tvform and compatibility-only scope](0038-native-office-forms-and-compatibility-only.md)
- [ADR 0039 — M7A native control inventory and first worksheet metadata slice](0039-m7a-native-control-metadata.md)
- [ADR 0040 — Native UserForm import and attachment](0040-native-userform-import-and-attachment.md)
- [ADR 0041 — Native worksheet Form Controls](0041-worksheet-form-controls.md)
- [ADR 0042 — UserForm instance identity and window ownership](0042-userform-instance-routing.md)
- [ADR 0043 — native control properties and collection inspection](0043-native-controls-and-project-inspection.md)
- [ADR 0044 — UserForm window, default reference and code lifetime](0044-userform-window-and-code-lifetime.md)
- [ADR 0048 — native container focus and validation](0048-native-container-focus-and-validation.md)
- [ADR 0049 — native TextBox selection and text input](0049-native-textbox-selection-and-input.md)
- [ADR 0050 — native SpinButton and ScrollBar](0050-native-spinbutton-and-scrollbar.md)
- [ADR 0051 — native unbound ListBox](0051-native-unbound-listbox.md)
- [ADR 0052 — native ComboBox text, selection and drop state](0052-native-combobox.md)
- [ADR 0053 — native list array loading and Column access](0053-native-list-array-loading.md)
- [ADR 0054 — native list array readback](0054-native-list-array-readback.md)
- [ADR 0055 — native form metadata and control tips](0055-native-form-metadata-and-control-tips.md)
- [ADR 0056 — native Frame containers](0056-native-frame-containers.md)
- [ADR 0057 — native TabStrip and tab identity](0057-native-tabstrip.md)
- [ADR 0058 — native MultiPage and Page containers](0058-native-multipage.md)
- [ADR 0059 — native Image and immutable picture resources](0059-native-image-pictures.md)
- [ADR 0060 — Native RefEdit and host range selection](0060-native-refedit.md)
- [ADR 0061 — Native Tabs/Pages mutation and stable member identity](0061-native-tab-page-mutation.md)
- [ADR 0062 — Native runtime Controls collections](0062-native-dynamic-controls.md)
- [ADR 0063 — Native ListBox multiple selection](0063-native-listbox-multiple-selection.md)
- [ADR 0065 — Native cell and row bindings](0065-native-cell-and-row-bindings.md)
- [ADR 0070 — Native keyboard events and default input](0070-native-keyboard-events.md)
- [ADR 0071 — Native keyboard focus and default event order](0071-native-keyboard-focus-and-default-order.md)
- [0072 — Native form accelerators](0072-native-form-accelerators.md)
- [0073 — Native typed mouse callbacks and capture](0073-native-mouse-callbacks.md)
- [0074 — Native TextBox pointer selection](0074-native-text-pointer-selection.md)
- [0075 — Native composite, list, numeric and tab pointer defaults](0075-native-composite-pointer-defaults.md)
- [0076 — Native modal input continuations](0076-native-modal-input-continuations.md)
- [ADR 0077: Native UserForm window activation](0077-native-window-activation.md)
- [ADR 0078: Document activation before document input](0078-document-activation-input-barrier.md)
- [ADR 0079: Earlier form input precedes document activation](0079-form-input-before-document-activation.md)
- [ADR 0080: Native form and control colours](0080-native-form-colours.md)
- [ADR 0081: Native form fonts and bounded nominal metadata](0081-native-form-fonts.md)
- [ADR 0082: Native Font property assignment](0082-native-font-assignment.md)
- [ADR 0083: Page appearance belongs to MultiPage](0083-page-appearance-ownership.md)
- [ADR 0084: Native control borders, effects and backgrounds](0084-native-control-appearance.md)
- [ADR 0085: Published Font attribute aliases](0085-native-font-attribute-aliases.md)
- [ADR 0086: Native control text alignment](0086-native-text-alignment.md)
- [ADR 0087: Native caption wrapping and TextBox defaults](0087-native-caption-word-wrap.md)
- [ADR 0088: Native Boolean-control caption wrapping](0088-native-boolean-caption-wrapping.md)
- [ADR 0090: Native CheckBox/OptionButton caption alignment](0090-native-caption-alignment.md)
- [ADR 0091: Native HideSelection and inactive text highlights](0091-native-hide-selection.md)
- [ADR 0092: Native AutoTab through ordinary focus continuations](0092-native-auto-tab.md)
- [ADR 0093: Native SelectionMargin and deferred line selection](0093-native-selection-margin.md)
- [ADR 0094: Native TextBox ScrollBars and bounded viewports](0094-native-textbox-scrollbars.md)
- [ADR 0095: Native TextBox caret viewports without visible bars](0095-native-textbox-hidden-viewports.md)
- [ADR 0096: Native button click focus and input identity](0096-native-button-click-focus-and-input-identity.md)
- [ADR 0097: Common forms completion and P9 compatibility backlog](0097-common-forms-scope-and-p9.md)

## Persistence and release

- [ADR 0012: M4 external oracle acceptance](0012-m4-oracle-acceptance.md)
- [ADR 0032: M6 sign-off and deferred Excel/Word comparison](0032-m6-sign-off-and-reference-comparison-deferral.md)
- [ADR 0033: Intrinsic functions before package acceptance](0033-intrinsics-before-package-acceptance.md)
- [ADR 0037 — M7 original-project preservation and binary writeback contract](0037-m7-package-preservation-and-writeback.md)
- [ADR 0098: Native form encoding and source-only project writeback](0098-native-form-encoding-and-source-only-writeback.md)
- [ADR 0099: Native-only form file workflows](0099-native-only-form-workflows.md)
- [ADR 0100: Common native forms acceptance](0100-common-native-forms-acceptance.md)
- [ADR 0102 — Offline preparation and validation of M8 release evidence](0102-m8-release-evidence-gate.md)
