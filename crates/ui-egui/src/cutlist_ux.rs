//! Cut lists, UX prototype (#137): the Cut List, Layout on Stock and Stock Library windows the
//! Inspect tab's Fabricate section opens, and the Stock size and Fabrication sections of the
//! Properties card. Everything here is sample data for the plywood workbench (eight 18 mm panels,
//! four 2440 × 1220 sheets); no command computes it yet. It exists to settle the experience before
//! the engine work, and is replaced by `cutlist.*` results when those are built.

use egui::{Color32, FontId, Rect, Stroke, Ui, pos2, vec2};

use crate::SolveApp;
use crate::theme::Tokens;
use crate::ui_kit::{self, Column};

/// What the open windows show; the parts and sheets are fixed sample data.
#[derive(Default)]
pub struct State {
    pub cut_list: bool,
    pub layout: bool,
    pub stock: bool,
    /// Cut List: the picked row and units (0 mm, 1 in).
    pub picked: Option<usize>,
    pub units: usize,
    /// Layout on Stock: grain locked on every part, the method and sheet size.
    pub lock_grain: bool,
    pub method: usize,
    pub sheet_choice: usize,
    /// Stock Library: the picked row.
    pub stock_picked: Option<usize>,
    /// Properties: the fabrication fields of the asked-about part.
    pub in_cut_list: Option<bool>,
    pub material: usize,
    pub grain: usize,
}

/// One cut-list row: identical parts grouped, sizes in mm along the part.
struct Part {
    name: &'static str,
    members: &'static str,
    qty: u32,
    length: f64,
    width: f64,
    thickness: f64,
    grain: &'static str,
}

const PARTS: &[Part] = &[
    Part { name: "Top layer", members: "upper, lower", qty: 2, length: 1828.8, width: 762.0, thickness: 18.0, grain: "Length" },
    Part { name: "End / center support", members: "left, center, right", qty: 3, length: 878.4, width: 591.6, thickness: 18.0, grain: "Any" },
    Part { name: "Rear bracing panel", members: "", qty: 1, length: 1676.4, width: 778.4, thickness: 18.0, grain: "Any" },
    Part { name: "Shelf", members: "left, right", qty: 2, length: 811.2, width: 591.6, thickness: 18.0, grain: "Any" },
];

/// A sheet 2440 × 1220 (4 × 8 ft), as laid out.
const SHEET: (f64, f64) = (2438.4, 1219.2);
const TRIM: f64 = 10.0;

