//! kazari: pick a palette, a font, a prompt and a window effect, preview it, then apply it.
//!
//! The app follows iced's Elm architecture: `App` is the state, `Message` lists every user
//! action, `update` changes the state, and `view` draws it. All the real work lives in
//! `kazari-core`; this crate only collects choices and shows them.

use std::path::PathBuf;

use iced::widget::{Space, button, column, container, pick_list, radio, row, slider, text};
use iced::{Background, Border, Color as IcedColor, Element, Fill, Font, Length, Theme};
use kazari_core::palette::{self, Color, Palette};
use kazari_core::plan::{self, Paths};
use kazari_core::rice::FONTS;
use kazari_core::{PromptStyle, Rice, Terminal, WindowEffect};

pub fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("kazari 飾り")
        .theme(App::theme)
        .window_size((980.0, 640.0))
        .run()
}

/// The three window effects, without the opacity value, for the radio buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EffectKind {
    Opaque,
    Transparent,
    Acrylic,
}

struct App {
    palettes: Vec<Palette>,
    palette: Palette,
    font: String,
    prompt: PromptStyle,
    terminal: Terminal,
    effect: EffectKind,
    opacity: u8,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    PaletteSelected(Palette),
    FontSelected(String),
    PromptSelected(PromptStyle),
    TerminalSelected(Terminal),
    EffectSelected(EffectKind),
    OpacityChanged(u8),
    Export,
    Apply,
}

impl App {
    fn new() -> Self {
        let palettes = palette::builtin();
        Self {
            palette: palettes[0].clone(),
            palettes,
            font: FONTS[0].to_owned(),
            prompt: PromptStyle::Powerline,
            terminal: Terminal::WindowsTerminal,
            effect: EffectKind::Acrylic,
            opacity: 85,
            status: String::from("Elige tu estilo y aplícalo."),
        }
    }

    fn rice(&self) -> Rice {
        let effect = match self.effect {
            EffectKind::Opaque => WindowEffect::Opaque,
            EffectKind::Transparent => WindowEffect::Transparent {
                opacity: self.opacity,
            },
            EffectKind::Acrylic => WindowEffect::Acrylic {
                opacity: self.opacity,
            },
        };
        Rice {
            palette: self.palette.clone(),
            font: self.font.clone(),
            font_size: 12,
            prompt: self.prompt,
            terminal: self.terminal,
            effect,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PaletteSelected(palette) => self.palette = palette,
            Message::FontSelected(font) => self.font = font,
            Message::PromptSelected(prompt) => self.prompt = prompt,
            Message::TerminalSelected(terminal) => self.terminal = terminal,
            Message::EffectSelected(effect) => self.effect = effect,
            Message::OpacityChanged(opacity) => self.opacity = opacity,
            Message::Export => {
                let dir = PathBuf::from("kazari-export");
                self.status = self.write_to(&Paths::export(&dir), &format!("Exportado en {}", dir.display()));
            }
            Message::Apply => {
                self.status = match Paths::from_env() {
                    Some(paths) => {
                        self.write_to(&paths, "Aplicado. Recarga komorebi y YASB para ver los cambios.")
                    }
                    None => {
                        "Aplicar solo funciona en Windows; usa Exportar para revisar los archivos.".into()
                    }
                };
            }
        }
    }

    fn write_to(&self, paths: &Paths, success: &str) -> String {
        let result =
            plan::plan(&self.rice(), paths, plan::read_or_empty).and_then(|files| plan::write(&files));
        match result {
            Ok(()) => success.to_owned(),
            Err(error) => format!("Error: {error}"),
        }
    }

    /// The app wears the palette being edited.
    fn theme(&self) -> Theme {
        let p = &self.palette;
        Theme::custom(
            "kazari",
            iced::theme::Palette {
                background: to_iced(p.background),
                text: to_iced(p.foreground),
                primary: to_iced(p.accent),
                success: to_iced(p.ansi[2]),
                warning: to_iced(p.ansi[3]),
                danger: to_iced(p.ansi[1]),
            },
        )
    }

    fn view(&self) -> Element<'_, Message> {
        let fonts: Vec<String> = FONTS.iter().map(|f| (*f).to_owned()).collect();

        let effect_radios = [
            (EffectKind::Opaque, "Opaca"),
            (EffectKind::Transparent, "Solo transparencia"),
            (EffectKind::Acrylic, "Acrílico"),
        ]
        .into_iter()
        .fold(column![].spacing(6), |col, (kind, label)| {
            col.push(radio(label, kind, Some(self.effect), Message::EffectSelected))
        });

        let mut effect_section = column![label("Ventanas de terminal"), effect_radios].spacing(8);
        if self.effect != EffectKind::Opaque {
            effect_section = effect_section.push(
                row![
                    slider(30..=100, self.opacity, Message::OpacityChanged).width(Fill),
                    text(format!("{}%", self.opacity)).width(48),
                ]
                .spacing(10),
            );
        }
        if self.effect == EffectKind::Acrylic && !self.terminal.supports_acrylic() {
            effect_section = effect_section.push(
                text(format!(
                    "{} no tiene acrílico: se aplicará solo transparencia.",
                    self.terminal
                ))
                .size(12),
            );
        }

