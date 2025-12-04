use serde_json::Value;

fn main() {
    let mut first_tick = true;

    for line in io::stdin().lock().lines() {
        if line.is_err() {
            break;
        }
        let line = line.unwrap();

        let data = if !line.trim().is_empty() {
            serde_json::from_str::<Value>(&line).map(|v| v).ok()
        } else {
            None
        };

        // infos for the start
        if first_tick {
            if let Some(Value::Object(map)) = data.as_ref() {
                if let Some(Value::Object(cfg_map)) = map.get("config") {
                    let width = cfg_map.get("width")
                        .map(|v| v.as_i64()
                        .unwrap_or(0))
                        .unwrap_or(0);
                    let height = cfg_map.get("height").map(|v| v.as_i64().unwrap_or(0)).unwrap_or(0);
                    eprintln!("Random walker (Rust) launching on a {width}x{height} map");
                    first_tick = false;
                }
            }
        }
    }
}