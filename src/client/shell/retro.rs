//! Retro sidebar drawing, enabled with `[ui.sidebar] retro = true`.

use ratatui::style::Color;

use super::agent_sidebar::AgentRow;
use super::render::{display_width, put_right_text, put_text};
use super::*;
use crate::api::schema::AgentStatus;

pub(super) const SPACES_LABEL: &str = "▓▒░ SPACES";
pub(super) const AGENTS_LABEL: &str = "▓▒░ AGENTS";

const CARD_ROWS: u16 = 8;
const COMPACT_CARD_ROWS: u16 = 4;
const MIN_ART_WIDTH: u16 = 28;
const ART_COLS: usize = 14;
const ART_PX_ROWS: usize = 12;
const SPRITE_COLS: usize = 12;

const FOCUS_EDGE: Color = Color::Rgb(232, 165, 60);
const FOCUS_BG: Color = Color::Rgb(30, 34, 48);
const BLOCK_EDGE: Color = Color::Rgb(240, 100, 90);
const STAR: Color = Color::Rgb(225, 215, 240);
const INK_DARK: Color = Color::Rgb(24, 22, 32);
const TEXT: Color = Color::Rgb(232, 232, 240);
const DIM: Color = Color::Rgb(140, 140, 158);

const SPRITES: [[&str; 10]; 6] = [
    [
        "............",
        "..oooooooo..",
        "..okooooko..",
        "..okooooko..",
        "oooooooooooo",
        "oooooooooooo",
        "..oooooooo..",
        "..oooooooo..",
        "..o.o..o.o..",
        "..o.o..o.o..",
    ],
    [
        ".wwwwwwwwww.",
        ".wkkkkkkkkw.",
        ".wkgkkkkgkw.",
        ".wkgkkkkgkw.",
        ".wkkkkkkkkw.",
        ".wkkggggkkw.",
        ".wkkkkkkkkw.",
        ".wwwwwwwwww.",
        "....wwww....",
        "..wwwwwwww..",
    ],
    [
        ".....rr.....",
        ".....ww.....",
        "..wwwwwwww..",
        "..wbbwwbbw..",
        "..wbbwwbbw..",
        "..wwwwwwww..",
        "..wwkkkkww..",
        "...wwwwww...",
        ".gg.wwww.gg.",
        "....w..w....",
    ],
    [
        "...yyyy.....",
        "..yyyyyy....",
        "..yykyyy....",
        "..yyyyyyrr..",
        "..yyyyyyrr..",
        "...yyyy.....",
        "y.yyyyyyyy..",
        "yyyyyyyyyy..",
        ".yyyyyyyy...",
        "...r..r.....",
    ],
    [
        "...pppppp...",
        "..pppppppp..",
        ".pppppppppp.",
        ".ppkkppkkpp.",
        ".ppkkppkkpp.",
        ".pppppppppp.",
        ".pppppppppp.",
        ".pppppppppp.",
        ".pp.pppp.pp.",
        ".p...pp...p.",
    ],
    [
        ".c........c.",
        ".cc......cc.",
        ".cccccccccc.",
        ".ckcccccckc.",
        ".ckcccccckc.",
        ".cccckkcccc.",
        ".cccccccccc.",
        "..cccccccc..",
        "..cc.cc.cc..",
        "..cc....cc..",
    ],
];

fn ink(key: u8) -> Color {
    match key {
        b'o' => Color::Rgb(205, 124, 94),
        b'k' => INK_DARK,
        b'w' => Color::Rgb(214, 222, 235),
        b'g' => Color::Rgb(120, 220, 140),
        b'r' => Color::Rgb(232, 96, 84),
        b'b' => Color::Rgb(86, 170, 240),
        b'y' => Color::Rgb(245, 205, 70),
        b'p' => Color::Rgb(190, 160, 245),
        _ => Color::Rgb(150, 150, 170),
    }
}

