use rikka_gecko_protocol::geckoview::autocomplete::{LoginEntry, SelectHint};
use rikka_gecko_protocol::geckoview::prompt::{
    AllowResponse, AuthFlags, AuthMode, AuthResponse, ButtonResponse, ButtonTitle, ButtonType,
    CaptureType, ChoiceMode, ChoiceResponse, DateTimeMode, FileMode, FileResponse, PromptKind,
    PromptResponse, Selection, SelectionResponse, ShareResponse, ShareResult, ShowPrompt,
    TargetResponse, TextResponse,
};
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{answered, window_event};

fn prompt(fields: Value) -> PromptKind {
    let mut prompt = fields;
    prompt["id"] = json!("6c1f7a2e-0b8e-4d77-9d3c-1a2b3c4d5e6f");
    match window_event("GeckoView:Prompt", json!({"prompt": prompt})) {
        WindowEvent::ShowPrompt(ShowPrompt { prompt }) => prompt.kind,
        other => panic!("{other:?}"),
    }
}

type Expectation = fn(&PromptKind) -> bool;

#[test]
fn decodes_every_prompt_type_as_the_prompters_build_it() {
    let cases: Vec<(Value, Expectation)> = vec![
        (
            json!({"type": "alert", "title": null, "msg": "Hello", "hasCheck": false}),
            |kind| matches!(kind, PromptKind::Alert(alert) if alert.msg.as_deref() == Some("Hello")),
        ),
        (
            json!({
                "type": "button",
                "title": "Leave?",
                "msg": "Really?",
                "hasCheck": true,
                "checkMsg": "Don't ask again",
                "checkValue": false,
                "btnTitle": ["ok", null, "cancel"],
                "btnCustomTitle": [null, null, null],
            }),
            |kind| {
                matches!(kind, PromptKind::Button(button)
                if button.btn_title == [Some(ButtonTitle::Ok), None, Some(ButtonTitle::Cancel)]
                && button.check.has_check)
            },
        ),
        (
            json!({"type": "text", "title": "Name", "msg": "Your name", "hasCheck": false, "value": "Rikka"}),
            |kind| matches!(kind, PromptKind::Text(text) if text.value.as_deref() == Some("Rikka")),
        ),
        (
            json!({
                "type": "auth",
                "title": null,
                "msg": "https://example.org is requesting your username and password.",
                "mode": "auth",
                "options": {
                    "flags": 1,
                    "uri": "https://example.org/private",
                    "level": 2,
                    "username": "",
                    "password": "",
                },
            }),
            |kind| {
                matches!(kind, PromptKind::Auth(auth)
                if auth.mode == AuthMode::Auth && auth.options.flags.contains(AuthFlags::HOST))
            },
        ),
        (
            json!({
                "type": "auth",
                "title": "Password",
                "msg": "Enter password",
                "mode": "password",
                "options": {"flags": 8, "password": ""},
            }),
            |kind| {
                matches!(kind, PromptKind::Auth(auth)
                if auth.options.flags.contains(AuthFlags::ONLY_PASSWORD) && auth.options.uri.is_none())
            },
        ),
        (
            json!({
                "type": "choice",
                "mode": "single",
                "choices": [
                    {"id": "0", "disabled": false, "label": "Red", "selected": true},
                    {"id": "1", "disabled": false, "label": "Fruits", "items": [
                        {"id": "2", "disabled": true, "label": "Apple", "selected": false},
                    ]},
                    {"id": "3", "disabled": false, "separator": true},
                ],
            }),
            |kind| {
                matches!(kind, PromptKind::Choice(choice)
                if choice.mode == ChoiceMode::Single
                && choice.choices[1].items.as_ref().unwrap()[0].disabled
                && choice.choices[2].separator)
            },
        ),
        (
            json!({"type": "color", "title": "Pick", "value": "#ff0000", "predefinedValues": ["#000000"]}),
            |kind| matches!(kind, PromptKind::Color(color) if color.value.as_deref() == Some("#ff0000")),
        ),
        (
            json!({"type": "datetime", "mode": "datetime-local", "value": "", "min": "", "max": "", "step": ""}),
            |kind| matches!(kind, PromptKind::DateTime(datetime) if datetime.mode == DateTimeMode::DatetimeLocal),
        ),
        (
            json!({"type": "file", "title": "Upload", "mode": "multiple", "mimeTypes": ["image/*", ".pdf"], "capture": 0}),
            |kind| {
                matches!(kind, PromptKind::File(file)
                if file.mode == FileMode::Multiple && file.capture == CaptureType::None && file.mime_types.len() == 2)
            },
        ),
        (
            json!({"type": "folderUpload", "directoryName": "photos"}),
            |kind| matches!(kind, PromptKind::FolderUpload(folder) if folder.directory_name == "photos"),
        ),
        (
            json!({"type": "popup", "targetUri": "https://ads.example/"}),
            |kind| matches!(kind, PromptKind::Popup(popup) if popup.target_uri.is_some()),
        ),
        (
            json!({"type": "redirect", "targetUri": "https://elsewhere.example/"}),
            |kind| matches!(kind, PromptKind::Redirect(_)),
        ),
        (json!({"type": "repost"}), |kind| {
            matches!(kind, PromptKind::Repost {})
        }),
        (json!({"type": "beforeUnload"}), |kind| {
            matches!(kind, PromptKind::BeforeUnload {})
        }),
        (
            json!({"type": "share", "title": "Look", "text": null, "uri": "https://example.org/"}),
            |kind| matches!(kind, PromptKind::Share(share) if share.uri.is_some()),
        ),
        (
            json!({
                "type": "Autocomplete:Save:Login",
                "hint": 2,
                "logins": [{
                    "origin": "https://example.org",
                    "formActionOrigin": "https://example.org",
                    "httpRealm": null,
                    "username": "rikka",
                    "password": "hunter2",
                    "guid": null,
                    "timeCreated": null,
                    "timeLastUsed": null,
                    "timePasswordChanged": null,
                    "timesUsed": null,
                }],
            }),
            |kind| matches!(kind, PromptKind::LoginSave(save) if save.logins[0].username == "rikka"),
        ),
        (
            json!({
                "type": "Autocomplete:Select:Login",
                "options": [{
                    "value": {"origin": "https://example.org", "username": "rikka", "password": "x", "guid": "{1}"},
                    "hint": 8,
                }],
            }),
            |kind| {
                matches!(kind, PromptKind::LoginSelect(select)
                if select.options[0].hint == SelectHint::MATCHING_ORIGIN)
            },
        ),
        (
            json!({
                "type": "Autocomplete:Select:CreditCard",
                "options": [{"value": {"name": "R", "number": "************1111", "expMonth": "4", "expYear": "2030", "type": "visa", "guid": "c1"}, "hint": 0}],
            }),
            |kind| matches!(kind, PromptKind::CreditCardSelect(select) if select.options[0].value.card_type == "visa"),
        ),
        (
            json!({
                "type": "Autocomplete:Select:Address",
                "options": [{"value": {"name": "R", "streetAddress": "1 Main St", "postalCode": "100", "country": "JP"}, "hint": 0}],
            }),
            |kind| matches!(kind, PromptKind::AddressSelect(select) if select.options[0].value.country == "JP"),
        ),
    ];
    for (fields, check) in cases {
        let kind = prompt(fields.clone());
        assert!(check(&kind), "{fields} decoded as {kind:?}");
    }
}

