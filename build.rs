const COMMANDS: &[&str] = &[
  "check_permissions",
  "request_permissions",
  "wifi_current",
  "wifi_added_ssids",
  "wifi_add",
  "wifi_request_join",
  "wifi_forget",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
