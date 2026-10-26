use std::collections::BTreeMap;

use rikka_gecko_protocol::WindowId;
use rikka_gecko_protocol::geckoview::content_blocking::{
    GetEarliestRecordedDate, GetEventsByDateRange, RequestLog,
};
use rikka_gecko_protocol::geckoview::preferences::{
    GetPrefs, PrefAssignment, PrefBranch, PrefType, PrefValue, SetDefaultPrefs, SetPrefs,
};
use rikka_gecko_protocol::geckoview::runtime::{OpenedWindow, ServiceWorkerOpenWindow, SetLocale};
use rikka_gecko_protocol::geckoview::session::SessionContextId;
use rikka_gecko_protocol::geckoview::storage::{ClearData, ClearFlags, ClearSessionContextData};
use rikka_gecko_protocol::inbound::{RuntimeEvent, WindowEvent};
use serde_json::{Value, json};

use crate::support::{answered, gecko_reply, runtime_event, sent_data, window_event};

#[test]
fn encodes_runtime_commands_with_the_keys_startup_reads() {
    let cases: [(Value, Value); 6] = [
        (
            sent_data(&SetDefaultPrefs(BTreeMap::from([
                (
                    "browser.display.use_document_fonts".to_owned(),
                    PrefValue::Int(1),
                ),
                ("javascript.enabled".to_owned(), PrefValue::Bool(true)),
                (
                    "intl.locale.requested".to_owned(),
                    PrefValue::String("ja".to_owned()),
                ),
            ]))),
            json!({
                "browser.display.use_document_fonts": 1,
                "intl.locale.requested": "ja",
                "javascript.enabled": true,
            }),
        ),
        (
            sent_data(&SetLocale {
                requested_locales: Some(vec!["ja-JP".to_owned(), "en-US".to_owned()]),
                accept_languages: "ja-JP,ja,en-US,en".to_owned(),
            }),
            json!({"requestedLocales": ["ja-JP", "en-US"], "acceptLanguages": "ja-JP,ja,en-US,en"}),
        ),
        (
            sent_data(&SetPrefs {
                prefs: vec![PrefAssignment {
                    pref: "privacy.globalprivacycontrol.enabled".to_owned(),
                    value: PrefValue::Bool(true),
                    branch: PrefBranch::User,
                }],
            }),
            json!({"prefs": [{"pref": "privacy.globalprivacycontrol.enabled", "type": 128, "value": true, "branch": "user"}]}),
        ),
        (
            sent_data(&ClearData {
                flags: ClearFlags::COOKIES | ClearFlags::HISTORY,
            }),
            json!({"flags": 9}),
        ),
        (
            sent_data(&ClearSessionContextData {
                context_id: SessionContextId::from_unsafe("work"),
            }),
            json!({"contextId": "gvctx776f726b"}),
        ),
        (
            sent_data(&GetEventsByDateRange {
                date_from: 1761004800000,
                date_to: 1761523200000,
            }),
            json!({"dateFrom": 1761004800000i64, "dateTo": 1761523200000i64}),
        ),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
}

#[test]
fn decodes_runtime_replies_and_notifications() {
    let prefs = gecko_reply::<GetPrefs>(json!({
        "prefs": [
            {"pref": "javascript.enabled", "type": 128, "defaultValue": true, "userValue": false},
            {"pref": "browser.startup.page", "type": 64, "defaultValue": 1, "userValue": null},
            {"pref": "no.such.pref", "type": 0, "defaultValue": null, "userValue": null},
        ],
    }));
    assert_eq!(prefs.prefs[0].value(), Some(&PrefValue::Bool(false)));
    assert_eq!(prefs.prefs[1].value(), Some(&PrefValue::Int(1)));
    assert_eq!(prefs.prefs[2].pref_type, PrefType::Invalid);

    let RuntimeEvent::PreferenceChanged(changed) = runtime_event(
        "GeckoView:GeckoPreferences:Change",
        json!({"data": {"pref": "general.useragent.locale", "type": 32, "defaultValue": "en-US", "userValue": "ja"}}),
    ) else {
        panic!()
    };
    assert_eq!(
        changed.data.value(),
        Some(&PrefValue::String("ja".to_owned()))
    );

    let RuntimeEvent::ServiceWorkerOpenWindow(open) = runtime_event(
        "GeckoView:ServiceWorkerOpenWindow",
        json!({"url": "https://example.org/inbox"}),
    ) else {
        panic!()
    };
    assert_eq!(open.url, "https://example.org/inbox");
    assert_eq!(
        answered::<ServiceWorkerOpenWindow>(&Some(OpenedWindow {
            is_open: false,
            session_id: WindowId::new("c3").unwrap(),
        })),
        json!({"isOpen": false, "sessionId": "c3"})
    );

    let log = gecko_reply::<RequestLog>(json!({
        "log": [{"origin": "https://tracker.example", "blockData": [{"category": 4096, "blocked": true, "count": 2}]}],
    }));
    assert_eq!(log.log[0].block_data[0].count, 2);
    assert_eq!(
        gecko_reply::<GetEarliestRecordedDate>(json!({"date": 0})).date,
        0
    );

    let WindowEvent::ContentBlockingEvent(blocked) = window_event(
        "GeckoView:ContentBlockingEvent",
        json!({
            "uri": "https://tracker.example/pixel.gif",
            "category": 4096,
            "blockedList": "content-track-digest256",
            "loadedLists": [],
        }),
    ) else {
        panic!()
    };
    assert_eq!(
        blocked.blocked_list.as_deref(),
        Some("content-track-digest256")
    );
}

#[test]
fn rejects_pref_assignments_whose_type_disagrees_with_the_value() {
    let mismatched = json!({"pref": "a.b", "type": 64, "value": "text", "branch": "default"});
    assert!(serde_json::from_value::<PrefAssignment>(mismatched).is_err());
}
