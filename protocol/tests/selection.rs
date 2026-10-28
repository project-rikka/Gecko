use rikka_gecko_protocol::geckoview::selection::{
    ClipboardPermissionRequest, ExecuteSelectionAction, HideReason, SelectionAction,
};
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{answered, sent_data, window_event};

#[test]
fn decodes_selection_events_as_the_actors_build_them() {
    let cases: [(&str, Value); 5] = [
        (
            "GeckoView:ShowSelectionAction",
            json!({
                "collapsed": false,
                "editable": true,
                "password": false,
                "selection": "hello",
                "screenRect": {"left": 10, "top": 20, "right": 80, "bottom": 56},
                "actions": [
                    "org.mozilla.geckoview.HIDE",
                    "org.mozilla.geckoview.CUT",
                    "org.mozilla.geckoview.COPY",
                    "org.mozilla.geckoview.SELECT_ALL",
                ],
                "actionId": "{7e1c4e44-8f3a-4c9f-9a52-0b5d1c2e3f40}",
            }),
        ),
        (
            "GeckoView:HideSelectionAction",
            json!({"reason": "invisibleselection"}),
        ),
        (
            "GeckoView:ShowMagnifier",
            json!({"screenPoint": {"x": 40, "y": 60.5}}),
        ),
        (
            "GeckoView:ClipboardPermissionRequest",
            json!({"screenPoint": {"x": 5, "y": 6}}),
        ),
        ("GeckoView:DismissClipboardPermissionRequest", Value::Null),
    ];
    let decoded: Vec<WindowEvent> = cases
        .into_iter()
        .map(|(event, data)| window_event(event, data))
        .collect();

    let WindowEvent::ShowSelectionAction(show) = &decoded[0] else {
        panic!()
    };
    assert_eq!(show.actions[1], SelectionAction::Cut);
    assert_eq!(show.screen_rect.unwrap().bottom, 56.0);

    let WindowEvent::HideSelectionAction(hide) = &decoded[1] else {
        panic!()
    };
    assert_eq!(hide.reason, HideReason::InvisibleSelection);
}

#[test]
fn executes_actions_with_the_id_gecko_handed_out() {
    assert_eq!(
        sent_data(&ExecuteSelectionAction {
            id: SelectionAction::PasteAsPlainText,
            action_id: "{7e1c}".to_owned(),
        }),
        json!({"id": "org.mozilla.geckoview.PASTE_AS_PLAIN_TEXT", "actionId": "{7e1c}"})
    );
    assert_eq!(answered::<ClipboardPermissionRequest>(&true), json!(true));
}
