use std::path::Path;

use rikka_gecko_protocol::inbound::{RuntimeEvent, WindowEvent};
use rikka_gecko_protocol::rikka::downloads::{
    CancelDownload, DownloadId, DownloadReason, DownloadRequest, DownloadState, PauseDownload,
    SaveTo,
};
use rikka_gecko_protocol::rikka::notifications::{NotificationClicked, NotificationClosed};
use rikka_gecko_protocol::rikka::user_content::{
    CallFunction, Evaluate, Frames, InjectAt, MessageHandler, ScriptMessage, SetUserContent,
    StyleLevel, UserScript, UserStyle,
};
use rikka_gecko_protocol::rikka::view::{Snapshot, SnapshotRegion};
use rikka_gecko_protocol::{CallbackId, Reply};
use serde_json::{Map, Value, json};

use crate::support::{answered, gecko_reply, runtime_event, sent_data, window_event};

#[test]
fn decodes_download_events_as_the_components_build_them() {
    let requests = [
        (
            json!({"url": "https://example.org/a.zip", "filename": "a.zip", "mimeType": "application/zip", "contentLength": 2048, "reason": "attachment"}),
            DownloadReason::Attachment,
        ),
        (
            json!({"url": "https://example.org/x", "filename": "x", "mimeType": null, "contentLength": null, "reason": "unhandled"}),
            DownloadReason::Unhandled,
        ),
        (
            json!({"url": "https://example.org/v", "filename": "v.bin", "mimeType": "application/octet-stream", "contentLength": null, "reason": "sniffed"}),
            DownloadReason::Sniffed,
        ),
        (
            json!({"url": "https://example.org/p.png", "filename": "p.png", "mimeType": "image/png", "contentLength": 10, "reason": "save"}),
            DownloadReason::Save,
        ),
    ];
    for (data, reason) in requests {
        let WindowEvent::DownloadRequest(request) = window_event("Rikka:DownloadRequest", data)
        else {
            panic!()
        };
        assert_eq!(request.reason, reason);
    }

    let changes = [
        (
            json!({
                "id": "1",
                "url": "https://example.org/a.zip",
                "referrer": "https://example.org/",
                "private": false,
                "path": "/home/rikka/Downloads/a.zip",
                "mimeType": "application/zip",
                "state": "downloading",
                "currentBytes": 512,
                "totalBytes": 2048,
                "startTime": 1761436800000i64,
                "error": null,
            }),
            DownloadState::Downloading,
        ),
        (
            json!({
                "id": "2",
                "url": "https://example.org/stream",
                "referrer": null,
                "private": true,
                "path": "/tmp/stream",
                "mimeType": null,
                "state": "failed",
                "currentBytes": 0,
                "totalBytes": null,
                "startTime": null,
                "error": "Network failure",
            }),
            DownloadState::Failed,
        ),
    ];
    for (data, state) in changes {
        let RuntimeEvent::DownloadChanged(changed) = runtime_event("Rikka:Download:Changed", data)
        else {
            panic!()
        };
        assert_eq!(changed.state, state);
    }

    let RuntimeEvent::DownloadRemoved(removed) =
        runtime_event("Rikka:Download:Removed", json!({"id": "2"}))
    else {
        panic!()
    };
    assert_eq!(removed.id, DownloadId("2".to_owned()));
}

#[test]
fn answers_download_requests_with_an_absolute_destination_or_cancels() {
    let destination = SaveTo::new(Path::new("/home/rikka/Downloads/a.zip")).unwrap();
    assert_eq!(
        answered::<DownloadRequest>(&Some(destination)),
        json!({"path": "/home/rikka/Downloads/a.zip"})
    );
    assert_eq!(answered::<DownloadRequest>(&None), Value::Null);
    assert_eq!(SaveTo::new(Path::new("Downloads/a.zip")), None);
    assert!(serde_json::from_value::<SaveTo>(json!({"path": "a.zip"})).is_err());
}

#[test]
fn drives_downloads_by_id() {
    let id = DownloadId("7".to_owned());
    assert_eq!(
        sent_data(&CancelDownload { id: id.clone() }),
        json!({"id": "7"})
    );
    gecko_reply::<PauseDownload>(Value::Null);
    let failed = Reply::failure(CallbackId::FIRST, "No download 7");
    assert_eq!(
        failed.outcome::<PauseDownload>().unwrap(),
        Err(json!("No download 7"))
    );
}