/// A placed part or a kept offcut on a sheet: name, x, y, width, height (mm from the sheet's
/// trimmed corner), turned 90°.
type Placed = (&'static str, f64, f64, f64, f64, bool);

const OFFCUT: &str = "";

const LAYOUT_FREE: &[&[Placed]] = &[
    &[("Top layer", 0.0, 0.0, 1828.8, 762.0, false), (OFFCUT, 1832.0, 0.0, 586.0, 1199.0, false), (OFFCUT, 0.0, 765.2, 1828.8, 434.0, false)],
    &[("Top layer", 0.0, 0.0, 1828.8, 762.0, false), (OFFCUT, 1832.0, 0.0, 586.0, 1199.0, false), (OFFCUT, 0.0, 765.2, 1828.8, 434.0, false)],
    &[("Rear bracing panel", 0.0, 0.0, 1676.4, 778.4, false), ("Left end", 1679.6, 0.0, 591.6, 878.4, true)],
    &[
        ("Center support", 0.0, 0.0, 878.4, 591.6, false),
        ("Right end", 0.0, 594.8, 878.4, 591.6, false),
        ("Left shelf", 881.6, 0.0, 811.2, 591.6, false),
        ("Right shelf", 881.6, 594.8, 811.2, 591.6, false),
    ],
];

const LAYOUT_GRAIN: &[&[Placed]] = &[
    &[("Top layer", 0.0, 0.0, 1828.8, 762.0, false), (OFFCUT, 0.0, 765.2, 1828.8, 434.0, false)],
    &[("Top layer", 0.0, 0.0, 1828.8, 762.0, false), (OFFCUT, 0.0, 765.2, 1828.8, 434.0, false)],
    &[("Rear bracing panel", 0.0, 0.0, 1676.4, 778.4, false)],
    &[
        ("Left end", 0.0, 0.0, 878.4, 591.6, false),
        ("Center support", 0.0, 594.8, 878.4, 591.6, false),
        ("Left shelf", 881.6, 0.0, 811.2, 591.6, false),
        ("Right shelf", 881.6, 594.8, 811.2, 591.6, false),
    ],
    &[("Right end", 0.0, 0.0, 878.4, 591.6, false)],
];

/// A stock library row: name, kind, thickness, size, lengths, price.
type Stock = (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str);

const STOCK: &[Stock] = &[
    ("Birch plywood 18 mm", "Sheet", "18", "2440 × 1220", "", "$68.00"),
    ("Birch plywood 6 mm", "Sheet", "6", "2440 × 1220", "", "$32.00"),
    ("MDF 18 mm", "Sheet", "18", "2440 × 1220", "", "—"),
    ("Plywood ¾″ (imperial)", "Sheet", "23/32″", "96″ × 48″", "", "—"),
    ("Oak 19 × 45", "Board", "19", "45", "2400, 2700", "$9.50 /m"),
    ("Pine 2×4 (nominal)", "Board", "38", "89", "2440, 3050", "$4.20"),
    ("Pine 1×6 (nominal)", "Board", "19", "140", "2440", "—"),
    ("Aluminium extrusion 20 × 20", "Length", "20", "20", "1000, 2000", "—"),
    ("Steel square tube 25 × 25 × 2", "Length", "25", "25", "6000", "—"),
];

fn part_area(p: &Placed) -> f64 {
    if p.0 == OFFCUT { 0.0 } else { p.3 * p.4 }
}

fn layout(s: &State) -> &'static [&'static [Placed]] {
    if s.lock_grain { LAYOUT_GRAIN } else { LAYOUT_FREE }
}

/// A length in the window's units.
fn len(s: &State, mm: f64) -> String {
    if s.units == 1 { format!("{:.2}", mm / 25.4) } else { format!("{mm:.1}") }
}

/// The toolbar's Fabricate commands open their window instead of running.
pub fn start(app: &mut SolveApp, id: &str) -> bool {
    let s = &mut app.cutlist;
    match id {
        "cutlist.parts" => s.cut_list = true,
        "cutlist.layout" => s.layout = true,
        "cutlist.stock" => s.stock = true,
        _ => return false,
    }
    true
}

/// Draw the open windows. Called once per frame.
pub fn show(app: &mut SolveApp, ctx: &egui::Context) {
    let mut s = std::mem::take(&mut app.cutlist);
    if s.cut_list {
        cut_list(&mut s, ctx, "plywood-workbench");
    }
    if s.layout {
        layout_window(&mut s, ctx);
    }
    if s.stock {
        stock_window(&mut s, ctx);
    }
    app.cutlist = s;
}

