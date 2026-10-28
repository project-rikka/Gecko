use rikka_gecko_protocol::geckoview::history::{GetVisited, OnVisited, VisitFlags};
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{answered, window_event};

#[test]
fn decodes_history_queries_as_geckoview_history_builds_them() {
    let cases: [(&str, Value); 4] = [
        (
            "GeckoView:OnVisited",
            json!({"url": "https://example.org/b", "lastVisitedURL": "https://example.org/a", "flags": 3}),
        ),
        (
            "GeckoView:OnVisited",
            json!({"url": "https://example.org/a", "flags": 1}),
        ),
        (
            "GeckoView:GetVisited",
            json!({"urls": ["https://a.example/", "https://b.example/"]}),
        ),
        (
            "GeckoView:GetHostVisitedSince",
            json!({"host": "example.org", "after": 1761436800000i64, "before": 1761523200000i64}),
        ),
    ];
    let decoded: Vec<WindowEvent> = cases
        .into_iter()
        .map(|(event, data)| window_event(event, data))
        .collect();

    let WindowEvent::OnVisited(redirect) = &decoded[0] else {
        panic!()
    };
    assert!(
        redirect
            .flags
            .contains(VisitFlags::TOP_LEVEL | VisitFlags::REDIRECT_TEMPORARY)
    );
    assert_eq!(
        redirect.last_visited_url.as_deref(),
        Some("https://example.org/a")
    );

    let WindowEvent::OnVisited(first) = &decoded[1] else {
        panic!()
    };
    assert_eq!(first.last_visited_url, None);

    let WindowEvent::GetHostVisitedSince(since) = &decoded[3] else {
        panic!()
    };
    assert_eq!(since.after, 1761436800000);
}

#[test]
fn answers_history_queries_with_plain_values() {
    assert_eq!(answered::<OnVisited>(&true), json!(true));
    assert_eq!(
        answered::<GetVisited>(&Some(vec![true, false])),
        json!([true, false])
    );
    assert_eq!(answered::<GetVisited>(&None), Value::Null);
}
