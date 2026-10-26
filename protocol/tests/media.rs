use rikka_gecko_protocol::geckoview::media::{
    AudioFocusChange, MuteAudio, Play, RecordingDeviceType, RecordingStatus, SeekTo,
    SystemAudioFocus,
};
use rikka_gecko_protocol::inbound::WindowEvent;
use serde_json::{Value, json};

use crate::support::{sent_data, window_event};

#[test]
fn decodes_media_session_events_as_the_controller_reports_them() {
    let cases: [(&str, Value); 8] = [
        ("GeckoView:MediaSession:Activated", Value::Null),
        (
            "GeckoView:MediaSession:Metadata",
            json!({
                "metadata": {
                    "title": "Song",
                    "artist": "Band",
                    "album": "",
                    "artwork": [{"src": "https://music.example/cover.png", "sizes": "512x512", "type": "image/png"}],
                },
            }),
        ),
        (
            "GeckoView:MediaSession:PositionState",
            json!({"state": {"duration": 215.4, "playbackRate": 1, "position": 12.5}}),
        ),
        (
            "GeckoView:MediaSession:PositionState",
            json!({"state": {"duration": null, "playbackRate": 1, "position": 0}}),
        ),
        (
            "GeckoView:MediaSession:Features",
            json!({"features": {"play": true, "pause": true, "seekto": true, "nexttrack": true}}),
        ),
        ("GeckoView:MediaSession:Playback:Playing", Value::Null),
        (
            "GeckoView:MediaSession:Fullscreen",
            json!({
                "metadata": {
                    "src": "https://video.example/v.mp4",
                    "width": 1920,
                    "height": 1080,
                    "duration": 61.2,
                    "seekable": true,
                    "audioTrackCount": 1,
                    "videoTrackCount": 1,
                },
                "enabled": true,
            }),
        ),
        (
            "GeckoView:MediaRecordingStatusChanged",
            json!({"devices": [{"type": "microphone", "status": "inactive"}]}),
        ),
    ];
    let decoded: Vec<WindowEvent> = cases
        .into_iter()
        .map(|(event, data)| window_event(event, data))
        .collect();

    let WindowEvent::MetadataChanged(metadata) = &decoded[1] else {
        panic!()
    };
    assert_eq!(
        metadata.metadata.artwork[0].sizes.as_deref(),
        Some("512x512")
    );

    let WindowEvent::PositionStateChanged(live) = &decoded[3] else {
        panic!()
    };
    assert_eq!(live.state.duration, None);

    let WindowEvent::FeaturesChanged(features) = &decoded[4] else {
        panic!()
    };
    assert!(features.supports("seekto"));
    assert!(!features.supports("skipad"));

    let WindowEvent::MediaFullscreen(fullscreen) = &decoded[6] else {
        panic!()
    };
    assert_eq!(fullscreen.metadata.width, Some(1920));

    let WindowEvent::RecordingStatusChanged(recording) = &decoded[7] else {
        panic!()
    };
    assert_eq!(
        recording.devices[0].device_type,
        RecordingDeviceType::Microphone
    );
    assert_eq!(recording.devices[0].status, RecordingStatus::Inactive);
}

#[test]
fn encodes_media_controls_with_the_keys_the_module_reads() {
    let cases: [(Value, Value); 4] = [
        (sent_data(&Play), Value::Null),
        (
            sent_data(&SeekTo {
                time: 30.0,
                fast: false,
            }),
            json!({"time": 30.0, "fast": false}),
        ),
        (sent_data(&MuteAudio { mute: true }), json!({"mute": true})),
        (
            sent_data(&SystemAudioFocus {
                reason: AudioFocusChange::SystemTransient,
            }),
            json!({"reason": "system-transient"}),
        ),
    ];
    for (sent, expected) in cases {
        assert_eq!(sent, expected);
    }
}
