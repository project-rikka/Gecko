use rikka_gecko_protocol::geckoview::progress::{GetQwacStatus, IdentityMode, MixedMode};
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{gecko_reply, window_event};

#[test]
fn decodes_progress_events_as_the_trackers_build_them() {
    let cases: [(&str, Value); 6] = [
        (
            "GeckoView:PageStart",
            json!({"uri": "https://example.org/"}),
        ),
        ("GeckoView:PageStop", json!({"success": false})),
        ("GeckoView:ProgressChanged", json!({"progress": 55})),
        (
            "GeckoView:SecurityChanged",
            json!({
                "identity": {
                    "mode": {"identity": 0, "mixed_display": 0, "mixed_active": 0},
                    "origin": "http://example.org",
                    "secure": false,
                },
            }),
        ),
        (
            "GeckoView:SecurityChanged",
            json!({
                "identity": {
                    "mode": {"identity": 1, "mixed_display": 2, "mixed_active": 1},
                    "origin": "https://example.org",
                    "secure": true,
                    "host": "example.org",
                    "certificate": "MIIBIjANBgkq",
                    "securityException": false,
                },
            }),
        ),
        (
            "GeckoView:PageTitleChanged",
            json!({"title": "Example Domain"}),
        ),
    ];
    let decoded: Vec<WindowEvent> = cases
        .into_iter()
        .map(|(event, data)| window_event(event, data))
        .collect();

    let WindowEvent::ProgressChanged(progress) = &decoded[2] else {
        panic!()
    };
    assert_eq!(progress.progress, 55);

    let WindowEvent::SecurityChanged(insecure) = &decoded[3] else {
        panic!()
    };
    assert!(!insecure.identity.secure);
    assert_eq!(insecure.identity.host, None);

    let WindowEvent::SecurityChanged(secure) = &decoded[4] else {
        panic!()
    };
    assert_eq!(secure.identity.mode.identity, IdentityMode::Identified);
    assert_eq!(secure.identity.mode.mixed_display, MixedMode::ContentLoaded);
    assert_eq!(secure.identity.mode.mixed_active, MixedMode::ContentBlocked);
    assert_eq!(secure.identity.security_exception, Some(false));
}

#[test]
fn qwac_status_is_absent_for_ordinary_certificates() {
    assert_eq!(gecko_reply::<GetQwacStatus>(Value::Null), None);
    assert_eq!(
        gecko_reply::<GetQwacStatus>(json!("MIIC")),
        Some("MIIC".to_owned())
    );
}
