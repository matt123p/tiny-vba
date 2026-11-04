# ADR 0037 — M7 original-project preservation and binary writeback contract

Date: 2026-11-04. Status: adopted for the M7 implementation requested by the owner.
Owners: fork package/storage/trust layers; tiny-vba remains a read-only source decoder.

## Decision and reviewed boundary

M7 accepts unchanged, independently authored source in macro-enabled packages for
the declared M1–M6/M6A profile. A package is decoded inertly, its source is compiled
unchanged, and execution requires explicit document permission. Document effects
may change while the original VBA project remains byte-identical. ZIP bytes,
relationship serialization and ordinary document XML need not remain identical.
This is original-project preservation, not whole-package byte identity.

Three operations have distinct results and acknowledgments:

1. Saving a document as `.xlsm`/`.docm` retains the original VBA project and the
   supported related payload/relationship graph. It does not acknowledge IDE edits.
2. Plain-source and `.tvform` exports retain canonical authored source, including
   attached hidden metadata; acknowledgments match source/design revision and
   project instance. They do not update a binary project or its signature.
3. Edited binary VBA project writeback remains **UNSUPPORTED**. No public tiny-vba
   encoder, P-code executor, protection bypass or binary add-in loader is added.
   GridCraft refuses replacing a captured project's blob while retaining its old
   related parts. A new blob without preservation metadata is caller-supplied data,
   not evidence of edited-source encoding or signature validity.

M7.3 asks for a design before implementation, not an encoder in this milestone.
The following is the contract any future edited-source encoder must satisfy.

## Future edited-source writeback

- A fork-owned explicit save operation captures document identity, raw-source and
  design revisions, original project hash, target format and signature policy.
  A source export or ordinary document save cannot select this operation implicitly.
- Compile the exact snapshot and validate encoding, component/code-name identities,
  module/member attributes, registered references and the representable form profile.
  Refuse unsupported resources, protected projects and ambiguous/cache-only projects.
  The existing importer's `NotChecked` cache/source consistency is never a pass.
- A separately reviewed Rust encoder must produce bounded MS-OVBA/CFB output from
  source. It must define directory records, stream offsets, encoding and cache
  removal/regeneration together; retaining stale P-code/caches is forbidden.
  This ADR defines no shortcut for those unimplemented format responsibilities.
- Remove every VBA signature format and associated relationships/content types
  when any signed project content changes. Never reuse original signatures on an
  edited project. Re-signing requires a separate explicit signing service/policy;
  no key discovery or ambient signing authority is permitted.
- Serialize into temporary bytes, reopen with the decoder, compare exact extracted
  source/metadata/reference identities, validate every internal relationship, and
  verify typed effects in an independent reviewed acceptance workflow. Only then
  perform the host's atomic file replacement. Recheck all captured identities and
  revisions before committing or acknowledging. Failure leaves the original file
  and IDE modified state intact; never partially update a project.

## Signatures and trust

Opaque VBA signature bytes are retained only beside the original project. Their
presence is **not** cryptographic verification, publisher trust or permission to run.
M7 sentinels test transport only; actual signature validity is NOT_RUN without a
licensed signed specimen and a verifier. A document edit may invalidate a package
signature covering document XML even when the VBA project is unchanged. The
writers do not claim package signature preservation and do not synthesize validity.
Macro-free export drops the VBA graph. Document permission is scoped to its open
instance and never inferred from preserved signature bytes or inherited on reopen.

External relationships are retained as text where the owning package profile
supports them, never followed. Missing/oversized/rewritten related graphs produce
explicit warnings/refusals, not a preservation PASS. GridCraft uses its existing
bounded graph traversal and writer collision handling. WordCraft's retained direct
`word/` VBA data/signature profile remains narrower; unsupported graph shapes are
not promoted to recursive preservation support.

## Contract review

Reviewed against ADR 0014 (inert decoder), ADR 0030 (canonical raw source),
ADR 0024 (independent forms/revision acknowledgments), the two fork package readers
and writers, and M7.1–M7.4. The design preserves the 19-crate shared DAG and native
host calls. Existing compiler/runtime semantics are unchanged. Package and signature
policy stay above the interpreter; source attachment grants no host authority.

Acceptance records must separate load/compile/run/save, exact raw source from
original binary bytes, authored expectations from inherited/fresh Office evidence,
and supported preservation from unsupported edited writeback. Fresh Office build
comparisons remain M8.5. Broader native forms/controls stay in the deferred plan.

Format references used for this review: Microsoft's [OPC package relationship table](https://learn.microsoft.com/en-us/dotnet/api/system.io.packaging.package.createrelationship)
identifies the distinct digital-signature origin relationship; its [VBA project signature specification](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-xlsb/8fbb98c8-bf03-429e-b6e8-ae024539b5b9)
identifies the VBA signature relationship. Microsoft's [macro signing guidance](https://support.microsoft.com/en-au/office/vba/digitally-sign-your-vba-macro-project)
documents signature removal when signed code changes. These are format/policy
references, not newly collected Office observations. Package-wide signature roots
are excluded by GridCraft and explicitly dropped/logged by WordCraft during import
into its rewrite model; VBA signature payload transport stays separate.
