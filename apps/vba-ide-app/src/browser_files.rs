//! Explicit browser upload/download through public web APIs; no ambient file IO.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
type ImportResult = Result<(String, String), String>;
#[derive(Default)]
pub struct BrowserFiles {
    result: Rc<RefCell<Option<ImportResult>>>,
    input: Option<(
        web_sys::HtmlInputElement,
        Closure<dyn FnMut(web_sys::Event)>,
    )>,
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
        input.set_accept(".bas,text/plain");
        input.set_attribute("hidden", "")?;
        document
            .body()
            .ok_or_else(|| JsValue::from_str("no body"))?
            .append_child(&input)?;
        let selected = input.clone();
        let result = self.result.clone();
        let callback = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            let Some(file) = selected.files().and_then(|files| files.get(0)) else {
                return;
            };
            let result = result.clone();
            let ctx = ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let name = file.name();
                let imported = if !name.to_ascii_lowercase().ends_with(".bas")
                    || file.size() > vba_ide::SourceFile::MAX_BYTES as f64
                {
                    Err("Choose a bounded UTF-8 .bas file".into())
                } else {
                    match wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await {
                        Ok(buffer) => {
                            let bytes = js_sys::Uint8Array::new(&buffer);
                            if bytes.length() as usize > vba_ide::SourceFile::MAX_BYTES {
                                Err("Source budget exceeded".into())
                            } else {
                                String::from_utf8(bytes.to_vec())
                                    .map(|text| (name, text))
                                    .map_err(|_| "Expected UTF-8 source".into())
                            }
                        }
                        Err(e) => Err(format!("File read failed: {e:?}")),
                    }
                };
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
        self.download_inner(name, text)
            .map_err(|e| format!("Browser download failed: {e:?}"))
    }
    fn download_inner(&mut self, name: &str, text: &str) -> Result<(), JsValue> {
        if text.len() > vba_ide::SourceFile::MAX_BYTES {
            return Err(JsValue::from_str("source budget exceeded"));
        }
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let parts = js_sys::Array::of1(&JsValue::from_str(text));
        let blob = web_sys::Blob::new_with_str_sequence(&parts)?;
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
