use rikka_gecko_protocol::geckoview::navigation::{LoadUri, OnLoadRequest, Stop};
use rikka_gecko_protocol::geckoview::session::{InitData, Module, Settings};
use rikka_gecko_protocol::message::{Close, Hello, Listen, Release, TargetError};
use rikka_gecko_protocol::{
    CallbackId, EmbedderMessage, Event, EventMessage, GeckoMessage, Reply, Target, WindowId,
};
use serde_json::{Value, json};

fn window(id: &str) -> WindowId {
    WindowId::new(id).unwrap()
}

#[test]
fn parses_what_the_bridge_sends() {
    let cases: [(&str, GeckoMessage); 7] = [
        (
            r#"{"type":"hello","protocol":1,"version":"158.0.1"}"#,
            GeckoMessage::Hello(Hello {
                protocol: 1,
                version: "158.0.1".to_owned(),
            }),
        ),
        (
            r#"{"type":"event","target":"window:a1","event":"GeckoView:PageStop","data":{"success":true}}"#,
            GeckoMessage::Event(EventMessage {
                target: Target::Window(window("a1")),
                event: "GeckoView:PageStop".to_owned(),
                data: Some(json!({"success": true})),
                callback: None,
            }),
        ),
        (
            r#"{"type":"event","target":"runtime","event":"GeckoView:WebExtension:OnDisabledProcessSpawning","callback":9007199254740991}"#,
            GeckoMessage::Event(EventMessage {
                target: Target::Runtime,
                event: "GeckoView:WebExtension:OnDisabledProcessSpawning".to_owned(),
                data: None,
                callback: CallbackId::new(9007199254740991),
            }),
        ),
        (
            r#"{"type":"reply","callback":12,"success":false,"data":"No listener for GeckoView:Stop"}"#,
            GeckoMessage::Reply(Reply {
                callback: CallbackId::new(12).unwrap(),
                success: false,
                data: Some(json!("No listener for GeckoView:Stop")),
            }),
        ),
        (
            r#"{"type":"release","callback":3}"#,
            GeckoMessage::Release(Release {
                callback: CallbackId::new(3).unwrap(),
            }),
        ),
        (
            r#"{"type":"ready","window":"a1"}"#,
            GeckoMessage::Ready(rikka_gecko_protocol::message::Ready {
                window: window("a1"),
            }),
        ),
        (
            r#"{"type":"closed","window":"a1"}"#,
            GeckoMessage::Closed(rikka_gecko_protocol::message::Closed {
                window: window("a1"),
            }),
        ),
    ];
    for (frame, expected) in cases {
        let parsed: GeckoMessage =
            serde_json::from_str(frame).unwrap_or_else(|error| panic!("{frame}: {error}"));
        assert_eq!(parsed, expected, "{frame}");
    }
}

#[test]
fn writes_what_the_bridge_expects() {
    let mut settings = Settings::default();
    settings.use_private_mode = true;
    let init = InitData::new(settings, [Module::GeckoViewNavigation]);

    let cases: [(EmbedderMessage, Value); 8] = [
        (
            EmbedderMessage::send(
                Target::Window(window("a1")),
                &LoadUri::new("https://example.org/"),
                None,
            )
            .unwrap(),
            json!({
                "type": "event",
                "target": "window:a1",
                "event": "GeckoView:LoadUri",
                "data": {
                    "uri": "https://example.org/",
                    "flags": 0,
                    "headerFilter": 1,
                    "textDirectiveUserActivation": false,
                    "appLinkLaunchType": 0,
                },
            }),
        ),
        (
            EmbedderMessage::send(Target::Window(window("a1")), &Stop, CallbackId::new(5)).unwrap(),
            json!({"type": "event", "target": "window:a1", "event": "GeckoView:Stop", "callback": 5}),
        ),
        (
            EmbedderMessage::answer::<OnLoadRequest>(
                CallbackId::new(2).unwrap(),
                &rikka_gecko_protocol::geckoview::navigation::LoadDecision::Deny,
            )
            .unwrap(),
            json!({"type": "reply", "callback": 2, "success": true, "data": true}),
        ),
        (
            EmbedderMessage::Reply(Reply::failure(CallbackId::new(2).unwrap(), "abort")),
            json!({"type": "reply", "callback": 2, "success": false, "data": "abort"}),
        ),
        (
            EmbedderMessage::Release(Release {
                callback: CallbackId::new(8).unwrap(),
            }),
            json!({"type": "release", "callback": 8}),
        ),
        (
            EmbedderMessage::Listen(Listen::new(
                Target::port(4),
                &["GeckoView:WebExtension:PortMessage"],
            )),
            json!({"type": "listen", "target": "named:port:4", "events": ["GeckoView:WebExtension:PortMessage"]}),
        ),
        (
            EmbedderMessage::Close(Close {
                window: window("a1"),
            }),
            json!({"type": "close", "window": "a1"}),
        ),
        (
            init.open(window("b2")).unwrap(),
            json!({
                "type": "open",
                "window": "b2",
                "private": true,
                "init": {
                    "settings": {
                        "chromeUri": null,
                        "screenId": 0,
                        "useTrackingProtection": false,
                        "usePrivateMode": true,
                        "suspendMediaWhenInactive": false,
                        "allowJavascript": true,
                        "fullAccessibilityTree": false,
                        "isExtensionPopup": false,
                        "userAgentMode": 0,
                        "userAgentOverride": null,
                        "viewportMode": 0,
                        "displayMode": 0,
                        "sessionContextId": null,
                        "unsafeSessionContextId": null,
                    },
                    "modules": Module::ALL
                        .iter()
                        .map(|module| {
                            let name = serde_json::to_value(module).unwrap();
                            (name.as_str().unwrap().to_owned(), json!(*module == Module::GeckoViewNavigation))
                        })
                        .collect::<serde_json::Map<_, _>>(),
                },
            }),
        ),
    ];
    for (message, expected) in cases {
        assert_eq!(serde_json::to_value(&message).unwrap(), expected);
    }
}