fn cut_list(s: &mut State, ctx: &egui::Context, design: &str) {
    let t = Tokens::get();
    let parts: u32 = PARTS.iter().map(|p| p.qty).sum();
    let mut export = false;
    let (closed, action) = ui_kit::Window::new(egui::Id::new("sc_cutlist"), "list", "Cut List")
        .resizable()
        .cards(3.4)
        .placed(egui::Align2::RIGHT_TOP, pos2(ctx.content_rect().right() - t.space_24 * 2.0, t.space_24 * 7.0))
        .actions("Layout on Stock…", "Close", None)
        .show_with_left(
            ctx,
            true,
            |ui| {
                ui_kit::text(ui, &format!("{design} · {parts} parts in {} rows · 1 material", PARTS.len()), ui_kit::Text::Note("cutlist_summary"));
                ui.horizontal(|ui| {
                    let options = ["mm", "in"];
                    ui_kit::text(ui, "Units", ui_kit::Text::Inline);
                    ui.allocate_ui(vec2(ui_kit::ContentWidth::dropdown(ui, &options), ui_kit::row_height()), |ui| {
                        ui_kit::dropdown(ui, "cutlist_units", &mut s.units, &options);
                    });
                });
                ui_kit::section_count(ui, "cutlist_plywood", "Plywood 18 mm", true, Some(parts as usize), |ui| {
                    let unit = if s.units == 1 { "in" } else { "mm" };
                    let (l, w, th) = (format!("Length ({unit})"), format!("Width ({unit})"), format!("Thick. ({unit})"));
                    let columns = [
                        Column::new("#", 0.25),
                        Column::new("Part", 2.2).flex(),
                        Column::new("Qty", 0.35),
                        Column::new(&l, 0.8),
                        Column::new(&w, 0.8),
                        Column::new(&th, 0.7),
                        Column::new("Grain", 0.6),
                    ];
                    ui_kit::table(ui, "cutlist", &columns, "No parts", |ui, table| {
                        for (i, p) in PARTS.iter().enumerate() {
                            let picked = s.picked == Some(i);
                            let row = table.row(ui, p.name, picked, |row| {
                                row.cell(|ui| ui_kit::text(ui, &(i + 1).to_string(), ui_kit::Text::Cell));
                                row.cell(|ui| {
                                    let name = if p.members.is_empty() { p.name.to_string() } else { format!("{}  ({})", p.name, p.members) };
                                    ui_kit::text(ui, &name, ui_kit::Text::NameCell(picked))
                                });
                                row.cell(|ui| ui_kit::text(ui, &format!("× {}", p.qty), ui_kit::Text::Cell));
                                row.cell(|ui| ui_kit::text(ui, &len(s, p.length), ui_kit::Text::MonoCell));
                                row.cell(|ui| ui_kit::text(ui, &len(s, p.width), ui_kit::Text::MonoCell));
                                row.cell(|ui| ui_kit::text(ui, &len(s, p.thickness), ui_kit::Text::MonoCell));
                                row.cell(|ui| ui_kit::text(ui, p.grain, ui_kit::Text::Cell));
                            });
                            if row.clicked() {
                                s.picked = if picked { None } else { Some(i) };
                            }
                        }
                    });
                });
                if let Some(p) = s.picked.and_then(|i| PARTS.get(i)) {
                    ui_kit::text(ui, &format!("Picked: {} × {} — highlighted in the viewport", p.qty, p.name), ui_kit::Text::Note("cutlist_picked"));
                }
                ui_kit::section_count(ui, "cutlist_checks", "Checks", true, Some(3), |ui| {
                    check(ui, Check::Ok, "All 8 bodies have a material and an exact stock box");
                    check(ui, Check::Note, "No fasteners modelled: screws and glue won't appear in the shopping list");
                    check(ui, Check::Live, "Sizes follow parameters: change bench_depth and this list updates");
                });
            },
            |ui| {
                for label in ["Export CSV…", "Export JSON…"] {
                    if ui_kit::button(ui, label, ui_kit::Button::Normal).clicked() {
                        export = true;
                    }
                }
            },
        );
    let _ = export;
    if action == ui_kit::Action::Ok {
        s.layout = true;
    }
    if closed || action == ui_kit::Action::Cancel {
        s.cut_list = false;
    }
}

#[derive(Clone, Copy)]
enum Check {
    Ok,
    Note,
    Live,
}

fn check(ui: &mut Ui, kind: Check, text: &str) {
    let t = Tokens::get();
    let (tag, fill, ink) = match kind {
        Check::Ok => ("OK", t.form_success_fill, t.form_success),
        Check::Note => ("Note", t.accent_soft, t.form_ink),
        Check::Live => ("Live", t.accent_soft, t.accent),
    };
    ui.horizontal(|ui| {
        egui::Frame::NONE.fill(fill).corner_radius(t.radius_sm).inner_margin(egui::Margin::symmetric(6, 1)).show(ui, |ui| {
            ui.label(egui::RichText::new(tag).font(FontId::proportional(t.font_small)).color(ink));
        });
        ui_kit::text(ui, text, ui_kit::Text::Inline);
    });
}

