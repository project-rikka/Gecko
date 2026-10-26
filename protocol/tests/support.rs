use rikka_gecko_protocol::inbound::{PortEvent, RuntimeEvent, WindowEvent};
use rikka_gecko_protocol::{
    CallbackId, EmbedderMessage, EventMessage, ForGecko, GeckoMessage, Request, Target,
};
use serde_json::{Value, json};

pub fn gecko_event(target: &str, event: &str, data: Value) -> EventMessage {
    let frame = json!({
        "type": "event",
        "target": target,
        "event": event,
        "data": data,
        "callback": 7,
    });
    match serde_json::from_value(frame).expect("an event frame from Gecko") {
        GeckoMessage::Event(message) => message,
        other => panic!("expected an event, got {other:?}"),
    }
}

pub fn window_event(event: &str, data: Value) -> WindowEvent {
    WindowEvent::decode(&gecko_event("window:a1", event, data))
        .unwrap_or_else(|error| panic!("{event} does not decode: {error}"))
        .unwrap_or_else(|| panic!("{event} is not a window event"))
}

pub fn runtime_event(event: &str, data: Value) -> RuntimeEvent {
    RuntimeEvent::decode(&gecko_event("runtime", event, data))
        .unwrap_or_else(|error| panic!("{event} does not decode: {error}"))
        .unwrap_or_else(|| panic!("{event} is not a runtime event"))
}

pub fn port_event(event: &str, data: Value) -> PortEvent {
    PortEvent::decode(&gecko_event("named:port:3", event, data))
        .unwrap_or_else(|error| panic!("{event} does not decode: {error}"))
        .unwrap_or_else(|| panic!("{event} is not a port event"))
}

pub fn sent<E: ForGecko>(target: Target, payload: &E) -> Value {
    let message = EmbedderMessage::send(target, payload, CallbackId::new(1)).expect("serializable");
    serde_json::to_value(message).expect("serializable")
}

pub fn sent_data<E: ForGecko>(payload: &E) -> Value {
    sent(Target::Runtime, payload)
        .get("data")
        .cloned()
        .unwrap_or(Value::Null)
}

pub fn answered<E: Request>(response: &E::Response) -> Value {
    let reply = rikka_gecko_protocol::Reply::success::<E>(CallbackId::FIRST, response)
        .expect("serializable");
    reply.data.unwrap_or(Value::Null)
}

pub fn gecko_reply<E: Request>(data: Value) -> E::Response {
    let frame = json!({"type": "reply", "callback": 4, "success": true, "data": data});
    match serde_json::from_value(frame).expect("a reply frame from Gecko") {
        GeckoMessage::Reply(reply) => reply
            .outcome::<E>()
            .expect("decodable")
            .expect("successful"),
        other => panic!("expected a reply, got {other:?}"),
    }
}
