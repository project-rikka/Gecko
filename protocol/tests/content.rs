use rikka_gecko_protocol::geckoview::content::{
    ExitFullScreen, HangReportStop, Priority, ScreenLengthType, ScrollBehavior, ScrollBy,
    ScrollRequest, SetActive, SetFocused, SetPriorityHint,
};
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{sent_data, window_event};

#[test]
fn decodes_content_events_as_the_actors_build_them() {
    let cases: [(&str, Value); 11] = [
        (
            "GeckoView:ContextMenu",
            json!({
                "screenX": 120.5,
                "screenY": 300,
                "baseUri": "https://example.org/",
                "uri": "https://example.org/next",
                "title": null,
                "alt": null,
                "elementType": "HTMLImageElement",
                "elementSrc": "https://example.org/cat.png",
                "textContent": null,
                "linkText": "Next",
            }),
        ),
        (
            "GeckoView:DOMMetaViewportFit",
            json!({"viewportfit": "cover"}),
        ),
        (
            "GeckoView:PreviewImage",
            json!({"previewImageUrl": "https://example.org/og.png"}),
        ),
        (
            "GeckoView:WebAppManifest",
            json!({"manifest": {"name": "Example", "start_url": "https://example.org/", "icons": []}}),
        ),
        ("GeckoView:DOMWindowClose", Value::Null),
        ("GeckoView:FocusRequest", Value::Null),
        ("GeckoView:ContentCrash", Value::Null),
        ("GeckoView:FirstContentfulPaint", Value::Null),
        (
            "GeckoView:ScrollChanged",
            json!({"scrollX": 0, "scrollY": 812.25}),
        ),
        ("GeckoView:PinOnScreen", json!({"pinned": true})),
        (
            "GeckoView:HangReport",
            json!({"hangId": 3, "scriptFileName": "https://example.org/app.js"}),
        ),
    ];
    let decoded: Vec<WindowEvent> = cases
        .into_iter()
        .map(|(event, data)| window_event(event, data))
        .collect();

    let WindowEvent::ContextMenu(menu) = &decoded[0] else {
        panic!()
    };
    assert_eq!(menu.element_type, "HTMLImageElement");
    assert_eq!(menu.link_text.as_deref(), Some("Next"));
    assert_eq!(menu.title, None);

    let WindowEvent::ScrollChanged(scroll) = &decoded[8] else {
        panic!()
    };
    assert_eq!(scroll.scroll_y, 812.25);

    let WindowEvent::HangReport(hang) = &decoded[10] else {
        panic!()
    };
    assert_eq!(hang.hang_id, 3);
}

#[test]
fn encodes_content_commands_with_the_keys_the_module_reads() {
    let cases: [(Value, Value); 6] = [
        (
            sent_data(&SetActive { active: false }),
            json!({"active": false}),
        ),
        (
            sent_data(&SetFocused { focused: true }),
            json!({"focused": true}),
        ),
        (
            sent_data(&SetPriorityHint {
                priority_hint: Priority::High,
            }),
            json!({"priorityHint": 1}),
        ),
        (
            sent_data(&ScrollBy(ScrollRequest {
                width_value: 0.0,
                width_type: ScreenLengthType::Pixel,
                height_value: 0.5,
                height_type: ScreenLengthType::VisualViewportHeight,
                behavior: ScrollBehavior::Smooth,
            })),
            json!({"widthValue": 0.0, "widthType": 0, "heightValue": 0.5, "heightType": 2, "behavior": 0}),
        ),
        (
            sent_data(&HangReportStop { hang_id: 3 }),
            json!({"hangId": 3}),
        ),
        (sent_data(&ExitFullScreen), Value::Null),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
}
