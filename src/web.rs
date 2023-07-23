use crate::{EsEvent, EventHandler, Result};
use wasm_bindgen_futures::spawn_local;
use web_sys::EventSource;

fn new_event_source(url: &str) -> Result<EventSource> {
    EventSource::new(url).map_err(|_| "couldn't aquire event source".to_string())
}

pub fn es_connect(url: String, on_event: EventHandler) -> Result<()> {
    spawn_local(async move {
        es_connect_async(url, on_event).await;
    });

    Ok(())
}

pub async fn es_connect_async(url: String, on_event: EventHandler) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast as _;
    use web_sys::console;

    let es = new_event_source(&url).unwrap();

    let on_event: std::rc::Rc<dyn Send + Fn(EsEvent) -> std::ops::ControlFlow<()>> =
        on_event.into();

    {
        let on_event = on_event.clone();
        let es_event = es.clone();
        let onerror_callback = Closure::wrap(Box::new(move |error_event: wasm_bindgen::JsValue| {
            console::log_2(
                &wasm_bindgen::JsValue::from_str("connect onerror"),
                &error_event,
            );
            let err_msg = format!("{:?}", error_event);
            let res = on_event(EsEvent::Error(err_msg));
            if let std::ops::ControlFlow::Break(_) = res {
                es_event.close();
            };
        }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);
        es.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
    }

    {
        let on_event = on_event.clone();
        let es_event = es.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            let res = on_event(EsEvent::Opened);
            if let std::ops::ControlFlow::Break(_) = res {
                es_event.close();
            };
        }) as Box<dyn FnMut(wasm_bindgen::JsValue)>);
        es.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
    }

    {
        let es_event = es.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |m: web_sys::MessageEvent| {
            // Handle
            let txt = m.data().as_string().unwrap_or(format!("{:#?}", m.data()));
            // todo: rm
            console::log_2(&wasm_bindgen::JsValue::from_str(&txt), &m.data());
            let res = on_event(EsEvent::Message(txt));
            if let std::ops::ControlFlow::Break(_) = res {
                es_event.close();
            };
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        // set message event handler on Eventsource
        es.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        // forget the callback to keep it alive
        onmessage_callback.forget();
    }
}
