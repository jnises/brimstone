use anyhow::Result;
use rfd::{MessageDialog, MessageLevel};

#[cfg(target_arch = "wasm32")]
fn save_png_file_impl(data: &[u8]) -> Result<()> {
    use anyhow::anyhow;
    use eframe::wasm_bindgen::{JsCast, JsValue};
    use js_sys::{Array, Uint8Array};
    use web_sys::{self, Blob, HtmlAnchorElement, Url};
    // TODO isn't there a nicer way to do this?
    (|| -> Result<(), JsValue> {
        let blob =
            Blob::new_with_u8_array_sequence(&JsValue::from(Array::of1(&Uint8Array::from(data))))
                .expect("file blob");
        let file_name = "gradient.png";
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("body");
        let a = document
            .create_element("a")?
            .dyn_into::<HtmlAnchorElement>()?;
        body.append_child(&a)?;
        a.style().set_css_text("display: none");
        let url = Url::create_object_url_with_blob(&blob)?;
        a.set_href(&url);
        a.set_download(file_name);
        a.click();
        Url::revoke_object_url(&a.href())?;
        a.remove();
        Ok(())
    })()
    .map_err(|e| anyhow!(e.as_string().unwrap_or("unknown error".into())))
}

#[cfg(not(target_arch = "wasm32"))]
fn save_png_file_impl(data: &[u8]) -> Result<()> {
    use rfd::FileDialog;
    use std::io::Write;

    if let Some(path) = FileDialog::new()
        .add_filter("PNG Image", &["png"])
        .save_file()
    {
        let mut f = std::fs::File::create(path)?;
        f.write_all(data)?;
    }
    Ok(())
}

pub fn save_png_file(data: &[u8]) {
    if let Err(e) = save_png_file_impl(data) {
        let _ = MessageDialog::new()
            .set_level(MessageLevel::Error)
            .set_title("error")
            .set_description(&e.to_string())
            .show();
    }
}
