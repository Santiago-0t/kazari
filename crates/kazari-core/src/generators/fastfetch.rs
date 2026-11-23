//! fastfetch: a config tuned to the terminal in use.

use serde_json::json;

use crate::rice::{Rice, Terminal};

/// Builds `config.jsonc`. The logo and its size change with the terminal: WezTerm can draw
/// a larger logo, Alacritty gets a compact one.
pub fn config(rice: &Rice) -> String {
    let p = &rice.palette;
    let logo = match rice.terminal {
        Terminal::WindowsTerminal => json!({ "source": "windows", "padding": { "top": 1, "right": 3 } }),
        Terminal::WezTerm => json!({ "source": "windows", "padding": { "top": 2, "left": 2, "right": 4 } }),
        Terminal::Alacritty => json!({ "source": "windows_small", "padding": { "right": 2 } }),
    };
    let mut logo = logo;
    logo["color"] = json!({ "1": p.accent.to_ansi_fg(), "2": p.ansi[4].to_ansi_fg() });

    let config = json!({
        "$schema": "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json",
        "logo": logo,
        "display": {
            "separator": "  ",
            "color": { "keys": p.accent.to_ansi_fg(), "title": p.foreground.to_ansi_fg() }
        },
        "modules": [
            "title", "separator", "os", "kernel", "uptime", "shell", "terminal",
            "terminalfont", "wm", "cpu", "gpu", "memory", "break", "colors"
        ]
    });
    serde_json::to_string_pretty(&config).expect("static JSON") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::sample_rice;
    use serde_json::Value;

    #[test]
    fn logo_depends_on_the_terminal() {
        let mut rice = sample_rice();
        rice.terminal = Terminal::Alacritty;
        let small: Value = serde_json::from_str(&config(&rice)).unwrap();
        rice.terminal = Terminal::WezTerm;
        let large: Value = serde_json::from_str(&config(&rice)).unwrap();
        assert_eq!(small["logo"]["source"], "windows_small");
        assert_eq!(large["logo"]["source"], "windows");
        assert_eq!(large["display"]["color"]["keys"], "38;2;183;126;100");
    }
}
