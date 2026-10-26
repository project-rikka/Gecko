use rikka_gecko_protocol::geckoview::session_state::{RestoreState, SessionState};
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{sent_data, window_event};

fn entry(url: &str) -> Value {
    json!({
        "url": url,
        "title": url,
        "cacheKey": 0,
        "ID": 1,
        "docshellUUID": "{a}",
        "triggeringPrincipal_base64": "e30=",
        "hasUserInteraction": true,
        "persist": true,
    })
}

fn update(data: Value) -> SessionState {
    let WindowEvent::StateUpdated(updated) =
        window_event("GeckoView:StateUpdated", json!({"data": data}))
    else {
        panic!()
    };
    let mut state = SessionState::default();
    state.apply(updated.data);
    state
}

fn apply(state: &mut SessionState, data: Value) {
    let WindowEvent::StateUpdated(updated) =
        window_event("GeckoView:StateUpdated", json!({"data": data}))
    else {
        panic!()
    };
    state.apply(updated.data);
}

fn urls(state: &SessionState) -> Vec<&str> {
    state
        .entries()
        .iter()
        .map(|entry| entry.url.as_str())
        .collect()
}

#[test]
fn accumulates_session_state_like_geckoview() {
    let mut state = update(json!({
        "historychange": {
            "entries": [entry("https://a/"), entry("https://b/"), entry("https://c/")],
            "requestedIndex": 0,
            "index": 3,
            "fromIdx": -1,
        },
    }));
    assert_eq!(urls(&state), ["https://a/", "https://b/", "https://c/"]);
    assert_eq!(state.current_index(), Some(2));

    let cases: [(Value, &[&str], Option<usize>); 4] = [
        (
            json!({"historychange": {"entries": [entry("https://d/")], "requestedIndex": 0, "index": 2, "fromIdx": 0}}),
            &["https://a/", "https://d/"],
            Some(1),
        ),
        (
            json!({"historychange": {"entries": [], "requestedIndex": 0, "index": 1, "fromIdx": 9007199254740990i64}}),
            &["https://a/", "https://d/"],
            Some(0),
        ),
        (
            json!({"scroll": {"scroll": "0,120"}, "formdata": {"id": {"q": "rikka"}}}),
            &["https://a/", "https://d/"],
            Some(0),
        ),
        (
            json!({"historychange": {"entries": [entry("https://e/")], "requestedIndex": 0, "index": 1, "fromIdx": -1}}),
            &["https://e/"],
            Some(0),
        ),
    ];
    for (data, expected, index) in cases {
        apply(&mut state, data);
        assert_eq!(urls(&state), expected);
        assert_eq!(state.current_index(), index);
    }
    assert_eq!(state.scrolldata, Some(json!({"scroll": "0,120"})));
}

#[test]
fn restores_with_every_field_it_was_given() {
    let state = update(json!({
        "historychange": {
            "entries": [entry("https://a/")],
            "requestedIndex": 0,
            "index": 1,
            "fromIdx": -1,
            "userContextId": 0,
        },
        "scroll": {"scroll": "0,40"},
    }));
    let restored = sent_data(&RestoreState(state.clone()));
    assert_eq!(
        restored,
        json!({
            "history": {
                "entries": [entry("https://a/")],
                "requestedIndex": 0,
                "index": 1,
                "userContextId": 0,
            },
            "scrolldata": {"scroll": "0,40"},
        })
    );
    assert_eq!(
        serde_json::from_value::<SessionState>(restored).unwrap(),
        state
    );
}
