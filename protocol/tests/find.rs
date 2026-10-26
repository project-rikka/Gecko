use rikka_gecko_protocol::geckoview::find::{DisplayMatches, FindInPage, FindResult};
use rikka_gecko_protocol::geckoview::geometry::Rect;
use serde_json::{Value, json};

use crate::support::{gecko_reply, sent_data};

#[test]
fn find_requests_carry_the_finder_options() {
    let cases: [(Value, Value); 2] = [
        (
            sent_data(&FindInPage {
                search_string: Some("rikka".to_owned()),
                match_case: true,
                ..FindInPage::default()
            }),
            json!({
                "searchString": "rikka",
                "backwards": false,
                "linksOnly": false,
                "matchCase": true,
                "wholeWord": false,
                "matchDiacritics": false,
            }),
        ),
        (
            sent_data(&DisplayMatches {
                highlight_all: true,
                dim_page: false,
                draw_outline: true,
            }),
            json!({"highlightAll": true, "dimPage": false, "drawOutline": true}),
        ),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
}

#[test]
fn find_results_decode_as_the_finder_listener_builds_them() {
    let cases: [(Value, bool, i32, Option<Rect>); 2] = [
        (
            json!({
                "found": true,
                "wrapped": false,
                "current": 2,
                "total": 7,
                "searchString": "rikka",
                "linkURL": null,
                "clientRect": {"left": 10, "top": 20, "right": 60, "bottom": 34},
                "flags": {"backwards": false, "linksOnly": false, "matchCase": true, "wholeWord": false},
            }),
            true,
            7,
            Some(Rect {
                left: 10.0,
                top: 20.0,
                right: 60.0,
                bottom: 34.0,
            }),
        ),
        (
            json!({
                "found": false,
                "wrapped": false,
                "current": 0,
                "total": 0,
                "searchString": "absent",
                "linkURL": null,
                "clientRect": null,
                "flags": {"backwards": true, "matchCase": false, "wholeWord": false},
            }),
            false,
            0,
            None,
        ),
    ];
    for (data, found, total, rect) in cases {
        let result: FindResult = gecko_reply::<FindInPage>(data);
        assert_eq!(
            (result.found, result.total, result.client_rect),
            (found, total, rect)
        );
    }
}
