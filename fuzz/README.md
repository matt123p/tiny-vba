# Robustness testing

Reserve standalone cargo-fuzz targets here after the parser and importer have public APIs.
Keep the fuzz workspace separate from normal builds; pin libfuzzer-sys when introduced.
Targets: lexer/parser, bytecode verifier, project decoder and bounded VM execution.
Store minimized self-authored regression inputs in `tests/regression/`.
No fuzz targets or fuzz results exist at scaffold stage.
