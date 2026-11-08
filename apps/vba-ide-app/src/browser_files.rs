//! Explicit browser upload/download through public web APIs; no ambient file IO.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
type ImportResult = Result<Vec<vba_ide::NativeFormResource>, String>;
type ImportInput = (
    web_sys::HtmlInputElement,
    Closure<dyn FnMut(web_sys::Event)>,
);
#[derive(Default)]
pub struct BrowserFiles {
    result: Rc<RefCell<Option<ImportResult>>>,
    input: Option<ImportInput>,
    urls: Vec<String>,
}
impl BrowserFiles {
    pub fn take_import(&mut self) -> Option<ImportResult> {
        self.result.borrow_mut().take()
    }
    pub fn choose(&mut self, ctx: egui_context::Context) -> Result<(), String> {
        self.choose_inner(ctx)
            .map_err(|e| format!("Browser upload failed: {e:?}"))
    }
    fn choose_inner(&mut self, ctx: egui_context::Context) -> Result<(), JsValue> {
        if let Some((input, _)) = self.input.take() {
            input.set_onchange(None);
            input.remove();
        }
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let input = document
            .create_element("input")?
            .dyn_into::<web_sys::HtmlInputElement>()?;
        input.set_type("file");
        input.set_accept(".bas,.cls,.frm,.frx,text/plain");
        input.set_multiple(true);
        input.set_attribute("hidden", "")?;
        document
            .body()
            .ok_or_else(|| JsValue::from_str("no body"))?
            .append_child(&input)?;
        let selected = input.clone();
        let result = self.result.clone();
        let callback = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let Some(files) = selected.files() else {
                return;
            };
            let result = result.clone();
            let ctx = ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let imported = async {
                    if files.length() == 0 || files.length() > 2 {
                        return Err("Choose one source file and its optional .frx companion".into());
                    }
                    let mut out = Vec::new();
                    let mut total = 0usize;
                    for index in 0..files.length() {
                        let file = files.get(index).ok_or("Missing selected file")?;
                        let name = file.name();
                        if ![".bas", ".cls", ".frm", ".frx"]
                            .iter()
                            .any(|e| name.to_ascii_lowercase().ends_with(e))
                            || file.size() > (8 * 1024 * 1024) as f64
                        {
                            return Err("Unsupported source file or size".into());
                        }
                        let buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
                            .await
                            .map_err(|e| format!("File read failed: {e:?}"))?;
                        let bytes = js_sys::Uint8Array::new(&buffer);
                        total = total.saturating_add(bytes.length() as usize);
                        if total > 8 * 1024 * 1024 {
                            return Err("Source/resource budget exceeded".into());
                        }
                        out.push(vba_ide::NativeFormResource {
                            name,
                            bytes: bytes.to_vec(),
                        });
                    }
                    Ok(out)
                }
                .await;
                *result.borrow_mut() = Some(imported);
                ctx.request_repaint();
            });
        });
        input.set_onchange(Some(callback.as_ref().unchecked_ref()));
        input.click();
        self.input = Some((input, callback));
        Ok(())
    }
    pub fn download(&mut self, name: &str, text: &str) -> Result<(), String> {
        self.download_bytes(name, text.as_bytes())
    }
    pub fn download_bytes(&mut self, name: &str, bytes: &[u8]) -> Result<(), String> {
        self.download_inner(name, bytes)
            .map_err(|e| format!("Browser download failed: {e:?}"))
    }
    fn download_inner(&mut self, name: &str, bytes: &[u8]) -> Result<(), JsValue> {
        if bytes.len() > 8 * 1024 * 1024 {
            return Err(JsValue::from_str("source budget exceeded"));
        }
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let array = js_sys::Uint8Array::from(bytes);
        let parts = js_sys::Array::of1(&array);
        let blob = web_sys::Blob::new_with_u8_array_sequence(&parts)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;
        let anchor = document
            .create_element("a")?
            .dyn_into::<web_sys::HtmlAnchorElement>()?;
        anchor.set_href(&url);
        anchor.set_download(name);
        anchor.set_attribute("hidden", "")?;
        document
            .body()
            .ok_or_else(|| JsValue::from_str("no body"))?
            .append_child(&anchor)?;
        anchor.click();
        anchor.remove();
        if self.urls.len() >= 16 {
            let old = self.urls.remove(0);
            web_sys::Url::revoke_object_url(&old)?;
        }
        self.urls.push(url);
        Ok(())
    }
}
impl Drop for BrowserFiles {
    fn drop(&mut self) {
        if let Some((input, _)) = self.input.take() {
            input.set_onchange(None);
            input.remove();
        }
        for url in &self.urls {
            let _ = web_sys::Url::revoke_object_url(url);
        }
    }
}
// Reuse eframe's exact egui family, without another direct toolkit dependency.
mod egui_context {
    pub use eframe::egui::Context;
}
