use rikka_gecko_protocol::geckoview::navigation::{ErrorClass, OpenWhere};
use rikka_gecko_protocol::inbound::WindowEvent;
use rikka_gecko_protocol::rikka::capture::{CaptureState, StopCapture};
use rikka_gecko_protocol::rikka::certificates::AllowCertificate;
use rikka_gecko_protocol::rikka::editing::{
    AddToDictionary, Edit, EditCommand, ReplaceMisspelling, Spelling,
};
use rikka_gecko_protocol::rikka::view::{Download, PermitUnload};
use serde_json::{Value, json};

use crate::support::{gecko_reply, sent_data, window_event};

#[test]
fn decodes_view_events_as_the_components_build_them() {
    let WindowEvent::OpenLink(link) = window_event(
        "Rikka:View:OpenLink",
        json!({"uri": "https://example.org/next", "referrer": "https://example.org/", "where": 5}),
    ) else {
        panic!()
    };
    assert_eq!(link.target, OpenWhere::NewTabBackground);
    assert_eq!(link.referrer.as_deref(), Some("https://example.org/"));

    let WindowEvent::CertificateError(failure) = window_event(
        "Rikka:View:CertificateError",
        json!({"uri": "https://self-signed.example:8443/", "host": "self-signed.example", "port": 8443,
               "certificate": "MIIB", "overridable": true}),
    ) else {
        panic!()
    };
    assert_eq!((failure.port, failure.overridable), (8443, true));

    let WindowEvent::RikkaContextMenu(menu) = window_event(
        "Rikka:View:ContextMenu",
        json!({
            "screenX": 40, "screenY": 60.5, "baseUri": "https://example.org/", "uri": null,
            "title": null, "alt": null, "elementType": "none", "elementSrc": null,
            "textContent": null, "linkText": null, "selection": "teh",
            "editable": true, "spelling": {"misspelling": "teh", "suggestions": ["the", "tech"]},
        }),
    ) else {
        panic!()
    };
    assert_eq!(menu.menu.element_type, "none");
    assert_eq!(
        menu.spelling,
        Some(Spelling {
            misspelling: "teh".to_owned(),
            suggestions: vec!["the".to_owned(), "tech".to_owned()],
        })
    );

    let WindowEvent::CaptureChanged(capture) = window_event(
        "Rikka:View:Capture",
        json!({"camera": "muted", "microphone": "live", "screen": "off"}),
    ) else {
        panic!()
    };
    assert_eq!(
        (capture.camera, capture.microphone, capture.screen),
        (CaptureState::Muted, CaptureState::Live, CaptureState::Off)
    );
}

#[test]
fn sends_view_commands_as_the_components_read_them() {
    let cases: [(Value, Value); 7] = [
        (
            sent_data(&Edit {
                command: EditCommand::PastePlainText,
            }),
            json!({"command": "pastePlainText"}),
        ),
        (
            sent_data(&Edit {
                command: EditCommand::SelectAll,
            }),
            json!({"command": "selectAll"}),
        ),
        (
            sent_data(&ReplaceMisspelling {
                suggestion: "the".to_owned(),
            }),
            json!({"suggestion": "the"}),
        ),
        (sent_data(&AddToDictionary), Value::Null),
        (
            sent_data(&StopCapture { only_screen: true }),
            json!({"onlyScreen": true}),
        ),
        (
            sent_data(&AllowCertificate {
                host: "self-signed.example".to_owned(),
                port: 8443,
                certificate: "MIIB".to_owned(),
                permanent: false,
            }),
            json!({"host": "self-signed.example", "port": 8443, "certificate": "MIIB", "permanent": false}),
        ),
        (
            sent_data(&Download {
                uri: "https://example.org/cat.png".to_owned(),
            }),
            json!({"uri": "https://example.org/cat.png"}),
        ),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
    assert_eq!(sent_data(&PermitUnload), Value::Null);
    assert!(!gecko_reply::<PermitUnload>(json!(false)));
}

#[test]
fn reads_the_nss_error_class_of_load_errors() {
    let cases = [
        (0, None),
        (1, Some(ErrorClass::SslProtocol)),
        (2, Some(ErrorClass::BadCertificate)),
    ];
    for (code, class) in cases {
        assert_eq!(ErrorClass::from_code(code), class);
    }
}
