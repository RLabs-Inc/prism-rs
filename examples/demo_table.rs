use prism::{
    frame, list, s, table, writeln, Align, Column, FrameOptions, ListOptions, ListStyle,
    TableOptions,
};

fn main() {
    writeln("=== Prism Table & Display Demo ===\n");

    // Table
    let data = vec![
        vec![
            ("bssid", "7C:10:C9:03:10:E4"),
            ("ssid", "RL-WiFi"),
            ("signal", "-42"),
            ("security", "WPA2"),
            ("ch", "7"),
        ],
        vec![
            ("bssid", "AA:BB:CC:DD:EE:01"),
            ("ssid", "NetGear-5G"),
            ("signal", "-67"),
            ("security", "WPA3"),
            ("ch", "36"),
        ],
        vec![
            ("bssid", "11:22:33:44:55:66"),
            ("ssid", "OpenCafe"),
            ("signal", "-78"),
            ("security", "Open"),
            ("ch", "11"),
        ],
        vec![
            ("bssid", "DE:AD:BE:EF:CA:FE"),
            ("ssid", "Hidden"),
            ("signal", "-55"),
            ("security", "WPA2-EAP"),
            ("ch", "1"),
        ],
    ];

    let output = table(
        &data,
        &TableOptions {
            columns: Some(vec![
                Column {
                    key: "bssid".into(),
                    label: Some("BSSID".into()),
                    ..Default::default()
                },
                Column {
                    key: "ssid".into(),
                    label: Some("SSID".into()),
                    ..Default::default()
                },
                Column {
                    key: "signal".into(),
                    label: Some("dBm".into()),
                    align: Align::Right,
                    ..Default::default()
                },
                Column {
                    key: "security".into(),
                    label: Some("Security".into()),
                    ..Default::default()
                },
                Column {
                    key: "ch".into(),
                    label: Some("Ch".into()),
                    align: Align::Right,
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
    );
    writeln(&output);

    // Frame
    writeln("");
    let framed = frame(
        "Network scan complete.\n4 access points found.\n1 open network detected!",
        &FrameOptions {
            title: Some("Scan Results".into()),
            ..Default::default()
        },
    );
    writeln(&framed);

    // List
    writeln("");
    let items = list(
        &[
            "WPA2-Personal",
            "WPA3-SAE",
            "WPA2-Enterprise",
            "Open",
            "WEP (deprecated)",
        ],
        &ListOptions {
            style: ListStyle::Bullet,
            ..Default::default()
        },
    );
    writeln(&format!(
        "{}:\n{}",
        s().bold().paint("Security Types"),
        items
    ));
}