#[test]
fn unlisten_shares_the_listen_shape() {
    let message = EmbedderMessage::Unlisten(Listen::new(Target::Runtime, &["GeckoView:SetLocale"]));
    assert_eq!(
        serde_json::to_value(message).unwrap(),
        json!({"type": "unlisten", "target": "runtime", "events": ["GeckoView:SetLocale"]})
    );
}

#[test]
fn targets_parse_and_format() {
    let cases: [(&str, Result<Target, TargetError>); 6] = [
        ("runtime", Ok(Target::Runtime)),
        ("named:port:17", Ok(Target::port(17))),
        ("window:3f2a", Ok(Target::Window(window("3f2a")))),
        ("window:", Err(TargetError::EmptyWindow)),
        ("named:", Err(TargetError::EmptyName)),
        ("tab:1", Err(TargetError::Unknown("tab:1".to_owned()))),
    ];
    for (text, expected) in cases {
        let parsed = text.parse::<Target>();
        assert_eq!(parsed, expected, "{text}");
        if let Ok(target) = parsed {
            assert_eq!(target.to_string(), text);
        }
    }
}

#[test]
fn callback_ids_stay_within_javascript_safe_integers() {
    for (id, valid) in [
        (0, false),
        (1, true),
        ((1 << 53) - 1, true),
        (1 << 53, false),
    ] {
        assert_eq!(CallbackId::new(id).is_some(), valid, "{id}");
        let parsed = serde_json::from_value::<CallbackId>(json!(id));
        assert_eq!(parsed.is_ok(), valid, "{id}");
    }
    assert_eq!(CallbackId::new(CallbackId::MAX).unwrap().next(), None);
}

#[test]
fn rejects_malformed_frames() {
    let frames = [
        r#"{"type":"event","target":"window:","event":"GeckoView:PageStop"}"#,
        r#"{"type":"reply","callback":0,"success":true}"#,
        r#"{"type":"open","window":"a1"}"#,
        r#"{"type":"teleport"}"#,
        r#"{"target":"runtime","event":"x"}"#,
    ];
    for frame in frames {
        assert!(
            serde_json::from_str::<GeckoMessage>(frame).is_err(),
            "{frame}"
        );
    }
    assert!(
        serde_json::from_str::<EmbedderMessage>(r#"{"type":"open","window":"","init":{}}"#)
            .is_err()
    );
}

#[test]
fn typed_payloads_round_trip_through_event_messages() {
    let message = EventMessage::new(
        Target::Window(window("a1")),
        &LoadUri::new("about:blank"),
        None,
    )
    .unwrap();
    assert!(message.is::<LoadUri>());
    assert_eq!(message.event, LoadUri::NAME);
    assert_eq!(
        message.payload::<LoadUri>().unwrap(),
        LoadUri::new("about:blank")
    );
}

#[test]
fn failed_replies_keep_the_error_value() {
    let reply = Reply::failure(CallbackId::FIRST, "No listener for GeckoView:OnLoadRequest");
    assert_eq!(
        reply.outcome::<OnLoadRequest>().unwrap(),
        Err(json!("No listener for GeckoView:OnLoadRequest"))
    );
}