pub(super) struct Scene {
    tag: &'static str,
    sky: [Color; 4],
    ground: Color,
    stripe: Color,
    edge: Color,
    bg: Color,
    head: Color,
}

const SCENES: [Scene; 3] = [
    Scene {
        tag: "DUSK",
        sky: [
            Color::Rgb(30, 18, 58),
            Color::Rgb(50, 24, 78),
            Color::Rgb(78, 34, 92),
            Color::Rgb(112, 46, 96),
        ],
        ground: Color::Rgb(38, 22, 60),
        stripe: Color::Rgb(112, 46, 96),
        edge: Color::Rgb(84, 56, 150),
        bg: Color::Rgb(28, 18, 48),
        head: Color::Rgb(240, 130, 90),
    },
    Scene {
        tag: "DEN",
        sky: [
            Color::Rgb(18, 34, 48),
            Color::Rgb(22, 46, 62),
            Color::Rgb(28, 60, 76),
            Color::Rgb(36, 76, 88),
        ],
        ground: Color::Rgb(20, 36, 46),
        stripe: Color::Rgb(46, 96, 104),
        edge: Color::Rgb(56, 110, 130),
        bg: Color::Rgb(22, 30, 38),
        head: Color::Rgb(230, 165, 70),
    },
    Scene {
        tag: "MOSS",
        sky: [
            Color::Rgb(20, 36, 30),
            Color::Rgb(28, 50, 36),
            Color::Rgb(40, 66, 44),
            Color::Rgb(56, 84, 52),
        ],
        ground: Color::Rgb(24, 38, 28),
        stripe: Color::Rgb(70, 104, 60),
        edge: Color::Rgb(80, 124, 80),
        bg: Color::Rgb(22, 32, 26),
        head: Color::Rgb(130, 215, 200),
    },
];

/// The local machine is always DUSK; other machines cycle through the rest.
pub(super) fn scene_for(endpoint_index: usize) -> &'static Scene {
    if endpoint_index == 0 {
        &SCENES[0]
    } else {
        &SCENES[1 + (endpoint_index - 1) % (SCENES.len() - 1)]
    }
}

fn status_badge(status: AgentStatus) -> (&'static str, Color) {
    match status {
        AgentStatus::Idle => ("IDLE", Color::Rgb(170, 170, 190)),
        AgentStatus::Working => ("BUSY", Color::Rgb(120, 220, 140)),
        AgentStatus::Blocked => ("WAIT", BLOCK_EDGE),
        AgentStatus::Done => ("DONE", Color::Rgb(110, 180, 250)),
        AgentStatus::Unknown => ("????", Color::Rgb(130, 130, 145)),
    }
}

