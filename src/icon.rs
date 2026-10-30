use std::f32::consts::FRAC_1_SQRT_2;
use tray_icon::Icon;

// tray-icon shows an image at its pixel size in points, capped at 22 pt tall (MAX_ICON_HEIGHT),
// so 44 px is the one size that lands 1:1 on a Retina menu bar.
const SIZE: usize = if cfg!(target_os = "macos") { 44 } else { 32 };
// macOS template images only use alpha; other trays draw the colour as-is on a dark panel.
const INK: u8 = if cfg!(target_os = "macos") { 0 } else { 0xff };
// assets/icon.svg units the bitmap spans. macOS keeps the padding so the glyph sits at the
// system icons' optical size; smaller trays have no room to spare.
const SPAN: f32 = if cfg!(target_os = "macos") {
    512.0
} else {
    416.0
};

// The numbers in assets/icon.svg, whose paths lie along +x before their -45 degree turn.
const STROKE: f32 = 40.0;
const RADIUS: f32 = 70.0;
const NEAR: f32 = 37.0; // centre of the arc that hooks into the other half
const FAR: f32 = 103.0; // centre of the arc at the outer end
const OPEN: f32 = 73.0; // where the intact hook's open side stops
const BURIED: f32 = 41.0; // the trimmed hook's open side stops inside the blade, so it ends flat
// The blade: a level gap, in units above the centre, on whole pixels at 44 px.
const BLADE: std::ops::Range<f32> = 58.0..93.0;

/// Signed distance to one half of the link's stroke, the upper-right one as written.
fn hook(x: f32, y: f32, open: f32) -> f32 {
    let line = |x0: f32, x1: f32, h: f32| (x - x.clamp(x0, x1)).hypot(y - h);
    let ring = |cx: f32| ((x - cx).hypot(y) - RADIUS).abs();
    let mut d = line(NEAR, FAR, RADIUS)
        .min(line(open, FAR, -RADIUS))
        .min((x - NEAR + RADIUS).hypot(y));
    if x >= FAR {
        d = d.min(ring(FAR));
    }
    if x <= NEAR && y >= 0.0 {
        d = d.min(ring(NEAR));
    }
    d - STROKE / 2.0
}

/// Whether a point, in units from the centre with y down, is inked.
fn inked(dx: f32, dy: f32) -> bool {
    let (x, y) = ((dx - dy) * FRAC_1_SQRT_2, (dx + dy) * FRAC_1_SQRT_2);
    hook(-x, -y, OPEN) <= 0.0 || (hook(x, y, BURIED) <= 0.0 && !BLADE.contains(&-dy))
}

fn rgba() -> Vec<u8> {
    const SAMPLES: usize = 4;
    let at = |px: usize, sub: usize| {
        ((px * SAMPLES + sub) as f32 + 0.5) * SPAN / (SIZE * SAMPLES) as f32 - SPAN / 2.0
    };
    (0..SIZE * SIZE)
        .flat_map(|i| {
            let hits = (0..SAMPLES * SAMPLES)
                .filter(|s| inked(at(i % SIZE, s % SAMPLES), at(i / SIZE, s / SAMPLES)))
                .count();
            [INK, INK, INK, (hits * 255 / (SAMPLES * SAMPLES)) as u8]
        })
        .collect()
}

pub fn tray_icon() -> Icon {
    Icon::from_rgba(rgba(), SIZE as u32, SIZE as u32).expect("icon is square RGBA")
}

#[cfg(test)]
mod tests {
    #[test]
    fn icon_is_valid_rgba() {
        super::tray_icon();
        let rgba = super::rgba();
        assert!(rgba.chunks(4).any(|p| p[3] == 0));
        assert!(rgba.chunks(4).any(|p| p[3] == 0xff));
    }
}
