use std::collections::BTreeMap;

use rikka_gecko_protocol::geckoview::navigation::{
    ErrorCategory, ErrorCode, GoBack, GotoHistoryIndex, HeaderFilter, LoadDecision, LoadFlags,
    LoadUri, OnLoadRequest, OnNewSession, OpenFlags, OpenWhere, PurgeHistory, Reload,
    WebRequestError,
};
use rikka_gecko_protocol::geckoview::permission::{PermissionKind, PermissionValue};
use rikka_gecko_protocol::inbound::WindowEvent;
use rikka_gecko_protocol::{Target, WindowId};
use serde_json::{Value, json};

use crate::support::{answered, sent_data, window_event};

#[test]
fn decodes_navigation_events_as_the_modules_build_them() {
    let cases: [(&str, Value); 5] = [
        (
            "GeckoView:LocationChange",
            json!({
                "uri": "https://example.org/",
                "canGoBack": true,
                "canGoForward": false,
                "isTopLevel": true,
                "permissions": [{
                    "uri": "https://example.org/",
                    "principal": "eyIxIjp7IjAiOiJodHRwczovL2V4YW1wbGUub3JnLyJ9fQ==",
                    "perm": "trackingprotection",
                    "value": 2,
                    "contextId": "",
                    "privateMode": false,
                }],
                "hasUserGesture": false,
            }),
        ),
        (
            "GeckoView:LocationChange",
            json!({
                "uri": "about:blank",
                "canGoBack": false,
                "canGoForward": false,
                "isTopLevel": true,
                "hasUserGesture": false,
            }),
        ),
        (
            "GeckoView:OnLoadRequest",
            json!({
                "uri": "https://example.org/popup",
                "where": 3,
                "flags": 0,
                "triggerUri": "https://example.org/",
                "hasUserGesture": true,
            }),
        ),
        (
            "GeckoView:OnNewSession",
            json!({"uri": "https://example.org/popup", "newSessionId": "0123456789abcdef0123456789abcdef"}),
        ),
        (
            "GeckoView:OnLoadError",
            json!({"uri": "https://nowhere.invalid/", "error": 2152398878u32, "errorModule": 6, "errorClass": 0}),
        ),
    ];
    let decoded: Vec<WindowEvent> = cases
        .into_iter()
        .map(|(event, data)| window_event(event, data))
        .collect();

    let WindowEvent::LocationChange(change) = &decoded[0] else {
        panic!()
    };
    let permission = &change.permissions.as_ref().unwrap()[0];
    assert_eq!(permission.kind(), Some(PermissionKind::Tracking));
    assert_eq!(permission.value, PermissionValue::Deny);

    let WindowEvent::LocationChange(blank) = &decoded[1] else {
        panic!()
    };
    assert_eq!(blank.permissions, None);

    let WindowEvent::OnLoadRequest(request) = &decoded[2] else {
        panic!()
    };
    assert_eq!(
        request,
        &OnLoadRequest {
            uri: "https://example.org/popup".to_owned(),
            target: OpenWhere::NewTab,
            flags: OpenFlags::NONE,
            trigger_uri: Some("https://example.org/".to_owned()),
            has_user_gesture: true,
        }
    );
    assert!(request.target.opens_new_session());

    let WindowEvent::OnNewSession(OnNewSession { new_session_id, .. }) = &decoded[3] else {
        panic!()
    };
    assert_eq!(
        Target::from(new_session_id.clone()).to_string(),
        "window:0123456789abcdef0123456789abcdef"
    );

    let WindowEvent::OnLoadError(error) = &decoded[4] else {
        panic!()
    };
    assert_eq!(
        error.classify(),
        WebRequestError {
            code: ErrorCode::UnknownHost,
            category: ErrorCategory::Uri
        }
    );
}

#[test]
fn classifies_load_errors_like_web_request_error() {
    let cases = [
        (0x804B_001E, 0, ErrorCode::UnknownHost, ErrorCategory::Uri),
        (
            0x804B_000D,
            0,
            ErrorCode::ConnectionRefused,
            ErrorCategory::Network,
        ),
        (0x804B_0010, 0, ErrorCode::Offline, ErrorCategory::Network),
        (
            0x804B_0048,
            0,
            ErrorCode::ProxyConnectionRefused,
            ErrorCategory::Proxy,
        ),
        (0x804B_0047, 0, ErrorCode::Unknown, ErrorCategory::Unknown),
        (0x8052_0012, 0, ErrorCode::FileNotFound, ErrorCategory::Uri),
        (
            0x805D_001F,
            0,
            ErrorCode::SafeBrowsingPhishingUri,
            ErrorCategory::SafeBrowsing,
        ),
        (
            0x805E_0010,
            0,
            ErrorCode::ContentCrashed,
            ErrorCategory::Content,
        ),
        (
            0x805A_1FF3,
            2,
            ErrorCode::SecurityBadCert,
            ErrorCategory::Security,
        ),
        (
            0x805A_2FD2,
            1,
            ErrorCode::SecuritySsl,
            ErrorCategory::Security,
        ),
        (0x805A_2FD2, 0, ErrorCode::Unknown, ErrorCategory::Security),
    ];
    for (error, class, code, category) in cases {
        assert_eq!(
            WebRequestError::from_gecko(error, class),
            WebRequestError { code, category },
            "{error:#x}"
        );
    }
}

#[test]
fn encodes_navigation_commands_with_the_keys_the_module_reads() {
    let mut load = LoadUri::new("https://example.org/");
    load.flags = LoadFlags::BYPASS_CACHE | LoadFlags::REPLACE_HISTORY;
    load.header_filter = HeaderFilter::UnrestrictedUnsafe;
    load.referrer_session_id = Some(WindowId::new("a1").unwrap());
    load.headers = Some(BTreeMap::from([
        ("X-Test".to_owned(), Some("1".to_owned())),
        ("X-Empty".to_owned(), None),
    ]));
    load.original_input = Some("example.org".to_owned());

    let cases: [(Value, Value); 5] = [
        (
            sent_data(&load),
            json!({
                "uri": "https://example.org/",
                "flags": 65,
                "headerFilter": 2,
                "textDirectiveUserActivation": false,
                "referrerSessionId": "a1",
                "headers": {"X-Empty": null, "X-Test": "1"},
                "originalInput": "example.org",
                "appLinkLaunchType": 0,
            }),
        ),
        (
            sent_data(&Reload {
                flags: LoadFlags::BYPASS_CACHE,
            }),
            json!({"flags": 1}),
        ),
        (
            sent_data(&GoBack {
                user_interaction: true,
            }),
            json!({"userInteraction": true}),
        ),
        (
            sent_data(&GotoHistoryIndex { index: 2 }),
            json!({"index": 2}),
        ),
        (sent_data(&PurgeHistory), Value::Null),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
}

#[test]
fn load_decisions_answer_whether_the_embedder_handled_the_load() {
    assert_eq!(
        answered::<OnLoadRequest>(&LoadDecision::Allow),
        json!(false)
    );
    assert_eq!(answered::<OnLoadRequest>(&LoadDecision::Deny), json!(true));
}