fn hash(text: &str) -> u32 {
    text.bytes().fold(0x811c_9dc5, |value, byte| {
        (value ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

pub(super) fn card_height(width: u16) -> u16 {
    if width >= MIN_ART_WIDTH {
        CARD_ROWS
    } else {
        COMPACT_CARD_ROWS
    }
}

fn put_pixel_cell(buffer: &mut Buffer, x: u16, y: u16, top: Color, bottom: Color) {
    if let Some(cell) = buffer.cell_mut((x, y)) {
        cell.set_char('▀').set_fg(top).set_bg(bottom);
    }
}

fn art_pixels(seed: u32, scene: &Scene) -> [[Color; ART_COLS]; ART_PX_ROWS] {
    let mut pixels = [[scene.ground; ART_COLS]; ART_PX_ROWS];
    let sky_rows = ART_PX_ROWS - 2;
    for (y, row) in pixels.iter_mut().enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            *pixel = if y >= sky_rows {
                if (x + y) % 3 == 0 {
                    scene.stripe
                } else {
                    scene.ground
                }
            } else {
                scene.sky[y * scene.sky.len() / sky_rows]
            };
        }
    }
    for star in 0..3 {
        let x = (seed >> (star * 5)) as usize % ART_COLS;
        let y = (seed >> (star * 3 + 16)) as usize % 4;
        pixels[y][x] = STAR;
    }
    let sprite = &SPRITES[seed as usize % SPRITES.len()];
    let top = ART_PX_ROWS - sprite.len();
    for (sprite_y, line) in sprite.iter().enumerate() {
        for (sprite_x, key) in line.bytes().take(SPRITE_COLS).enumerate() {
            if key != b'.' {
                pixels[top + sprite_y][sprite_x + 1] = ink(key);
            }
        }
    }
    pixels
}

fn wrap_two_lines(text: &str, width: usize) -> (String, String) {
    if UnicodeWidthStr::width(text) <= width {
        return (text.to_owned(), String::new());
    }
    let mut used = 0;
    let mut cut = None;
    for (index, character) in text.char_indices() {
        used += UnicodeWidthStr::width(character.encode_utf8(&mut [0; 4]) as &str);
        if used > width {
            break;
        }
        if character == ' ' {
            cut = Some(index);
        }
    }
    match cut {
        Some(cut) => (
            text[..cut].to_owned(),
            crate::ui::truncate_end(&text[cut + 1..], width),
        ),
        None => (crate::ui::truncate_end(text, width), String::new()),
    }
}

pub(super) fn render_card(
    buffer: &mut Buffer,
    rect: Rect,
    row: &AgentRow,
    scene: &Scene,
    stable_key: &str,
) {
    if rect.width < 3 || rect.height < 2 {
        return;
    }
    let (edge, background) = if row.status == AgentStatus::Blocked {
        (BLOCK_EDGE, scene.bg)
    } else if row.focused {
        (FOCUS_EDGE, FOCUS_BG)
    } else {
        (scene.edge, scene.bg)
    };
    buffer.set_style(rect, Style::default().bg(background));
    let inner_width = rect.width - 2;
    let edge_style = Style::default().fg(edge);
    let bar = "─".repeat(inner_width as usize);
    put_text(
        buffer,
        rect.x,
        rect.y,
        rect.width,
        &format!("┌{bar}┐"),
        edge_style,
    );
    put_text(
        buffer,
        rect.x,
        rect.bottom() - 1,
        rect.width,
        &format!("└{bar}┘"),
        edge_style,
    );
    for y in rect.y + 1..rect.bottom() - 1 {
        put_text(buffer, rect.x, y, 1, "│", edge_style);
        put_text(buffer, rect.right() - 1, y, 1, "│", edge_style);
    }

    let (badge, badge_color) = status_badge(row.status);
    let badge_style = Style::default()
        .fg(badge_color)
        .add_modifier(Modifier::BOLD);
    let title_style = Style::default().fg(TEXT).add_modifier(Modifier::BOLD);
    let subtitle_style = Style::default().fg(DIM);

    if rect.height < CARD_ROWS || rect.width < MIN_ART_WIDTH {
        let x = rect.x + 1;
        let badge_text = format!("{badge} ");
        let badge_width = display_width(&badge_text).min(inner_width);
        put_text(buffer, x, rect.y + 1, inner_width, &badge_text, badge_style);
        let title_width = usize::from(inner_width - badge_width);
        put_text(
            buffer,
            x + badge_width,
            rect.y + 1,
            inner_width - badge_width,
            &crate::ui::truncate_end(&row.title, title_width),
            title_style,
        );
        if rect.height >= COMPACT_CARD_ROWS {
            put_text(
                buffer,
                x,
                rect.y + 2,
                inner_width,
                &crate::ui::truncate_end(&row.subtitle, usize::from(inner_width)),
                subtitle_style,
            );
        }
        return;
    }

    let pixels = art_pixels(hash(stable_key), scene);
    for cell_y in 0..ART_PX_ROWS / 2 {
        for x in 0..ART_COLS {
            put_pixel_cell(
                buffer,
                rect.x + 1 + x as u16,
                rect.y + 1 + cell_y as u16,
                pixels[cell_y * 2][x],
                pixels[cell_y * 2 + 1][x],
            );
        }
    }
    let marker = match row.status {
        AgentStatus::Idle => Some(("z ", Style::default().fg(TEXT).bg(INK_DARK))),
        AgentStatus::Blocked => Some(("! ", Style::default().fg(INK_DARK).bg(BLOCK_EDGE))),
        _ => None,
    };
    if let Some((text, style)) = marker {
        put_text(
            buffer,
            rect.x + 1 + ART_COLS as u16 - 2,
            rect.y + 1,
            2,
            text,
            style,
        );
    }

    let text_x = rect.x + 1 + ART_COLS as u16 + 1;
    let text_width = rect.right().saturating_sub(text_x + 2);
    let width = usize::from(text_width);
    let (first, second) = wrap_two_lines(&row.subtitle, width);
    let lines = [
        (2, format!("[{badge}]"), badge_style),
        (4, crate::ui::truncate_end(&row.title, width), title_style),
        (5, first, subtitle_style),
        (6, second, subtitle_style),
    ];
    for (offset, text, style) in lines {
        put_text(buffer, text_x, rect.y + offset, text_width, &text, style);
    }
}

pub(super) fn render_group_header(
    buffer: &mut Buffer,
    rect: Rect,
    name: &str,
    scene: &Scene,
    busy: usize,
    total: usize,
) {
    let right = format!("{} {busy}/{total}", scene.tag);
    let right_width = display_width(&right);
    let left = crate::ui::truncate_end(
        &name.to_uppercase(),
        usize::from(rect.width.saturating_sub(right_width + 4)),
    );
    let left_width = display_width(&left);
    put_text(
        buffer,
        rect.x,
        rect.y,
        rect.width,
        &left,
        Style::default().fg(scene.head).add_modifier(Modifier::BOLD),
    );
    let rule_width = rect.width.saturating_sub(left_width + right_width + 2);
    put_text(
        buffer,
        rect.x + left_width + 1,
        rect.y,
        rule_width,
        &"─".repeat(usize::from(rule_width)),
        Style::default().fg(scene.edge),
    );
    put_right_text(buffer, rect, rect.y, &right, Style::default().fg(DIM));
}

/// Machine row in the spaces section. Returns the clickable status badge.
pub(super) fn render_endpoint_header(
    buffer: &mut Buffer,
    rect: Rect,
    marker: &str,
    endpoint: &ClientShellEndpoint,
    scene: &Scene,
    auth: &super::machine_diagnostics::MachineDiagnostics,
    palette: &Palette,
) -> Rect {
    let count = endpoint
        .snapshot
        .as_deref()
        .map_or(0, |snapshot| snapshot.workspaces.len());
    let (glyph, state, color) = endpoint_status_presentation(endpoint.status, palette);
    let online = endpoint.status == ClientEndpointStatus::Online;
    let signal = if auth.required_for(endpoint) {
        "! auth".to_owned()
    } else if endpoint.status == ClientEndpointStatus::Attention {
        "! error".to_owned()
    } else if endpoint.endpoint_id.is_local() {
        String::new()
    } else if online {
        glyph.to_owned()
    } else {
        format!("{glyph} {state}")
    };
    let signal_width = display_width(&signal).min(rect.width);
    let badge = Rect::new(
        rect.right().saturating_sub(signal_width),
        rect.y,
        signal_width,
        1,
    );
    let count_text = if signal.is_empty() {
        count.to_string()
    } else {
        format!("{count} ")
    };
    let right_width = display_width(&count_text) + signal_width;
    let left = crate::ui::truncate_end(
        &format!(" {marker} {}", endpoint.label.to_uppercase()),
        usize::from(rect.width.saturating_sub(right_width + 3)),
    );
    let left_width = display_width(&left);
    put_text(
        buffer,
        rect.x,
        rect.y,
        rect.width,
        &left,
        Style::default()
            .fg(if online { scene.head } else { DIM })
            .add_modifier(Modifier::BOLD),
    );
    let rule_width = rect.width.saturating_sub(left_width + right_width + 2);
    put_text(
        buffer,
        rect.x + left_width + 1,
        rect.y,
        rule_width,
        &"─".repeat(usize::from(rule_width)),
        Style::default().fg(scene.edge),
    );
    put_text(
        buffer,
        rect.right().saturating_sub(right_width),
        rect.y,
        display_width(&count_text),
        &count_text,
        Style::default().fg(color),
    );
    put_right_text(
        buffer,
        rect,
        rect.y,
        &signal,
        auth.badge_style(endpoint, palette, Style::default().fg(color)),
    );
    badge
}

pub(super) fn render_focus_bar(buffer: &mut Buffer, x: u16, rect: Rect) {
    for y in rect.y..rect.bottom() {
        put_text(buffer, x, y, 1, "▎", Style::default().fg(FOCUS_EDGE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(status: AgentStatus, subtitle: &str) -> AgentRow {
        AgentRow {
            pane_id: "w1:p1".into(),
            status,
            focused: false,
            rows: Vec::new(),
            title: "Tare".into(),
            subtitle: subtitle.into(),
        }
    }

    fn row_text(buffer: &Buffer, y: u16) -> String {
        (buffer.area.left()..buffer.area.right())
            .map(|x| buffer[(x, y)].symbol())
            .collect()
    }

    #[test]
    fn sprites_are_rectangular_and_use_known_inks() {
        for sprite in SPRITES {
            for line in sprite {
                assert_eq!(line.len(), SPRITE_COLS, "{line}");
                assert!(
                    line.bytes().all(|key| b".okwgrbypc".contains(&key)),
                    "{line}"
                );
            }
        }
    }

    #[test]
    fn full_card_stays_inside_its_rect_and_closes_its_border() {
        let area = Rect::new(0, 0, 40, 10);
        let rect = Rect::new(1, 1, 36, CARD_ROWS);
        let mut buffer = Buffer::empty(area);
        render_card(
            &mut buffer,
            rect,
            &row(
                AgentStatus::Idle,
                "a very long terminal title that has to wrap and then truncate",
            ),
            scene_for(1),
            "Tare/w1:p1",
        );
        assert!(row_text(&buffer, 1).trim_end().ends_with('┐'));
        assert!(row_text(&buffer, 8).trim_end().ends_with('┘'));
        for y in 2..8 {
            assert_eq!(buffer[(36, y)].symbol(), "│", "row {y}");
            assert_eq!(buffer[(37, y)].symbol(), " ", "row {y}");
        }
        assert!(row_text(&buffer, 3).contains("[IDLE]"));
        assert!(row_text(&buffer, 5).contains("Tare"));
    }

    #[test]
    fn narrow_card_drops_the_art() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 4));
        render_card(
            &mut buffer,
            Rect::new(0, 0, 20, COMPACT_CARD_ROWS),
            &row(AgentStatus::Working, "subtitle"),
            scene_for(0),
            "key",
        );
        assert!(row_text(&buffer, 1).contains("BUSY Tare"));
        assert!(row_text(&buffer, 2).contains("subtitle"));
        assert!(!row_text(&buffer, 1).contains('▀'));
    }

    #[test]
    fn mascot_and_stars_are_stable_for_a_key() {
        assert_eq!(hash("Tare/w1:p1"), hash("Tare/w1:p1"));
        assert_ne!(hash("Tare/w1:p1"), hash("Tare/w1:p2"));
    }

    #[test]
    fn group_header_fits_a_long_machine_name() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 24, 1));
        render_group_header(
            &mut buffer,
            Rect::new(0, 0, 24, 1),
            "christophers-mac-studio",
            scene_for(1),
            0,
            1,
        );
        let text = row_text(&buffer, 0);
        assert!(text.starts_with("CHRISTOPHERS"), "{text}");
        assert!(text.ends_with("DEN 0/1"), "{text}");
    }
}