fn layout_window(s: &mut State, ctx: &egui::Context) {
    let t = Tokens::get();
    let sheets = layout(s);
    let sheet_area = SHEET.0 * SHEET.1;
    let used: f64 = sheets.iter().flat_map(|sh| sh.iter()).map(part_area).sum();
    let overall = used / (sheet_area * sheets.len() as f64) * 100.0;
    let (closed, action) = ui_kit::Window::new(egui::Id::new("sc_cutlist_layout"), "sheet", "Layout on Stock")
        .resizable()
        .cards(4.6)
        .actions("", "Close", None)
        .show_with_left(
            ctx,
            true,
            |ui| {
                ui.horizontal_top(|ui| {
                    let side = t.form_label_width * 3.0;
                    ui.vertical(|ui| {
                        ui.set_width(side);
                        {
                            ui_kit::section(ui, "layout_stock", "Stock per material", true, |ui| {
                                ui_kit::text(ui, "Plywood 18 mm", ui_kit::Text::Inline);
                                let options = ["4 × 8 ft (2440 × 1220)", "5 × 5 ft (1525 × 1525)", "From library…"];
                                ui_kit::dropdown(ui, "layout_sheet", &mut s.sheet_choice, &options);
                            });
                            ui_kit::section(ui, "layout_cutting", "Cutting", true, |ui| {
                                ui_kit::row(ui, "Kerf (blade)", |ui| ui_kit::text(ui, "3.2 mm", ui_kit::Text::Cell));
                                ui_kit::row(ui, "Edge trim", |ui| ui_kit::text(ui, "10 mm", ui_kit::Text::Cell));
                                ui_kit::choice_row(ui, "Method", "layout_method", &mut s.method, &["Guillotine (saw)", "Free (CNC)"]);
                                ui_kit::text(ui, "Guillotine: every cut runs edge to edge. Free: CNC.", ui_kit::Text::Note("layout_method_note"));
                            });
                            ui_kit::section(ui, "layout_grain", "Grain", true, |ui| {
                                ui_kit::checkbox(ui, "Lock grain on every part", &mut s.lock_grain);
                                let note = if s.lock_grain {
                                    "Locked: the ends and shelves can no longer turn 90° beside the rear brace, so this bench needs 5 sheets."
                                } else {
                                    "Locking grain on the ends and shelves needs 5 sheets instead of 4."
                                };
                                ui_kit::text(ui, note, ui_kit::Text::Note("layout_grain_note"));
                            });
                        }
                    });
                    ui.add_space(t.space_16);
                    ui.vertical(|ui| {
                        ui_kit::text(
                            ui,
                            &format!("Plywood 18 mm · {} sheets of 2440 × 1220 · rip, then crosscut · overall yield {overall:.0}%", sheets.len()),
                            ui_kit::Text::Heading("layout_summary"),
                        );
                        let width = ui.available_width();
                        let cell = (width - t.space_16) / 2.0;
                        // Identical sheets in a row are drawn once, with their count.
                        let mut groups: Vec<(usize, usize, &[Placed])> = Vec::new();
                        for (i, sheet) in sheets.iter().enumerate() {
                            match groups.last_mut() {
                                Some((_, count, last)) if *last == *sheet => *count += 1,
                                _ => groups.push((i + 1, 1, sheet)),
                            }
                        }
                        for pair in groups.chunks(2) {
                            ui.horizontal(|ui| {
                                for (first, count, sheet) in pair {
                                    let yield_ = sheet.iter().map(part_area).sum::<f64>() / sheet_area * 100.0;
                                    let title = if *count > 1 {
                                        format!("Sheets {first}–{} · × {count} · {yield_:.0}% each", first + count - 1)
                                    } else {
                                        format!("Sheet {first} · {yield_:.0}%")
                                    };
                                    ui.vertical(|ui| {
                                        ui.set_width(cell);
                                        ui_kit::text(ui, &title, ui_kit::Text::Inline);
                                        draw_sheet(ui, cell, sheet);
                                    });
                                }
                            });
                            ui.add_space(t.space_8);
                        }
                        ui_kit::section(ui, "layout_buy", "To buy", true, |ui| {
                            ui_kit::row(ui, &format!("{} × Plywood 18 mm", sheets.len()), |ui| {
                                ui_kit::text(ui, &format!("2440 × 1220 (4 × 8 ft) · ${:.2}", 68.0 * sheets.len() as f64), ui_kit::Text::Cell)
                            });
                            let why = if s.lock_grain {
                                "Why 5: with grain locked, the third end can't share a sheet with the rear brace."
                            } else {
                                "Why 4: each 762 mm top layer needs its own sheet; two don't fit across 1220."
                            };
                            ui_kit::text(ui, why, ui_kit::Text::Note("layout_why"));
                        });
                        ui_kit::section(ui, "layout_offcuts", "Offcuts", true, |ui| {
                            ui_kit::text(ui, "2 × 1829 × 434 · 2 × 586 × 1199, plus strips under the rear brace", ui_kit::Text::Inline);
                        });
                    });
                });
            },
            |ui| {
                for label in ["SVG…", "DXF…", "CSV…"] {
                    let _ = ui_kit::button(ui, label, ui_kit::Button::Normal);
                }
                ui_kit::text(ui, "All 8 parts placed", ui_kit::Text::Inline);
            },
        );
    if closed || action == ui_kit::Action::Cancel {
        s.layout = false;
    }
}