#[test]
fn encodes_prompt_answers_the_prompters_read() {
    let cases: [(Option<PromptResponse>, Value); 10] = [
        (None, Value::Null),
        (
            Some(PromptResponse::Button(ButtonResponse {
                button: ButtonType::Negative,
                check_value: Some(true),
            })),
            json!({"button": 2, "checkValue": true}),
        ),
        (
            Some(PromptResponse::Text(TextResponse {
                text: "Rikka".to_owned(),
                check_value: None,
            })),
            json!({"text": "Rikka"}),
        ),
        (
            Some(PromptResponse::Auth(AuthResponse {
                username: Some("me".to_owned()),
                password: "pw".to_owned(),
            })),
            json!({"username": "me", "password": "pw"}),
        ),
        (
            Some(PromptResponse::Choice(ChoiceResponse {
                choices: vec!["0".to_owned()],
            })),
            json!({"choices": ["0"]}),
        ),
        (
            Some(PromptResponse::File(FileResponse {
                files: vec!["/home/rikka/a.png".to_owned()],
                files_in_webkit_directory: None,
            })),
            json!({"files": ["/home/rikka/a.png"]}),
        ),
        (
            Some(PromptResponse::Allow(AllowResponse { allow: true })),
            json!({"allow": true}),
        ),
        (
            Some(PromptResponse::Target(TargetResponse { response: false })),
            json!({"response": false}),
        ),
        (
            Some(PromptResponse::Share(ShareResponse {
                response: ShareResult::Abort,
            })),
            json!({"response": 2}),
        ),
        (
            Some(PromptResponse::Login(SelectionResponse {
                selection: Selection {
                    value: LoginEntry {
                        origin: "https://example.org".to_owned(),
                        username: "rikka".to_owned(),
                        password: "pw".to_owned(),
                        ..LoginEntry::default()
                    },
                    hint: SelectHint::NONE,
                },
            })),
            json!({
                "selection": {
                    "value": {
                        "guid": null,
                        "origin": "https://example.org",
                        "formActionOrigin": null,
                        "httpRealm": null,
                        "username": "rikka",
                        "password": "pw",
                    },
                    "hint": 0,
                },
            }),
        ),
    ];
    for (response, expected) in cases {
        assert_eq!(answered::<ShowPrompt>(&response), expected);
    }
}

#[test]
fn prompt_dismissal_and_update_identify_the_prompt() {
    let WindowEvent::DismissPrompt(dismiss) =
        window_event("GeckoView:Prompt:Dismiss", json!({"id": "p-1"}))
    else {
        panic!()
    };
    assert_eq!(dismiss.id.0, "p-1");

    let WindowEvent::UpdatePrompt(update) = window_event(
        "GeckoView:Prompt:Update",
        json!({"prompt": {"type": "choice", "mode": "multiple", "choices": [], "id": "p-1"}}),
    ) else {
        panic!()
    };
    assert_eq!(update.prompt.id.0, "p-1");
}
