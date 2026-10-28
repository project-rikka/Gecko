use rikka_gecko_protocol::geckoview::webextension::browsing_data::{
    BrowsingDataSettings, BrowsingDataType, BrowsingDataTypes, GetBrowsingDataSettings,
    RemovalOptions,
};
use rikka_gecko_protocol::geckoview::webextension::controller::{
    Enable, EnableSource, Install, InstallFailure, List, Update,
};
use rikka_gecko_protocol::geckoview::webextension::download::{
    Download, DownloadChanged, DownloadInfo, DownloadState,
};
use rikka_gecko_protocol::geckoview::webextension::extension::{DisabledFlag, ExtensionFlags};
use rikka_gecko_protocol::geckoview::webextension::messaging::{
    PortDisconnect, PortMessageFromApp, SenderEnvironment,
};
use rikka_gecko_protocol::geckoview::webextension::prompt::{InstallPrompt, InstallPromptResponse};
use rikka_gecko_protocol::geckoview::webextension::tab::{NewTab, SetTabActive};
use rikka_gecko_protocol::inbound::{PortEvent, RuntimeEvent, WindowEvent};
use rikka_gecko_protocol::{CallbackId, GeckoMessage, Target};
use serde_json::{Value, json};

use crate::support::{
    answered, gecko_reply, port_event, runtime_event, sent, sent_data, window_event,
};

fn exported() -> Value {
    json!({
        "webExtensionId": "uBlock0@raymondhill.net",
        "locationURI": "",
        "isBuiltIn": false,
        "webExtensionFlags": 0,
        "metaData": {
            "amoListingURL": "https://addons.mozilla.org/firefox/addon/ublock-origin/",
            "averageRating": 4.8,
            "baseURL": "moz-extension://0d9f/",
            "blocklistState": 0,
            "creatorName": "Raymond Hill",
            "creatorURL": null,
            "description": "Finally, an efficient blocker.",
            "disabledFlags": ["userDisabled"],
            "downloadUrl": null,
            "enabled": false,
            "fullDescription": null,
            "homepageURL": "https://github.com/gorhill/uBlock",
            "icons": {"48": "moz-extension://0d9f/img/icon_64.png"},
            "incognito": "spanning",
            "isRecommended": true,
            "name": "uBlock Origin",
            "openOptionsPageInTab": true,
            "optionsPageURL": "moz-extension://0d9f/dashboard.html",
            "privateBrowsingAllowed": false,
            "reviewCount": 1200,
            "reviewURL": null,
            "signedState": 2,
            "temporary": false,
            "updateDate": "2026-10-01T00:00:00.000Z",
            "version": "1.66.0",
            "requiredPermissions": ["storage", "webRequest", "webRequestBlocking"],
            "requiredOrigins": ["<all_urls>"],
            "requiredDataCollectionPermissions": [],
            "optionalPermissions": [],
            "optionalOrigins": [],
            "optionalDataCollectionPermissions": [],
            "grantedOptionalPermissions": [],
            "grantedOptionalOrigins": [],
            "grantedOptionalDataCollectionPermissions": [],
        },
    })
}

fn sender() -> Value {
    json!({"id": "uBlock0@raymondhill.net", "envType": "content_child", "url": "https://example.org/", "frameId": 0})
}