        let controls = column![
            column![
                label("Paleta"),
                pick_list(
                    self.palettes.clone(),
                    Some(self.palette.clone()),
                    Message::PaletteSelected
                )
                .width(Fill)
            ]
            .spacing(6),
            column![
                label("Tipografía"),
                pick_list(fonts, Some(self.font.clone()), Message::FontSelected).width(Fill)
            ]
            .spacing(6),
            column![
                label("Prompt (Oh My Posh)"),
                pick_list(PromptStyle::ALL, Some(self.prompt), Message::PromptSelected).width(Fill)
            ]
            .spacing(6),
            column![
                label("Terminal (y su fastfetch)"),
                pick_list(Terminal::ALL, Some(self.terminal), Message::TerminalSelected).width(Fill)
            ]
            .spacing(6),
            effect_section,
            Space::new().height(Fill),
            row![
                button("Exportar")
                    .on_press(Message::Export)
                    .style(button::secondary),
                button("Aplicar").on_press(Message::Apply),
            ]
            .spacing(10),
            text(&self.status).size(13),
        ]
        .spacing(18)
        .width(320);

        let content = row![controls, self.preview()].spacing(28).padding(28);
        container(content).width(Fill).height(Fill).into()
    }

    /// A mock terminal: swatches, a prompt in the chosen style and a fastfetch-like block.
    fn preview(&self) -> Element<'_, Message> {
        let p = &self.palette;
        let mono = Font::with_name(leak(&self.font));
        let colored = |s: String, c: Color| text(s).font(mono).color(to_iced(c));

        let swatch = |c: Color| {
            container(Space::new().width(Fill).height(28)).style(move |_: &Theme| container::Style {
                background: Some(Background::Color(to_iced(c))),
                ..Default::default()
            })
        };
        let normal = p.ansi[..8]
            .iter()
            .fold(row![].spacing(4), |r, c| r.push(swatch(*c)));
        let bright = p.ansi[8..]
            .iter()
            .fold(row![].spacing(4), |r, c| r.push(swatch(*c)));

        let prompt_line = match self.prompt {
            PromptStyle::Minimal => row![
                colored("~/kazari ".into(), p.accent),
                colored("main".into(), p.muted)
            ],
            PromptStyle::Powerline | PromptStyle::Pills => {
                let pill = |s: &str, bg: Color| {
                    container(text(s.to_owned()).font(mono).color(to_iced(p.background)))
                        .padding([2, 10])
                        .style(move |_: &Theme| container::Style {
                            background: Some(Background::Color(to_iced(bg))),
                            border: Border::default().rounded(if self.prompt == PromptStyle::Pills {
                                12
                            } else {
                                0
                            }),
                            ..Default::default()
                        })
                };
                row![pill("~/kazari", p.accent), pill("main", p.ansi[2])]
                    .spacing(if self.prompt == PromptStyle::Pills { 6 } else { 0 })
            }
        };

        let facts = column![
            row![
                colored("os      ".into(), p.accent),
                colored("Windows 11".into(), p.foreground)
            ],
            row![
                colored("wm      ".into(), p.accent),
                colored("komorebi".into(), p.foreground)
            ],
            row![
                colored("bar     ".into(), p.accent),
                colored("YASB".into(), p.foreground)
            ],
            row![
                colored("term    ".into(), p.accent),
                colored(self.terminal.to_string(), p.foreground)
            ],
            row![
                colored("font    ".into(), p.accent),
                colored(self.font.clone(), p.foreground)
            ],
        ]
        .spacing(2);

        let rice = self.rice().effect.for_terminal(self.terminal);
        let alpha = rice.opacity();
        let window = container(
            column![
                text(format!("{}  ·  {}", self.terminal, effect_label(rice)))
                    .size(12)
                    .color(to_iced(p.muted)),
                prompt_line,
                colored("❯ fastfetch".into(), p.foreground),
                facts,
                normal,
                bright,
            ]
            .spacing(16),
        )
        .padding(22)
        .width(Fill)
        .height(Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(IcedColor {
                a: alpha,
                ..to_iced(p.background)
            })),
            border: Border::default().rounded(10).color(to_iced(p.accent)).width(1),
            ..Default::default()
        });

        container(window)
            .padding(18)
            .width(Length::Fill)
            .height(Fill)
            .style(move |_: &Theme| container::Style {
                background: Some(Background::Color(to_iced(p.surface))),
                border: Border::default().rounded(14),
                ..Default::default()
            })
            .into()
    }
}

fn label(s: &str) -> iced::widget::Text<'_> {
    text(s.to_uppercase()).size(11)
}

fn effect_label(effect: WindowEffect) -> String {
    match effect {
        WindowEffect::Opaque => "opaca".into(),
        WindowEffect::Transparent { opacity } => format!("transparencia {opacity}%"),
        WindowEffect::Acrylic { opacity } => format!("acrílico {opacity}%"),
    }
}

fn to_iced(c: Color) -> IcedColor {
    IcedColor::from_rgb8(c.r, c.g, c.b)
}

/// iced fonts need a `'static` name. The font list is tiny and fixed, so leaking each
/// distinct name once is fine; a cache keeps it from growing.
fn leak(name: &str) -> &'static str {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .expect("font cache poisoned");
    cache
        .entry(name.to_owned())
        .or_insert_with(|| Box::leak(name.to_owned().into_boxed_str()))
}
