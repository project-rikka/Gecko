use rikka_gecko_protocol::geckoview::print::{DotPrintFinish, PdfSave};
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{gecko_reply, sent_data, window_event};

#[test]
fn decodes_print_and_pdf_events() {
    let cases: [(&str, Value); 2] = [
        (
            "GeckoView:DotPrintRequest",
            json!({"canonicalBrowsingContextId": 4294967309u64}),
        ),
        (
            "GeckoView:SavePdf",
            json!({
                "url": "blob:resource://pdf.js/0f2c",
                "filename": "report.pdf",
                "originalUrl": "https://example.org/report.pdf",
                "isPrivate": false,
                "skipConfirmation": true,
                "requestExternalApp": false,
            }),
        ),
    ];
    let decoded: Vec<WindowEvent> = cases
        .into_iter()
        .map(|(event, data)| window_event(event, data))
        .collect();

    let WindowEvent::DotPrintRequest(request) = &decoded[0] else {
        panic!()
    };
    assert_eq!(request.canonical_browsing_context_id, 4294967309);

    let WindowEvent::SavePdf(save) = &decoded[1] else {
        panic!()
    };
    assert_eq!(save.filename, "report.pdf");
}

#[test]
fn print_completion_and_pdf_export_use_the_handler_keys() {
    assert_eq!(
        sent_data(&DotPrintFinish {
            is_pdf_successful: false,
            error_reason: Some(1)
        }),
        json!({"isPdfSuccessful": false, "errorReason": 1})
    );
    let saved = gecko_reply::<PdfSave>(json!({
        "url": "blob:resource://pdf.js/1a",
        "filename": "a.pdf",
        "originalUrl": "https://example.org/a.pdf",
        "isPrivate": true,
    }));
    assert!(saved.is_private);
}