#[test]
fn decodes_runtime_extension_events_as_the_module_builds_them() {
    let cases: [(&str, Value); 10] = [
        (
            "GeckoView:WebExtension:OnInstalled",
            json!({"extension": exported()}),
        ),
        (
            "GeckoView:WebExtension:OnReady",
            json!({"extension": exported()}),
        ),
        (
            "GeckoView:WebExtension:OnInstallationFailed",
            json!({"extension": null, "addonName": "Broken", "error": -2}),
        ),
        (
            "GeckoView:WebExtension:InstallPrompt",
            json!({
                "extension": exported(),
                "permissions": ["webRequest"],
                "origins": ["<all_urls>"],
                "dataCollectionPermissions": ["technicalAndInteraction"],
            }),
        ),
        (
            "GeckoView:WebExtension:OptionalPrompt",
            json!({
                "extensionId": "uBlock0@raymondhill.net",
                "permissions": {"permissions": ["tabs"], "origins": [], "data_collection": []},
            }),
        ),
        (
            "GeckoView:BrowserAction:Update",
            json!({
                "action": {
                    "title": "uBlock",
                    "icon": {"16": "moz-extension://0d9f/img/16.png"},
                    "popup": "moz-extension://0d9f/popup.html",
                    "badgeText": "3",
                    "badgeBackgroundColor": [102, 102, 102, 255],
                    "badgeTextColor": [255, 255, 255, 255],
                    "enabled": true,
                    "patternMatching": false,
                },
                "aTabId": null,
                "extensionId": "uBlock0@raymondhill.net",
            }),
        ),
        (
            "GeckoView:WebExtension:NewTab",
            json!({
                "extensionId": "uBlock0@raymondhill.net",
                "createProperties": {"active": true, "url": "moz-extension://0d9f/dashboard.html"},
                "newSessionId": "9b1f2c3d4e5f60718293a4b5c6d7e8f9",
            }),
        ),
        (
            "GeckoView:WebExtension:Download",
            json!({
                "options": {"url": "https://example.org/a.zip", "filename": "a.zip", "conflictAction": "uniquify", "saveAs": false},
                "extensionId": "uBlock0@raymondhill.net",
            }),
        ),
        (
            "GeckoView:BrowsingData:Clear",
            json!({"dataType": "formData", "since": 0, "extensionId": "uBlock0@raymondhill.net"}),
        ),
        (
            "GeckoView:WebExtension:Message",
            json!({
                "sender": {"id": "uBlock0@raymondhill.net", "envType": "addon_child", "url": "moz-extension://0d9f/background.html"},
                "data": {"kind": "ping"},
                "extensionId": "uBlock0@raymondhill.net",
                "nativeApp": "rikka",
            }),
        ),
    ];
    let decoded: Vec<RuntimeEvent> = cases
        .into_iter()
        .map(|(event, data)| runtime_event(event, data))
        .collect();

    let RuntimeEvent::OnInstalled(installed) = &decoded[0] else {
        panic!()
    };
    let metadata = &installed.extension.meta_data;
    assert_eq!(metadata.disabled_flags, [DisabledFlag::UserDisabled]);
    assert_eq!(metadata.required_origins, ["<all_urls>"]);
    assert_eq!(
        installed.extension.web_extension_flags,
        ExtensionFlags::NONE
    );

    let RuntimeEvent::OnInstallationFailed(failed) = &decoded[2] else {
        panic!()
    };
    assert_eq!((failed.extension.is_none(), failed.error), (true, -2));

    let RuntimeEvent::OptionalPrompt(optional) = &decoded[4] else {
        panic!()
    };
    assert_eq!(optional.permissions.permissions, ["tabs"]);

    let RuntimeEvent::BrowserActionUpdate(update) = &decoded[5] else {
        panic!()
    };
    assert_eq!(update.0.tab_id, None);
    assert_eq!(
        update.0.action.badge_background_color,
        Some([102.0, 102.0, 102.0, 255.0])
    );

    let RuntimeEvent::NewTab(NewTab {
        new_session_id,
        create_properties,
        ..
    }) = &decoded[6]
    else {
        panic!()
    };
    assert_eq!(new_session_id.as_str(), "9b1f2c3d4e5f60718293a4b5c6d7e8f9");
    assert_eq!(create_properties.active, Some(true));

    let RuntimeEvent::ExtensionDownload(Download { options, .. }) = &decoded[7] else {
        panic!()
    };
    assert_eq!(options.filename.as_deref(), Some("a.zip"));

    let RuntimeEvent::ClearBrowsingData(clear) = &decoded[8] else {
        panic!()
    };
    assert_eq!(clear.data_type, BrowsingDataType::FormData);

    let RuntimeEvent::NativeMessage(message) = &decoded[9] else {
        panic!()
    };
    assert_eq!(message.sender.env_type, SenderEnvironment::AddonChild);
    assert!(message.sender.is_top_level());
}

#[test]
fn decodes_session_and_port_extension_events() {
    let cases: [(&str, Value); 4] = [
        (
            "GeckoView:WebExtension:Connect",
            json!({"sender": sender(), "data": {}, "portId": 12, "extensionId": "uBlock0@raymondhill.net", "nativeApp": "rikka"}),
        ),
        (
            "GeckoView:PageAction:OpenPopup",
            json!({"action": {"title": "Reader"}, "popupUri": "moz-extension://0d9f/p.html", "aTabId": 10042, "extensionId": "x@y"}),
        ),
        (
            "GeckoView:WebExtension:UpdateTab",
            json!({"extensionId": "x@y", "updateProperties": {"muted": true}}),
        ),
        (
            "GeckoView:WebExtension:CloseTab",
            json!({"extensionId": "x@y"}),
        ),
    ];
    for (event, data) in cases {
        window_event(event, data);
    }

    let WindowEvent::Connect(connect) = window_event(
        "GeckoView:WebExtension:Connect",
        json!({"sender": sender(), "data": {}, "portId": 12, "extensionId": "x@y", "nativeApp": "rikka"}),
    ) else {
        panic!()
    };
    let port = Target::port(connect.port_id);
    assert_eq!(port.to_string(), "named:port:12");

    let PortEvent::PortMessage(message) = port_event(
        "GeckoView:WebExtension:PortMessage",
        json!({"data": {"n": 1}}),
    ) else {
        panic!()
    };
    assert_eq!(message.data, json!({"n": 1}));
    assert!(matches!(
        port_event("GeckoView:WebExtension:Disconnect", json!({})),
        PortEvent::PortDisconnected(_)
    ));

    assert_eq!(
        sent(
            port.clone(),
            &PortMessageFromApp {
                message: json!({"reply": true})
            }
        ),
        json!({
            "type": "event",
            "target": "named:port:12",
            "event": "GeckoView:WebExtension:PortMessageFromApp",
            "data": {"message": {"reply": true}},
            "callback": 1,
        })
    );
    assert_eq!(
        sent_data(&PortDisconnect { port_id: 12 }),
        json!({"portId": 12})
    );
    assert_eq!(
        sent_data(&SetTabActive { active: true }),
        json!({"active": true})
    );
}

