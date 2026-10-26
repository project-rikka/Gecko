use rikka_gecko_protocol::geckoview::permission::{
    ContentPermission, ContentPermissionRequest, GetAllPermissions, GetPermissionsByUri,
    MediaPermissionDecision, MediaPermissionRequest, PermissionKind, PermissionValue,
    SetPermission, SetPermissionByUri,
};
use rikka_gecko_protocol::geckoview::session::SessionContextId;
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{answered, gecko_reply, sent_data, window_event};

fn camera() -> Value {
    json!({
        "type": "videoinput",
        "id": "cam-raw",
        "rawId": "cam-raw",
        "name": "Integrated Camera",
        "mediaSource": "camera",
    })
}

#[test]
fn decodes_permission_requests_as_the_prompt_builds_them() {
    let cases: [(&str, Value); 5] = [
        (
            "GeckoView:ContentPermission",
            json!({
                "uri": "https://maps.example/",
                "thirdPartyOrigin": "https://maps.example",
                "principal": "eyJ9",
                "perm": "geolocation",
                "value": 0,
                "contextId": null,
                "privateMode": false,
                "requestId": "b3c7d1a0-1111-2222-3333-444455556666",
            }),
        ),
        (
            "GeckoView:ContentPermission",
            json!({
                "uri": "https://video.example/",
                "thirdPartyOrigin": "https://video.example",
                "principal": "eyJ9",
                "perm": "autoplay-media-audible",
                "value": 0,
                "contextId": "gvctx776f726b",
                "privateMode": true,
                "requestId": "r2",
            }),
        ),
        (
            "GeckoView:MediaPermission",
            json!({"video": [camera()], "audio": null, "uri": "https://meet.example/"}),
        ),
        (
            "GeckoView:AndroidPermission",
            json!({"perms": ["android.permission.CAMERA"]}),
        ),
        (
            "GeckoView:MediaRecordingStatusChanged",
            json!({"devices": [{"type": "camera", "status": "recording"}]}),
        ),
    ];
    let decoded: Vec<WindowEvent> = cases
        .into_iter()
        .map(|(event, data)| window_event(event, data))
        .collect();

    let WindowEvent::ContentPermissionRequest(ContentPermissionRequest(geolocation)) = &decoded[0]
    else {
        panic!()
    };
    assert_eq!(geolocation.kind(), Some(PermissionKind::Geolocation));
    assert_eq!(
        geolocation.request_id.as_deref(),
        Some("b3c7d1a0-1111-2222-3333-444455556666")
    );

    let WindowEvent::ContentPermissionRequest(ContentPermissionRequest(autoplay)) = &decoded[1]
    else {
        panic!()
    };
    assert_eq!(autoplay.kind(), Some(PermissionKind::AutoplayAudible));

    let WindowEvent::MediaPermissionRequest(MediaPermissionRequest { video, audio, .. }) =
        &decoded[2]
    else {
        panic!()
    };
    assert_eq!(video.as_ref().unwrap()[0].raw_id, "cam-raw");
    assert_eq!(audio, &None);
}

#[test]
fn permission_kinds_follow_geckoview_names() {
    let cases = [
        (
            "desktop-notification",
            Some(PermissionKind::DesktopNotification),
            false,
        ),
        (
            "media-key-system-access",
            Some(PermissionKind::MediaKeySystemAccess),
            false,
        ),
        (
            "trackingprotection-pb",
            Some(PermissionKind::Tracking),
            true,
        ),
        (
            "3rdPartyFrameStorage^https://cdn.example",
            Some(PermissionKind::StorageAccess),
            false,
        ),
        (
            "local-network",
            Some(PermissionKind::LocalNetworkAccess),
            false,
        ),
        ("MediaManagerVideo", None, false),
    ];
    for (perm, kind, private) in cases {
        assert_eq!(PermissionKind::of(perm), kind, "{perm}");
        if let Some(kind) = kind.filter(|kind| *kind != PermissionKind::StorageAccess) {
            assert_eq!(kind.perm(private), perm);
        }
    }
}

#[test]
fn answers_permission_requests_in_the_shape_gecko_checks() {
    assert_eq!(
        answered::<ContentPermissionRequest>(&PermissionValue::Allow),
        json!(1)
    );
    assert_eq!(
        answered::<MediaPermissionRequest>(&MediaPermissionDecision::Deny),
        json!(false)
    );
    assert_eq!(
        answered::<MediaPermissionRequest>(&MediaPermissionDecision::Grant {
            video: Some("cam-raw".to_owned()),
            audio: None,
        }),
        json!({"video": "cam-raw", "audio": null})
    );
}

#[test]
fn manages_stored_permissions_through_the_runtime() {
    let stored = json!({
        "permissions": [{
            "uri": "https://example.org/",
            "principal": "eyJ9",
            "perm": "3rdPartyStorage^https://cdn.example",
            "value": 1,
            "contextId": "",
            "privateMode": false,
        }],
    });
    let list = gecko_reply::<GetAllPermissions>(stored);
    let permission: &ContentPermission = &list.permissions[0];
    assert_eq!(permission.storage_origin(), Some("https://cdn.example"));

    let cases: [(Value, Value); 3] = [
        (
            sent_data(&GetPermissionsByUri {
                uri: "https://example.org/".to_owned(),
                context_id: Some(SessionContextId::from_unsafe("work")),
                private_browsing_id: 0,
            }),
            json!({"uri": "https://example.org/", "contextId": "gvctx776f726b", "privateBrowsingId": 0}),
        ),
        (
            sent_data(&SetPermission {
                permission: permission.clone(),
                new_value: PermissionValue::Deny,
                allow_permanent_private_browsing: false,
            }),
            json!({
                "uri": "https://example.org/",
                "principal": "eyJ9",
                "perm": "3rdPartyStorage^https://cdn.example",
                "value": 1,
                "contextId": "",
                "privateMode": false,
                "newValue": 2,
                "allowPermanentPrivateBrowsing": false,
            }),
        ),
        (
            sent_data(&SetPermissionByUri {
                uri: "https://example.org/".to_owned(),
                context_id: None,
                private_id: 0,
                perm: "desktop-notification".to_owned(),
                new_value: PermissionValue::Allow,
            }),
            json!({"uri": "https://example.org/", "contextId": null, "privateId": 0, "perm": "desktop-notification", "newValue": 1}),
        ),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
}
