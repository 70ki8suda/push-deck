use push_deck::config::schema::{
    AppSettings, Config, LayoutProfile, PadAction, PadBinding, PadColorId, ShortcutKey,
    ShortcutModifier, ShortcutSpec,
};
use serde_json::json;

#[test]
fn default_config_generates_one_default_profile_with_64_unassigned_pads() {
    let config = Config::default();

    assert_eq!(config.schema_version, 1);
    assert_eq!(config.settings.active_profile_id, "default");
    assert_eq!(config.profiles.len(), 1);

    let profile = &config.profiles[0];
    assert_eq!(profile.id, "default");
    assert_eq!(profile.name, "Default");
    assert_eq!(profile.pads.len(), 64);
    assert_eq!(profile.pads[0].pad_id, "r0c0");
    assert_eq!(profile.pads[63].pad_id, "r7c7");
    assert!(profile
        .pads
        .iter()
        .all(|pad| pad.action == PadAction::Unassigned));
}

#[test]
fn normalize_profile_fills_missing_pads_to_64_entries_with_unassigned_actions() {
    let profile = LayoutProfile {
        id: "default".to_string(),
        name: "Default".to_string(),
        pads: vec![
            PadBinding {
                pad_id: "r0c0".to_string(),
                label: "Launch".to_string(),
                color: PadColorId::Green,
                action: PadAction::Unassigned,
            },
            PadBinding {
                pad_id: "r7c7".to_string(),
                label: "".to_string(),
                color: PadColorId::Off,
                action: PadAction::Unassigned,
            },
        ],
    };

    let normalized = Config::from_parts(
        AppSettings {
            active_profile_id: "default".to_string(),
        },
        vec![profile],
    )
    .expect("profile should normalize");

    let normalized_profile = &normalized.profiles[0];
    assert_eq!(normalized_profile.pads.len(), 64);
    assert_eq!(normalized_profile.pads[0].pad_id, "r0c0");
    assert_eq!(normalized_profile.pads[63].pad_id, "r7c7");
    assert!(normalized_profile
        .pads
        .iter()
        .any(|pad| pad.pad_id == "r0c1" && pad.action == PadAction::Unassigned));
}

#[test]
fn unassigned_pads_are_preserved_when_normalizing() {
    let profile = LayoutProfile {
        id: "default".to_string(),
        name: "Default".to_string(),
        pads: vec![PadBinding {
            pad_id: "r3c2".to_string(),
            label: "".to_string(),
            color: PadColorId::Off,
            action: PadAction::Unassigned,
        }],
    };

    let normalized = Config::from_parts(
        AppSettings {
            active_profile_id: "default".to_string(),
        },
        vec![profile],
    )
    .expect("profile should normalize");

    let binding = normalized.profile("default").expect("default profile");
    assert!(binding
        .pads
        .iter()
        .any(|pad| pad.pad_id == "r3c2" && pad.action == PadAction::Unassigned));
}

#[test]
fn invalid_pad_ids_are_rejected() {
    let profile = LayoutProfile {
        id: "default".to_string(),
        name: "Default".to_string(),
        pads: vec![PadBinding {
            pad_id: "r8c0".to_string(),
            label: "".to_string(),
            color: PadColorId::Off,
            action: PadAction::Unassigned,
        }],
    };

    let error = Config::from_parts(
        AppSettings {
            active_profile_id: "default".to_string(),
        },
        vec![profile],
    )
    .expect_err("invalid pad id should be rejected");

    assert!(error.to_string().contains("padId"));
}

#[test]
fn invalid_shortcut_modifier_and_key_are_rejected() {
    let invalid_modifier = serde_json::from_value::<ShortcutSpec>(json!({
        "modifiers": ["Cmd", "Cmdr"],
        "key": "A"
    }));
    assert!(invalid_modifier.is_err());

    let invalid_key = serde_json::from_value::<ShortcutSpec>(json!({
        "modifiers": ["Cmd"],
        "key": "F13"
    }));
    assert!(invalid_key.is_err());
}

#[test]
fn launch_or_focus_app_serializes_with_camel_case_field_names() {
    let action = PadAction::LaunchOrFocusApp {
        bundle_id: "com.apple.Terminal".to_string(),
        app_name: "Terminal".to_string(),
    };

    assert_eq!(
        serde_json::to_value(action).expect("action should serialize"),
        json!({
            "type": "launch_or_focus_app",
            "bundleId": "com.apple.Terminal",
            "appName": "Terminal"
        })
    );
}

#[test]
fn send_shortcut_serializes_with_tagged_enum_and_value_names() {
    let action = PadAction::SendShortcut {
        key: ShortcutKey::F12,
        modifiers: vec![ShortcutModifier::Cmd, ShortcutModifier::Shift],
    };

    assert_eq!(
        serde_json::to_value(action).expect("action should serialize"),
        json!({
            "type": "send_shortcut",
            "key": "F12",
            "modifiers": ["Cmd", "Shift"]
        })
    );
}

#[test]
fn invalid_action_payload_is_rejected_during_config_normalization() {
    let profile = LayoutProfile {
        id: "default".to_string(),
        name: "Default".to_string(),
        pads: vec![PadBinding {
            pad_id: "r0c0".to_string(),
            label: "".to_string(),
            color: PadColorId::Off,
            action: PadAction::SendShortcut {
                key: ShortcutKey::A,
                modifiers: vec![ShortcutModifier::Cmd, ShortcutModifier::Cmd],
            },
        }],
    };

    let error = Config::from_parts(
        AppSettings {
            active_profile_id: "default".to_string(),
        },
        vec![profile],
    )
    .expect_err("invalid shortcut payload should be rejected");

    assert!(error.to_string().contains("shortcut"));
}