#[test]
fn drives_the_extension_controller() {
    let cases: [(Value, Value); 3] = [
        (
            sent_data(&Install {
                location_uri: "https://addons.mozilla.org/firefox/downloads/file/1/ublock.xpi"
                    .to_owned(),
                install_id: "install-1".to_owned(),
                install_method: Some("manager".to_owned()),
            }),
            json!({
                "locationUri": "https://addons.mozilla.org/firefox/downloads/file/1/ublock.xpi",
                "installId": "install-1",
                "installMethod": "manager",
            }),
        ),
        (
            sent_data(&Enable {
                web_extension_id: "x@y".to_owned(),
                source: EnableSource::User,
            }),
            json!({"webExtensionId": "x@y", "source": "user"}),
        ),
        (sent_data(&List), Value::Null),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }

    let listed = gecko_reply::<List>(json!({"extensions": [exported()]}));
    assert_eq!(
        listed.extensions[0].meta_data.name.as_deref(),
        Some("uBlock Origin")
    );
    assert_eq!(gecko_reply::<Update>(Value::Null), None);

    let failure = json!({"type": "reply", "callback": 2, "success": false, "data": {"installError": -1, "state": 3}});
    let GeckoMessage::Reply(reply) = serde_json::from_value(failure).unwrap() else {
        panic!()
    };
    assert_eq!(reply.callback, CallbackId::new(2).unwrap());
    let error = reply.outcome::<Install>().unwrap().unwrap_err();
    assert_eq!(
        serde_json::from_value::<InstallFailure>(error).unwrap(),
        InstallFailure {
            install_error: -1,
            state: 3,
            cancelled_by_user: None
        }
    );
}

#[test]
fn answers_extension_requests_in_the_shape_the_module_reads() {
    assert_eq!(
        answered::<InstallPrompt>(&InstallPromptResponse {
            allow: true,
            private_browsing_allowed: false,
            is_technical_and_interaction_data_granted: true,
        }),
        json!({"allow": true, "privateBrowsingAllowed": false, "isTechnicalAndInteractionDataGranted": true})
    );
    assert_eq!(
        answered::<Download>(&DownloadInfo {
            id: 1,
            bytes_received: 0,
            can_resume: false,
            exists: false,
            filename: "a.zip".to_owned(),
            file_size: -1,
            mime: "application/zip".to_owned(),
            paused: false,
            referrer: None,
            start_time: "1761436800000".to_owned(),
            state: DownloadState::InProgress,
            total_bytes: -1,
            end_time: None,
            error: None,
            estimated_end_time: None,
        }),
        json!({
            "id": 1,
            "bytesReceived": 0,
            "canResume": false,
            "exists": false,
            "filename": "a.zip",
            "fileSize": -1,
            "mime": "application/zip",
            "paused": false,
            "referrer": null,
            "startTime": "1761436800000",
            "state": 0,
            "totalBytes": -1,
        })
    );
    assert_eq!(
        sent_data(&DownloadChanged {
            download_item_id: 1,
            filename: "a.zip".to_owned(),
            mime: "application/zip".to_owned(),
            start_time: "1761436800000".to_owned(),
            end_time: Some("1761436805000".to_owned()),
            state: DownloadState::Complete,
            can_resume: false,
            paused: false,
            error: None,
            total_bytes: 2048,
            file_size: 2048,
            exists: true,
        }),
        json!({
            "downloadItemId": 1,
            "filename": "a.zip",
            "mime": "application/zip",
            "startTime": "1761436800000",
            "endTime": "1761436805000",
            "state": 2,
            "canResume": false,
            "paused": false,
            "totalBytes": 2048,
            "fileSize": 2048,
            "exists": true,
        })
    );
    let all = BrowsingDataTypes {
        cache: true,
        cookies: true,
        downloads: true,
        form_data: true,
        history: true,
        local_storage: true,
        passwords: true,
    };
    assert_eq!(
        answered::<GetBrowsingDataSettings>(&BrowsingDataSettings {
            options: RemovalOptions { since: 0 },
            data_to_remove: BrowsingDataTypes {
                history: true,
                ..BrowsingDataTypes::default()
            },
            data_removal_permitted: all,
        })["dataToRemove"],
        json!({
            "cache": false,
            "cookies": false,
            "downloads": false,
            "formData": false,
            "history": true,
            "localStorage": false,
            "passwords": false,
        })
    );
}
