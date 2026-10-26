# M1 external grammar evaluation

Run 2026-10-26 on Linux with the pinned Rust toolchain. Syntax only: no macro, upstream script,
generated parser or other interpreter was executed or linked. No Microsoft oracle observations.

Reviewed selection: the five directly authored examples below from harumiWeb/tree-sitter-vba at
`83c5c3c3c7210468fb6745b585b5fb6532aa2376`. The checkout's top-level MIT licence names
Copyright (c) 2026 harumiWeb; these examples have no separate third-party notice and are outside
`examples/third_party` and `examples/realworld`. They remain in the original submodule with its
licence; no source is redistributed in our fixtures. Pins/licence hashes remain in
[the source lock](../references/sources.lock.json), and candidate hashes in the existing inventory.

Only parser ingestion is approved for this selection. All five require no runtime capabilities for
that operation. `basic.bas` contains Debug.Print and `class.cls` property procedures; they have not
been classified or approved for execution. Profile: M1 syntax subset, not an Office runtime profile.
Expected results below follow manual source review: Select Case and Property are outside the subset;
the other files intentionally omit a call argument/closing parenthesis, End If, or a string quote.

```sh
cargo run -p vba-syntax --example grammar_corpus -- tests/corpora/tree-sitter-vba
```

This explicit allowlist requires the pinned checkout. Normal `cargo test --workspace` does not.
The runner reads only these five bounded source files and fails if their result categories differ
from the reviewed expectations. Run corpus integrity checks first when reproducing against a checkout.

| Specimen under examples/ | Actual / expected | First diagnostic |
|---|---|---|
| basic.bas | UNSUPPORTED | 18:5 Select Case |
| class.cls | UNSUPPORTED | 10:8 Property Get |
| broken/incomplete-call.bas | REJECTED | 4:17 expected expression |
| broken/incomplete-if.bas | REJECTED | 6:5 expected If before End Sub |
| broken/malformed-string.bas | REJECTED | 4:15 unterminated string literal |

Five of five classifications matched. Zero of the two nominally valid upstream modules parsed
successfully; both require unsupported constructs. All three intentionally broken inputs were
rejected. Recovery may emit additional diagnostics after unsupported compound blocks. This is a
small grammar evaluation, not a language compatibility pass rate. Corpus inventory entries retain
NOT_RUN for execution; no global or per-scenario execution result changes.

SHA-256, in the same order:

```text
d7d6f8896e93586b6cc72693dace3cb11ceb9d1e83b0e8b6722fa350fb35355e basic.bas
02437c4ba8e2310cbab53bd21e0c8e394f490322c68afed880f64e458044ec96 class.cls
30dcc1063e39d01bd986cfd9e0e9049c01555368b306d13c7df10238cad7922c broken/incomplete-call.bas
64c4aac84206a01c117a18d86b148f57ba8b017be306e5a5b8e6d93a9fe67ec4 broken/incomplete-if.bas
47575b254b4774996401126434e335edc6e0e0f4b7e8fd7648a02d17935487ef broken/malformed-string.bas
```
