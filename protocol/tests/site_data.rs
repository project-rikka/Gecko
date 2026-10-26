use rikka_gecko_protocol::geckoview::session::SessionContextId;
use rikka_gecko_protocol::geckoview::storage::ClearFlags;
use rikka_gecko_protocol::rikka::extensions::GetExtensionIcon;
use rikka_gecko_protocol::rikka::storage::{ClearDataBetween, GetCookies, GetSiteData, SameSite};
use serde_json::{Value, json};

use crate::support::{gecko_reply, sent_data};

#[test]
fn asks_for_site_data_in_a_partition() {
    let context = SessionContextId::from_unsafe("music");
    let cases: [(Value, Value); 4] = [
        (
            sent_data(&ClearDataBetween {
                from: 1_761_436_800_000,
                to: 1_761_440_400_000,
                flags: ClearFlags::COOKIES | ClearFlags::DOM_STORAGES,
            }),
            json!({"from": 1_761_436_800_000u64, "to": 1_761_440_400_000u64, "flags": 17}),
        ),
        (
            sent_data(&GetSiteData {
                host: "example.org".to_owned(),
                private: false,
                context_id: None,
            }),
            json!({"host": "example.org", "private": false, "contextId": null}),
        ),
        (
            sent_data(&GetCookies {
                uri: "https://www.youtube.com/".to_owned(),
                private: true,
                context_id: Some(context.clone()),
            }),
            json!({"uri": "https://www.youtube.com/", "private": true, "contextId": context}),
        ),
        (
            sent_data(&GetExtensionIcon {
                uri: "moz-extension://4a2f/icon.png".to_owned(),
            }),
            json!({"uri": "moz-extension://4a2f/icon.png"}),
        ),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
}

#[test]
fn reads_site_data_and_cookies_as_rikka_storage_answers() {
    let data = gecko_reply::<GetSiteData>(json!({"cookies": 3, "usage": 40960}));
    assert_eq!((data.cookies, data.usage), (3, 40960));

    let cookies = gecko_reply::<GetCookies>(json!([
        {"name": "SID", "value": "a1", "host": ".youtube.com", "path": "/", "secure": true,
         "httpOnly": true, "expiry": 1793000000000i64, "sameSite": "lax"},
        {"name": "PREF", "value": "f6=8", "host": "www.youtube.com", "path": "/", "secure": false,
         "httpOnly": false, "expiry": null, "sameSite": "unset"},
    ]));
    assert!(cookies[0].http_only);
    assert_eq!(cookies[0].same_site, SameSite::Lax);
    assert_eq!(cookies[1].expiry, None);

    let icon = gecko_reply::<GetExtensionIcon>(json!({"mimeType": "image/png", "data": "iVBOR"}));
    assert_eq!(icon.mime_type.as_deref(), Some("image/png"));
}