/// One sheet to scale: parts filled, offcuts dashed, turned parts marked.
fn draw_sheet(ui: &mut Ui, width: f32, sheet: &[Placed]) {
    let t = Tokens::get();
    let scale = width / SHEET.0 as f32;
    let (rect, _) = ui.allocate_exact_size(vec2(width, SHEET.1 as f32 * scale), egui::Sense::hover());
    let painter = ui.painter_at(rect.expand(1.0));
    painter.rect(rect, 2.0, t.dialog_footer, Stroke::new(1.0, t.dialog_rule), egui::StrokeKind::Inside);
    let origin = rect.min + vec2(TRIM as f32 * scale, TRIM as f32 * scale);
    for (name, x, y, w, h, turned) in sheet {
        let r = Rect::from_min_size(origin + vec2(*x as f32 * scale, *y as f32 * scale), vec2(*w as f32 * scale, *h as f32 * scale));
        if *name == OFFCUT {
            for side in [[r.left_top(), r.right_top()], [r.right_top(), r.right_bottom()], [r.right_bottom(), r.left_bottom()], [r.left_bottom(), r.left_top()]] {
                painter.extend(egui::Shape::dashed_line(&side, Stroke::new(1.0, t.text_dim), 4.0, 3.0));
            }
            painter.text(r.center(), egui::Align2::CENTER_CENTER, format!("offcut {:.0} × {:.0}", w, h), FontId::proportional(t.font_micro), t.text_dim);
            continue;
        }
        let fill = Color32::from_rgba_unmultiplied(t.accent.r(), t.accent.g(), t.accent.b(), 46);
        painter.rect(r, 1.0, fill, Stroke::new(1.0, t.accent), egui::StrokeKind::Inside);
        // Grain runs along the part's length: across the sheet, or down it when turned.
        let grain = Stroke::new(0.6, Color32::from_rgba_unmultiplied(t.accent.r(), t.accent.g(), t.accent.b(), 60));
        let step = 9.0;
        if *turned {
            let mut x = r.left() + step;
            while x < r.right() - 2.0 {
                painter.line_segment([pos2(x, r.top() + 3.0), pos2(x, r.bottom() - 3.0)], grain);
                x += step;
            }
        } else {
            let mut y = r.top() + step;
            while y < r.bottom() - 2.0 {
                painter.line_segment([pos2(r.left() + 3.0, y), pos2(r.right() - 3.0, y)], grain);
                y += step;
            }
        }
        let label = Rect::from_center_size(r.center(), vec2((r.width() - 8.0).min(150.0), t.font_small * 2.6));
        painter.rect_filled(label, 3.0, t.dialog_body.gamma_multiply(0.85));
        let size = if *turned { format!("{:.0} × {:.0} · turned 90°", h, w) } else { format!("{:.0} × {:.0}", w, h) };
        painter.text(r.center() - vec2(0.0, t.font_small * 0.6), egui::Align2::CENTER_CENTER, *name, FontId::proportional(t.font_small), t.form_ink);
        painter.text(r.center() + vec2(0.0, t.font_small * 0.7), egui::Align2::CENTER_CENTER, size, FontId::proportional(t.font_micro), t.form_dim);
    }
}

