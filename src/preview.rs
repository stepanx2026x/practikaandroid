//! Режим предпросмотра графики: `sakura-yokai --preview <папка>` сохраняет листы со спрайтами в PNG.

use resvg::tiny_skia::{Pixmap, PixmapPaint, Transform};

use crate::art::{self, chibi};
use crate::data::*;

fn sheet(items: &[(String, u32, u32)], cols: u32, cell_w: u32, cell_h: u32, bg: [u8; 4]) -> Pixmap {
    let rows = (items.len() as u32).div_ceil(cols);
    let mut pm = Pixmap::new(cols * cell_w, rows * cell_h).unwrap();
    pm.fill(resvg::tiny_skia::Color::from_rgba8(
        bg[0], bg[1], bg[2], bg[3],
    ));
    for (i, (svg, w, h)) in items.iter().enumerate() {
        if let Some(p) = art::rasterize(svg, *w, *h) {
            let x = (i as u32 % cols) * cell_w + (cell_w - w) / 2;
            let y = (i as u32 / cols) * cell_h + (cell_h - h) / 2;
            pm.draw_pixmap(
                x as i32,
                y as i32,
                p.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );
        }
    }
    pm
}

pub fn run(dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    let mut items = Vec::new();
    for h in HEROINES.iter() {
        items.push((chibi::heroine_svg(&h.look, false, !h.clicker), 240, 330));
    }
    for o in OUTFITS.iter() {
        items.push((chibi::heroine_svg(&sakura_look(o), false, false), 240, 330));
    }
    items.push((chibi::heroine_svg(&HEROINES[0].look, true, false), 240, 330));
    items.push((chibi::katana_svg(), 64, 240));
    sheet(&items, 8, 250, 340, [255, 240, 245, 255])
        .save_png(format!("{dir}/heroines.png"))
        .unwrap();

    let mut portraits = Vec::new();
    for h in HEROINES.iter() {
        portraits.push((chibi::portrait_svg(&h.look), 120, 120));
    }
    sheet(&portraits, 7, 130, 130, [255, 240, 245, 255])
        .save_png(format!("{dir}/portraits.png"))
        .unwrap();
    let mut mons = Vec::new();
    for m in MONSTERS.iter() {
        mons.push((art::yokai::monster_svg(m), 240, 240));
    }
    sheet(&mons, 7, 250, 250, [235, 228, 240, 255])
        .save_png(format!("{dir}/yokai.png"))
        .unwrap();
    // локации: задний + передний слой
    let (cw, ch) = (480u32, 470u32);
    let mut pm = Pixmap::new(cw * 4, ch * 2).unwrap();
    for z in 0..ZONES.len() {
        let (bg, _) = art::scenery::backdrop(z, 820.0, 800.0);
        let fg = art::scenery::foreground(z, 820.0, 800.0);
        let x = (z as u32 % 4) * cw;
        let y = (z as u32 / 4) * ch;
        for svg in [bg, fg] {
            if let Some(p) = art::rasterize(&svg, cw - 10, ch - 10) {
                pm.draw_pixmap(
                    x as i32,
                    y as i32,
                    p.as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            }
        }
    }
    pm.save_png(format!("{dir}/zones.png")).unwrap();
    let big = vec![
        (
            chibi::heroine_svg(&HEROINES[0].look, false, false),
            480,
            660,
        ),
        (chibi::heroine_svg(&HEROINES[7].look, false, true), 480, 660),
    ];
    sheet(&big, 2, 500, 680, [255, 240, 245, 255])
        .save_png(format!("{dir}/big.png"))
        .unwrap();
    println!("preview saved to {dir}");
}