#[test]
fn relays_notifications_both_ways() {
    let RuntimeEvent::ShowNotification(shown) = runtime_event(
        "Rikka:Notification:Show",
        json!({
            "name": "https://chat.example#4",
            "title": "New message",
            "text": "Hi",
            "imageUrl": "https://chat.example/avatar.png",
            "origin": "https://chat.example",
            "lang": null,
            "dir": "auto",
            "silent": false,
            "requireInteraction": true,
            "private": false,
            "actions": [{"action": "reply", "title": "Reply"}],
        }),
    ) else {
        panic!()
    };
    assert_eq!(shown.actions[0].action, "reply");
    assert!(shown.require_interaction);

    let RuntimeEvent::CloseNotification(closed) = runtime_event(
        "Rikka:Notification:Close",
        json!({"name": "https://chat.example#4"}),
    ) else {
        panic!()
    };
    assert_eq!(closed.name, shown.name);

    let cases: [(Value, Value); 3] = [
        (
            sent_data(&NotificationClicked {
                name: "n".to_owned(),
                action: Some("reply".to_owned()),
            }),
            json!({"name": "n", "action": "reply"}),
        ),
        (
            sent_data(&NotificationClicked {
                name: "n".to_owned(),
                action: None,
            }),
            json!({"name": "n", "action": null}),
        ),
        (
            sent_data(&NotificationClosed {
                name: "n".to_owned(),
            }),
            json!({"name": "n"}),
        ),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
}

#[test]
fn sends_user_content_with_explicit_worlds() {
    let content = SetUserContent {
        scripts: vec![UserScript {
            id: "dark".to_owned(),
            source: "document.documentElement.dataset.dark = 1;".to_owned(),
            inject_at: InjectAt::Start,
            frames: Frames::All,
            world: None,
            allow: vec!["*://*/*".to_owned()],
            block: vec![],
        }],
        styles: vec![UserStyle {
            id: "fonts".to_owned(),
            source: "body { font-family: sans-serif }".to_owned(),
            frames: Frames::Top,
            level: StyleLevel::User,
            allow: vec!["<all_urls>".to_owned()],
            block: vec!["https://bank.example/*".to_owned()],
        }],
        handlers: vec![MessageHandler {
            name: "rikka".to_owned(),
            world: Some("rikka".to_owned()),
            replying: true,
        }],
    };
    let cases: [(Value, Value); 3] = [
        (
            sent_data(&content),
            json!({
                "scripts": [{
                    "id": "dark",
                    "source": "document.documentElement.dataset.dark = 1;",
                    "injectAt": "start",
                    "frames": "all",
                    "world": null,
                    "allow": ["*://*/*"],
                    "block": [],
                }],
                "styles": [{
                    "id": "fonts",
                    "source": "body { font-family: sans-serif }",
                    "frames": "top",
                    "level": "user",
                    "allow": ["<all_urls>"],
                    "block": ["https://bank.example/*"],
                }],
                "handlers": [{"name": "rikka", "world": "rikka", "replying": true}],
            }),
        ),
        (
            sent_data(&Evaluate {
                world: None,
                source: "document.title".to_owned(),
            }),
            json!({"world": null, "source": "document.title"}),
        ),
        (
            sent_data(&CallFunction {
                world: Some("rikka".to_owned()),
                body: "return a + b;".to_owned(),
                arguments: Map::from_iter([("a".to_owned(), json!(1)), ("b".to_owned(), json!(2))]),
            }),
            json!({"world": "rikka", "body": "return a + b;", "arguments": {"a": 1, "b": 2}}),
        ),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
    assert_eq!(
        gecko_reply::<Evaluate>(json!("Example Domain")),
        json!("Example Domain")
    );
}

#[test]
fn receives_script_messages_and_answers_them() {
    let messages = [
        json!({"name": "rikka", "world": "rikka", "body": {"kind": "ping"}, "frame": {"url": "https://example.org/", "top": true}}),
        json!({"name": "page", "world": null, "body": [1, 2], "frame": {"url": "https://ads.example/f", "top": false}}),
    ];
    for data in messages {
        let WindowEvent::ScriptMessage(message) =
            window_event("Rikka:UserContent:Message", data.clone())
        else {
            panic!()
        };
        assert_eq!(message.body, data["body"]);
        assert_eq!(message.frame.top, data["frame"]["top"].as_bool().unwrap());
    }
    assert_eq!(
        answered::<ScriptMessage>(&json!({"pong": true})),
        json!({"pong": true})
    );
}

#[test]
fn carries_view_events_as_rikka_view_builds_them() {
    let WindowEvent::Icons(icons) = window_event(
        "Rikka:View:Icons",
        json!({"page": "https://example.org/", "icons": [
            {"url": "https://example.org/a.png", "rel": "icon", "sizes": ["32x32"], "type": "image/png"},
            {"url": "https://example.org/favicon.ico", "rel": "icon", "sizes": [], "type": null},
        ]}),
    ) else {
        panic!()
    };
    assert_eq!(icons.icons[0].mime_type.as_deref(), Some("image/png"));
    assert_eq!(icons.icons[1].mime_type, None);

    let WindowEvent::Tooltip(tooltip) = window_event(
        "Rikka:View:Tooltip",
        json!({"x": 10, "y": 20, "text": "Help", "direction": "ltr"}),
    ) else {
        panic!()
    };
    assert_eq!(tooltip.text, "Help");
    assert!(matches!(
        window_event("Rikka:View:TooltipHidden", json!({})),
        WindowEvent::TooltipHidden(_)
    ));
    let WindowEvent::OverLink(link) = window_event("Rikka:View:OverLink", json!({"url": null}))
    else {
        panic!()
    };
    assert_eq!(link.url, None);

    assert_eq!(
        sent_data(&Snapshot {
            region: SnapshotRegion::Document,
            scale: None,
        }),
        json!({"region": "document", "scale": null})
    );
    let image = gecko_reply::<Snapshot>(json!({"width": 2, "height": 1, "png": "iVBOR"}));
    assert_eq!((image.width, image.height), (2, 1));
}
