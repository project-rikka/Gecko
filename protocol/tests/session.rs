use rikka_gecko_protocol::geckoview::session::{
    DisplayMode, InitData, Module, SessionContextId, Settings, UpdateModuleState, UpdateSettings,
    UserAgentMode, ViewportMode,
};
use serde_json::json;

use crate::support::sent_data;

#[test]
fn derives_session_context_ids_like_java_big_integer_hex() {
    let cases = [
        ("", "gvctx0"),
        ("a", "gvctx61"),
        ("abc", "gvctx616263"),
        ("\u{1}", "gvctx1"),
        ("\0a", "gvctx61"),
        ("é", "gvctx-3c57"),
        ("éa", "gvctx-3c569f"),
    ];
    for (context, expected) in cases {
        assert_eq!(
            SessionContextId::from_unsafe(context).as_str(),
            expected,
            "{context:?}"
        );
        let parsed: SessionContextId = serde_json::from_value(json!(expected)).unwrap();
        assert_eq!(parsed.as_str(), expected);
    }
    for invalid in ["", "gvctx", "gvctxZZ", "context-1"] {
        assert!(
            serde_json::from_value::<SessionContextId>(json!(invalid)).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn settings_update_carries_every_key_the_modules_read() {
    let mut settings = Settings::default();
    settings.user_agent_mode = UserAgentMode::Desktop;
    settings.viewport_mode = ViewportMode::Desktop;
    settings.display_mode = DisplayMode::Standalone;
    settings.user_agent_override = Some("Rikka".to_owned());
    settings.set_context_id(Some("work"));

    let data = sent_data(&UpdateSettings(settings.clone()));
    assert_eq!(
        data,
        json!({
            "chromeUri": null,
            "screenId": 0,
            "useTrackingProtection": false,
            "usePrivateMode": false,
            "suspendMediaWhenInactive": false,
            "allowJavascript": true,
            "fullAccessibilityTree": false,
            "isExtensionPopup": false,
            "userAgentMode": 1,
            "userAgentOverride": "Rikka",
            "viewportMode": 1,
            "displayMode": 2,
            "sessionContextId": "gvctx776f726b",
            "unsafeSessionContextId": "work",
        })
    );
    assert_eq!(settings.context_id(), Some("work"));

    assert_eq!(
        sent_data(&UpdateModuleState {
            module: Module::GeckoViewProgress,
            enabled: true,
        }),
        json!({"module": "GeckoViewProgress", "enabled": true})
    );
}

#[test]
fn init_data_round_trips() {
    let init = InitData::new(Settings::default(), Module::ALL);
    let value = serde_json::to_value(&init).unwrap();
    assert_eq!(value["modules"]["GeckoViewContent"], json!(true));
    assert_eq!(serde_json::from_value::<InitData>(value).unwrap(), init);
}
