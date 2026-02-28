use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use serde::{de::DeserializeOwned, Serialize};
use serde_wasm_bindgen::{to_value, from_value};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke_extern(cmd: &str, args: JsValue) -> JsValue;
    
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], js_name = listen)]
    async fn listen_extern(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> JsValue;
}

pub async fn invoke<T: DeserializeOwned>(cmd: &str, args: impl Serialize) -> Result<T, String> {
    let args = to_value(&args).map_err(|e| e.to_string())?;
    let result = invoke_extern(cmd, args).await;
    from_value(result).map_err(|e| e.to_string())
}

pub async fn invoke_no_args<T: DeserializeOwned>(cmd: &str) -> Result<T, String> {
    let result = invoke_extern(cmd, JsValue::NULL).await;
    from_value(result).map_err(|e| e.to_string())
}

pub struct Listener {
    unlisten: js_sys::Function,
    #[allow(dead_code)]
    closure: Closure<dyn FnMut(JsValue)>,
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self.unlisten.call0(&JsValue::NULL);
    }
}

pub async fn listen<F>(event: &str, handler: F) -> Result<Listener, String>
where
    F: FnMut(JsValue) + 'static,
{
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(JsValue)>);
    let unlisten_val = listen_extern(event, &closure).await;
    let unlisten = unlisten_val.dyn_into::<js_sys::Function>().map_err(|_| "Failed to cast unlisten to function".to_string())?;
    
    Ok(Listener {
        unlisten,
        closure,
    })
}
