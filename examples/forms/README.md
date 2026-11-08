# Native CustomerForm example

`CustomerForm.frm` and its explicitly named `.frx` companion are original,
Rust-generated native Office form fixtures, licensed MIT OR Apache-2.0 with this
repository. They contain no Office-created data or vendor artwork. The source
handler body is in `CustomerForm.bas` solely as input to the fixture builder.

Open `CustomerForm.frm` in the native app; it reads the named companion. In the
browser choose both `.frm` and `.frx` together. Run the form and click Submit:
the authored callback pauses at Stop, then Continue prints 7 and hides the form.
Text changes update the Name label. Import/export never runs the handler.

Rebuild with `cargo run -p vba-ide --example customer_form -- <output-directory>`.
The builder uses the same native new-form/designer/export APIs as the product,
not a custom archive or converter. Companion basenames contain the encoder's
content discriminator; the `.frm` names the exact companion to distribute.
