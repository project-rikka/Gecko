use rikka_gecko_protocol::geckoview::autocomplete::{
    AddressStructure, FetchLogins, GetAddressStructure, LoginEntry, UsedFields,
};
use rikka_gecko_protocol::inbound::RuntimeEvent;
use serde_json::{Value, json};

use crate::support::{answered, gecko_reply, runtime_event, sent_data};

#[test]
fn decodes_storage_delegate_events_as_geckoview_autocomplete_sends_them() {
    let cases: [(&str, Value); 5] = [
        (
            "GeckoView:Autocomplete:Fetch:Login",
            json!({"domain": "example.org"}),
        ),
        ("GeckoView:Autocomplete:Fetch:Address", Value::Null),
        (
            "GeckoView:Autocomplete:Save:Login",
            json!({"login": {
                "origin": "https://example.org",
                "formActionOrigin": "https://example.org",
                "httpRealm": null,
                "username": "rikka",
                "password": "pw",
                "guid": null,
                "timeCreated": 1761436800000i64,
                "timeLastUsed": 1761436800000i64,
                "timePasswordChanged": 1761436800000i64,
                "timesUsed": 1,
            }}),
        ),
        (
            "GeckoView:Autocomplete:Save:CreditCard",
            json!({"creditCard": {"name": "R", "number": "4111111111111111", "expMonth": "4", "expYear": "2030", "type": "visa", "guid": null}}),
        ),
        (
            "GeckoView:Autocomplete:Used:Login",
            json!({"usedFields": 1, "login": {"origin": "https://example.org", "username": "rikka", "password": "pw", "guid": "{g}"}}),
        ),
    ];
    let decoded: Vec<RuntimeEvent> = cases
        .into_iter()
        .map(|(event, data)| runtime_event(event, data))
        .collect();

    let RuntimeEvent::FetchLogins(FetchLogins { domain }) = &decoded[0] else {
        panic!()
    };
    assert_eq!(domain.as_deref(), Some("example.org"));

    let RuntimeEvent::SaveLogin(save) = &decoded[2] else {
        panic!()
    };
    assert_eq!(save.login.times_used, Some(1));

    let RuntimeEvent::LoginUsed(used) = &decoded[4] else {
        panic!()
    };
    assert!(used.used_fields.contains(UsedFields::PASSWORD));
    assert_eq!(used.login.guid.as_deref(), Some("{g}"));
}

#[test]
fn answers_fetches_with_the_entries_java_would_send() {
    let login = LoginEntry {
        guid: Some("{g}".to_owned()),
        origin: "https://example.org".to_owned(),
        username: "rikka".to_owned(),
        password: "pw".to_owned(),
        ..LoginEntry::default()
    };
    assert_eq!(
        answered::<FetchLogins>(&vec![login]),
        json!([{
            "guid": "{g}",
            "origin": "https://example.org",
            "formActionOrigin": null,
            "httpRealm": null,
            "username": "rikka",
            "password": "pw",
        }])
    );
    assert_eq!(
        sent_data(&GetAddressStructure {
            country: "JP".to_owned()
        }),
        json!({"country": "JP"})
    );
    let structure: AddressStructure = gecko_reply::<GetAddressStructure>(json!({
        "fields": [
            {"fieldId": "name", "l10nId": "autofill-address-name"},
            {"fieldId": "address-level1", "l10nId": "autofill-address-prefecture", "value": "", "options": [{"text": "Tokyo", "value": "13"}]},
        ],
    }));
    assert_eq!(structure.fields[1].options.as_ref().unwrap()[0].value, "13");
}