fn stock_window(s: &mut State, ctx: &egui::Context) {
    let (closed, action) = ui_kit::Window::new(egui::Id::new("sc_cutlist_stock"), "library", "Stock Library")
        .resizable()
        .cards(3.2)
        .actions("", "Close", None)
        .show(ctx, true, |ui| stock_table(ui, s));
    if closed || action == ui_kit::Action::Cancel {
        s.stock = false;
    }
}

fn stock_table(ui: &mut Ui, s: &mut State) {
    ui_kit::text(ui, "What you can buy. Shared by every design on this computer.", ui_kit::Text::Note("stock_intro"));
    let columns = [
        Column::new("Name", 1.6).flex(),
        Column::new("Kind", 0.5),
        Column::new("Thick.", 0.7),
        Column::new("Size (actual)", 1.4),
        Column::new("Lengths", 1.0),
        Column::new("Price", 0.6),
    ];
    ui_kit::table(ui, "stock", &columns, "No stock", |ui, table| {
        for (i, r) in STOCK.iter().enumerate() {
            let picked = s.stock_picked == Some(i);
            let row = table.row(ui, r.0, picked, |row| {
                row.cell(|ui| ui_kit::text(ui, r.0, ui_kit::Text::NameCell(picked)));
                for v in [r.1, r.2, r.3, r.4, r.5] {
                    row.cell(|ui| ui_kit::text(ui, v, ui_kit::Text::Cell));
                }
            });
            if row.clicked() {
                s.stock_picked = if picked { None } else { Some(i) };
            }
        }
    });
    let _ = ui_kit::button(ui, "+ Add stock", ui_kit::Button::Normal);
    if let Some(r) = s.stock_picked.and_then(|i| STOCK.get(i)) {
        ui_kit::section(ui, "stock_detail", r.0, true, |ui| {
            for (label, value) in [("Size (actual)", r.3), ("Lengths", if r.4.is_empty() { "—" } else { r.4 }), ("Grain", "Along length"), ("Price", r.5)] {
                ui_kit::row(ui, label, |ui| ui_kit::text(ui, value, ui_kit::Text::Cell));
            }
            ui_kit::text(ui, "Cut-list rows of this material and thickness use this stock when laid out.", ui_kit::Text::Note("stock_match"));
        });
    }
}

/// The Properties card's Stock size and Fabrication sections for a body whose world box is
/// `dims` (mm). The stock box here is the world box sorted longest first; the real one follows
/// the part.
pub fn properties(ui: &mut Ui, s: &mut State, dims: [f64; 3]) {
    let mut d = dims;
    d.sort_by(|a, b| b.total_cmp(a));
    ui_kit::section(ui, "props_stock", "Stock size", true, |ui| {
        for (label, v) in ["Length", "Width", "Thickness"].into_iter().zip(d) {
            ui_kit::row(ui, label, |ui| ui_kit::text(ui, &format!("{v:.1} mm   ƒx"), ui_kit::Text::Cell));
        }
        ui_kit::text(ui, "ƒx: use as a parameter, e.g. Left_end.stock.length", ui_kit::Text::Note("props_stock_fx"));
    });
    ui_kit::section(ui, "props_fabrication", "Fabrication", true, |ui| {
        let mut on = s.in_cut_list.unwrap_or(true);
        ui_kit::checkbox(ui, "Stock part (in cut list)", &mut on);
        s.in_cut_list = Some(on);
        ui_kit::choice_row(ui, "Material", "props_material", &mut s.material, &["Plywood 18 mm", "Plywood 12 mm", "Oak 19 × 45"]);
        ui_kit::choice_row(ui, "Grain", "props_grain", &mut s.grain, &["Any (structural)", "Along length", "Along width"]);
        ui_kit::text(ui, "\"Along length\" costs this bench a 5th sheet.", ui_kit::Text::Note("props_grain_note"));
    });
}
