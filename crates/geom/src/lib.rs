//! WordCraft geometry and units.
//!
//! Layout works in **points** (1/72 inch, `f32`). File formats use other units: OOXML uses
//! twentieths of a point (twips) for most lengths, half-points for font sizes, eighths of a point
//! for borders and English Metric Units (EMU, 914 400 per inch) for drawings. The conversions live
//! here so every crate agrees on them.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use serde::{Deserialize, Serialize};

mod region;
pub use region::Region;

/// Points per inch.
pub const PT_PER_IN: f32 = 72.0;
/// Twips (1/20 pt) per point.
pub const TWIPS_PER_PT: f32 = 20.0;
/// English Metric Units per point.
pub const EMU_PER_PT: f32 = 12_700.0;
/// Points per centimetre.
pub const PT_PER_CM: f32 = 72.0 / 2.54;
/// Centimetres per inch (exact).
pub const CM_PER_IN: f32 = 2.54;
/// Inches per centimetre.
pub const IN_PER_CM: f32 = 1.0 / 2.54;
/// Millimetres per inch.
pub const MM_PER_IN: f32 = 25.4;
/// Points per millimetre.
pub const PT_PER_MM: f32 = 72.0 / 25.4;

/// Twips → points.
pub fn twips(v: i64) -> f32 {
    v as f32 / TWIPS_PER_PT
}
/// Points → twips (rounded).
pub fn to_twips(pt: f32) -> i64 {
    finite(pt * TWIPS_PER_PT).round() as i64
}
/// EMU → points.
pub fn emu(v: i64) -> f32 {
    v as f32 / EMU_PER_PT
}
/// Points → EMU (rounded).
pub fn to_emu(pt: f32) -> i64 {
    (finite(pt) as f64 * EMU_PER_PT as f64).round() as i64
}
/// Half-points (OOXML font sizes) → points.
pub fn half_points(v: i64) -> f32 {
    v as f32 / 2.0
}
/// Points → half-points (rounded).
pub fn to_half_points(pt: f32) -> i64 {
    (finite(pt) * 2.0).round() as i64
}

/// Inches → centimetres.
pub fn in_to_cm(inches: f32) -> f32 {
    finite(inches) * CM_PER_IN
}
/// Centimetres → inches.
pub fn cm_to_in(cm: f32) -> f32 {
    finite(cm) * IN_PER_CM
}
/// Centimetres → points.
pub fn cm_to_pt(cm: f32) -> f32 {
    finite(cm) * PT_PER_CM
}
/// Points → centimetres.
pub fn pt_to_cm(pt: f32) -> f32 {
    finite(pt) / PT_PER_CM
}

/// Replace NaN/inf with 0 so casts and arithmetic stay defined.
pub fn finite(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

/// A measurement unit shown in the UI (Word's Options › Advanced › "Show measurements in units of").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    #[default]
    Inches,
    Centimeters,
    Millimeters,
    Points,
    Picas,
}

/// All measurement units, in the order Word's Options dialog shows them.
pub const ALL_UNITS: [Unit; 5] = [Unit::Inches, Unit::Centimeters, Unit::Millimeters, Unit::Points, Unit::Picas];

impl Unit {
    /// All units in display order.
    pub const fn all() -> [Unit; 5] {
        ALL_UNITS
    }
    /// Human-readable label (`"Inches"`, `"Centimeters"`, …).
    pub fn label(self) -> &'static str {
        match self {
            Unit::Inches => "Inches",
            Unit::Centimeters => "Centimeters",
            Unit::Millimeters => "Millimeters",
            Unit::Points => "Points",
            Unit::Picas => "Picas",
        }
    }
    /// Parse a unit name (`"inches"`, `"cm"`, `"pt"`, …). Case-insensitive.
    pub fn parse(s: &str) -> Option<Unit> {
        match s.trim().to_ascii_lowercase().as_str() {
            "inches" | "inch" | "in" | "\"" => Some(Unit::Inches),
            "centimeters" | "centimetres" | "cm" => Some(Unit::Centimeters),
            "millimeters" | "millimetres" | "mm" => Some(Unit::Millimeters),
            "points" | "point" | "pt" | "pts" => Some(Unit::Points),
            "picas" | "pica" | "pi" | "pc" => Some(Unit::Picas),
            _ => None,
        }
    }
    pub fn suffix(self) -> &'static str {
        match self {
            Unit::Inches => "\"",
            Unit::Centimeters => " cm",
            Unit::Millimeters => " mm",
            Unit::Points => " pt",
            Unit::Picas => " pi",
        }
    }
    pub fn pt_per_unit(self) -> f32 {
        match self {
            Unit::Inches => PT_PER_IN,
            Unit::Centimeters => PT_PER_CM,
            Unit::Millimeters => PT_PER_CM / 10.0,
            Unit::Points => 1.0,
            Unit::Picas => 12.0,
        }
    }
    /// Points → value in this unit.
    pub fn from_pt(self, pt: f32) -> f32 {
        finite(pt) / self.pt_per_unit()
    }
    /// Value in this unit → points.
    pub fn to_pt(self, value: f32) -> f32 {
        finite(value) * self.pt_per_unit()
    }
    /// Convert a value from `self` to `target` unit.
    ///
    /// ```
    /// # use wordcraft_geom::Unit;
    /// let cm = Unit::Inches.convert(1.0, Unit::Centimeters);
    /// assert!((cm - 2.54).abs() < 0.01);
    /// ```
    pub fn convert(self, value: f32, target: Unit) -> f32 {
        if self == target {
            return finite(value);
        }
        let pt = finite(value) * self.pt_per_unit();
        pt / target.pt_per_unit()
    }
    /// Format `pt` in this unit the way Word's fields show it (`1"`, `2.54 cm`, `12 pt`).
    pub fn format(self, pt: f32) -> String {
        self.format_with(pt, '.')
    }
    /// [`Unit::format`] with the region's decimal separator (`2,54 cm` with `,`).
    pub fn format_with(self, pt: f32, decimal: char) -> String {
        format!("{}{}", self.number(pt, decimal), self.suffix())
    }
    /// `pt` as a number in this unit, at most two decimals, without the unit (`2,54` with `,`).
    pub fn number(self, pt: f32, decimal: char) -> String {
        let v = finite(pt) / self.pt_per_unit();
        let s = format!("{v:.2}");
        let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.') } else { &s };
        let s = if s == "-0" { "0" } else { s };
        s.replace('.', &decimal.to_string())
    }
}

/// Parse a measurement typed by a user: `1"`, `1 in`, `2.5cm`, `12pt`, `3 pi`, `10 mm`, `1.5 li`
/// (lines are 12 pt). A bare number is in `default`. Returns points. Either decimal separator
/// works (`2,5 cm` and `2.5 cm`), whatever the region.
pub fn parse_measure(s: &str, default: Unit) -> Option<f32> {
    let t = s.trim().to_ascii_lowercase().replace('\u{2212}', "-");
    if t.is_empty() {
        return None;
    }
    let split = t.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == ',')).unwrap_or(t.len());
    let (num, unit) = t.split_at(split);
    let v: f32 = num.replace(',', ".").parse().ok()?;
    if !v.is_finite() {
        return None;
    }
    let k = match unit.trim() {
        "" => default.pt_per_unit(),
        "\"" | "in" | "inch" | "inches" => PT_PER_IN,
        "cm" => PT_PER_CM,
        "mm" => PT_PER_CM / 10.0,
        "pt" | "pts" | "point" | "points" => 1.0,
        "pi" | "pc" | "pica" | "picas" => 12.0,
        "li" | "line" | "lines" => 12.0,
        "px" => 0.75,
        _ => return None,
    };
    Some(v * k)
}

/// Snap `pt` to the nearest multiple of `grid_pt` (both in points). Useful for ruler snapping
/// and grid alignment. Returns `pt` unchanged if `grid_pt` is zero, negative or non-finite.
pub fn snap_to_grid(pt: f32, grid_pt: f32) -> f32 {
    if !grid_pt.is_finite() || grid_pt <= 0.0 {
        return finite(pt);
    }
    (finite(pt) / grid_pt).round() * grid_pt
}

/// Clamp a margin (in points) so it stays within `0..=max_pt`. Word silently clamps margins that
/// would make the text area zero or negative.
pub fn clamp_margin(pt: f32, max_pt: f32) -> f32 {
    finite(pt).clamp(0.0, finite(max_pt).max(0.0))
}

/// A point in page space (points, y down).
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

/// An axis-aligned rectangle (points, y down). `w`/`h` are never negative when built with [`Rect::new`].
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x: finite(x), y: finite(y), w: finite(w).max(0.0), h: finite(h).max(0.0) }
    }
    /// The centre of this rectangle.
    pub fn center(&self) -> Point {
        Point::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x <= self.right() && p.y >= self.y && p.y <= self.bottom()
    }
    pub fn intersects(&self, o: &Rect) -> bool {
        self.x < o.right() && o.x < self.right() && self.y < o.bottom() && o.y < self.bottom()
    }
    /// The overlapping area of two rectangles, or `None` if they don't overlap.
    pub fn intersection(&self, o: &Rect) -> Option<Rect> {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        let r = self.right().min(o.right());
        let b = self.bottom().min(o.bottom());
        if r > x && b > y { Some(Rect::new(x, y, r - x, b - y)) } else { None }
    }
    pub fn union(&self, o: &Rect) -> Rect {
        let x = self.x.min(o.x);
        let y = self.y.min(o.y);
        Rect::new(x, y, self.right().max(o.right()) - x, self.bottom().max(o.bottom()) - y)
    }
    pub fn inset(&self, l: f32, t: f32, r: f32, b: f32) -> Rect {
        Rect::new(self.x + l, self.y + t, self.w - l - r, self.h - t - b)
    }
    /// Grown by `d` on every side (shrunk for negative `d`, down to zero size).
    pub fn expand(&self, d: f32) -> Rect {
        self.inset(-d, -d, -d, -d)
    }
}

/// An angle in degrees brought into `0..360` (NaN and infinities become 0).
pub fn normalize_degrees(deg: f32) -> f32 {
    if !deg.is_finite() {
        return 0.0;
    }
    let d = deg.rem_euclid(360.0);
    // `rem_euclid` can round up to exactly 360 for tiny negative angles.
    if (0.0..360.0).contains(&d) { d } else { 0.0 }
}

/// How a drawing is turned about its centre: mirrored first (`flip_h` left to right, `flip_v` top
/// to bottom), then rotated `deg` degrees clockwise (page space, y down), as DrawingML's `a:xfrm`
/// `flipH`, `flipV` and `rot` say (ECMA-376 §20.1.7.6).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Spin {
    pub deg: f32,
    pub flip_h: bool,
    pub flip_v: bool,
}

impl Spin {
    /// A spin with its angle normalised to `0..360`.
    pub fn new(deg: f32, flip_h: bool, flip_v: bool) -> Spin {
        Spin { deg: normalize_degrees(deg), flip_h, flip_v }
    }

    /// No rotation and no mirroring.
    pub fn is_identity(&self) -> bool {
        normalize_degrees(self.deg) == 0.0 && !self.flip_h && !self.flip_v
    }

    /// The linear part (a, b, c, d: x' = a·x + c·y, y' = b·x + d·y).
    fn linear(&self) -> [f32; 4] {
        let r = normalize_degrees(self.deg).to_radians();
        let (s, c) = r.sin_cos();
        let fx = if self.flip_h { -1.0 } else { 1.0 };
        let fy = if self.flip_v { -1.0 } else { 1.0 };
        [c * fx, s * fx, -s * fy, c * fy]
    }

    /// The affine map (a, b, c, d, e, f: x' = a·x + c·y + e, y' = b·x + d·y + f) turning the page
    /// about point (`cx`, `cy`).
    pub fn matrix(&self, cx: f32, cy: f32) -> [f32; 6] {
        let [a, b, c, d] = self.linear();
        let (cx, cy) = (finite(cx), finite(cy));
        [a, b, c, d, cx - (a * cx + c * cy), cy - (b * cx + d * cy)]
    }

    /// Where point (`x`, `y`) goes when turned about (`cx`, `cy`).
    pub fn apply(&self, cx: f32, cy: f32, x: f32, y: f32) -> (f32, f32) {
        let [a, b, c, d] = self.linear();
        let (u, v) = (x - cx, y - cy);
        (cx + a * u + c * v, cy + b * u + d * v)
    }

    /// The point that [`Spin::apply`] takes to (`x`, `y`).
    pub fn unapply(&self, cx: f32, cy: f32, x: f32, y: f32) -> (f32, f32) {
        // The linear part is orthogonal: its inverse is its transpose.
        let [a, b, c, d] = self.linear();
        let (u, v) = (x - cx, y - cy);
        (cx + a * u + b * v, cy + c * u + d * v)
    }

    /// The axis-aligned box `r` covers once turned about its centre.
    pub fn bounds(&self, r: Rect) -> Rect {
        let (bw, bh) = self.extent(r.w, r.h);
        Rect::new(r.x + (r.w - bw) / 2.0, r.y + (r.h - bh) / 2.0, bw, bh)
    }

    /// The width and height of the box a `w` × `h` rectangle covers once turned.
    pub fn extent(&self, w: f32, h: f32) -> (f32, f32) {
        let (w, h) = (finite(w).abs(), finite(h).abs());
        let r = normalize_degrees(self.deg).to_radians();
        let (s, c) = (r.sin().abs(), r.cos().abs());
        (w * c + h * s, w * s + h * c)
    }

    /// This spin applied inside `outer` (a group member's own spin within its group's): the one
    /// spin doing both.
    pub fn within(self, outer: Spin) -> Spin {
        // outer = R(θo)·Fo, self = R(θs)·Fs; Fo·R(θ) = R(±θ)·Fo (one mirror reverses the turn).
        let one_mirror = outer.flip_h != outer.flip_v;
        let inner = if one_mirror { -self.deg } else { self.deg };
        Spin::new(outer.deg + inner, outer.flip_h != self.flip_h, outer.flip_v != self.flip_v)
    }
}

/// Standard paper sizes (Word's Layout › Size gallery), in points (portrait).
pub const PAPER_SIZES: &[(&str, f32, f32)] = &[
    ("Letter", 612.0, 792.0),
    ("Legal", 612.0, 1008.0),
    ("Executive", 522.0, 756.0),
    ("A3", 841.89, 1190.55),
    ("A4", 595.28, 841.89),
    ("A5", 419.53, 595.28),
    ("B4 (JIS)", 728.5, 1031.8),
    ("B5 (JIS)", 515.9, 728.5),
    ("Tabloid", 792.0, 1224.0),
    ("Statement", 396.0, 612.0),
    ("Envelope #10", 297.0, 684.0),
    ("Envelope DL", 311.8, 623.6),
];

/// The paper size whose dimensions match (either orientation, within half a point).
pub fn paper_name(w: f32, h: f32) -> Option<&'static str> {
    PAPER_SIZES
        .iter()
        .find(|(_, pw, ph)| ((pw - w).abs() < 0.5 && (ph - h).abs() < 0.5) || ((pw - h).abs() < 0.5 && (ph - w).abs() < 0.5))
        .map(|(n, _, _)| *n)
}

/// Get a paper size's dimensions in any unit (portrait). Returns `None` if the name is unknown.
///
/// ```
/// # use wordcraft_geom::{paper_size_in, Unit};
/// let (w, h) = paper_size_in("A4", Unit::Millimeters).unwrap();
/// assert!((w - 210.0).abs() < 0.5);
/// assert!((h - 297.0).abs() < 0.5);
/// ```
pub fn paper_size_in(name: &str, unit: Unit) -> Option<(f32, f32)> {
    PAPER_SIZES.iter().find(|(n, _, _)| n.eq_ignore_ascii_case(name)).map(|(_, w, h)| (unit.from_pt(*w), unit.from_pt(*h)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_round_trips() {
        assert_eq!(twips(1440), 72.0);
        assert_eq!(to_twips(72.0), 1440);
        assert_eq!(emu(914_400), 72.0);
        assert_eq!(to_emu(72.0), 914_400);
        assert_eq!(half_points(24), 12.0);
        assert_eq!(to_half_points(10.5), 21);
        assert_eq!(to_twips(f32::NAN), 0);
    }

    #[test]
    fn inch_cm_conversions() {
        // 1 inch = 2.54 cm
        assert!((in_to_cm(1.0) - 2.54).abs() < 0.001);
        assert!((cm_to_in(2.54) - 1.0).abs() < 0.001);
        // round-trip
        assert!((cm_to_in(in_to_cm(3.5)) - 3.5).abs() < 0.001);
        // NaN/inf → 0
        assert_eq!(in_to_cm(f32::NAN), 0.0);
        assert_eq!(cm_to_in(f32::INFINITY), 0.0);
    }

    #[test]
    fn unit_convert() {
        // inches → cm
        let cm = Unit::Inches.convert(1.0, Unit::Centimeters);
        assert!((cm - 2.54).abs() < 0.01);
        // cm → inches
        let inches = Unit::Centimeters.convert(2.54, Unit::Inches);
        assert!((inches - 1.0).abs() < 0.01);
        // same unit is identity
        assert_eq!(Unit::Points.convert(42.0, Unit::Points), 42.0);
        // inches → points
        assert!((Unit::Inches.convert(1.0, Unit::Points) - 72.0).abs() < 0.01);
        // mm → cm
        assert!((Unit::Millimeters.convert(10.0, Unit::Centimeters) - 1.0).abs() < 0.01);
        // NaN stays safe
        assert_eq!(Unit::Inches.convert(f32::NAN, Unit::Centimeters), 0.0);
    }

    #[test]
    fn unit_all_and_labels() {
        let all = Unit::all();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0], Unit::Inches);
        assert_eq!(Unit::Centimeters.label(), "Centimeters");
        assert_eq!(Unit::Points.label(), "Points");
    }

    #[test]
    fn unit_parse_names() {
        assert_eq!(Unit::parse("inches"), Some(Unit::Inches));
        assert_eq!(Unit::parse("CM"), Some(Unit::Centimeters));
        assert_eq!(Unit::parse("pt"), Some(Unit::Points));
        assert_eq!(Unit::parse("pica"), Some(Unit::Picas));
        assert_eq!(Unit::parse("mm"), Some(Unit::Millimeters));
        assert_eq!(Unit::parse("furlongs"), None);
        assert_eq!(Unit::parse(""), None);
    }

    #[test]
    fn unit_from_pt_to_pt() {
        // 72 pt = 1 inch
        assert!((Unit::Inches.from_pt(72.0) - 1.0).abs() < 0.001);
        assert!((Unit::Inches.to_pt(1.0) - 72.0).abs() < 0.001);
        // round-trip
        assert!((Unit::Centimeters.to_pt(Unit::Centimeters.from_pt(100.0)) - 100.0).abs() < 0.01);
        // NaN safe
        assert_eq!(Unit::Inches.from_pt(f32::NAN), 0.0);
    }

    #[test]
    fn parses_measurements() {
        assert_eq!(parse_measure("1\"", Unit::Points), Some(72.0));
        assert_eq!(parse_measure("1 in", Unit::Points), Some(72.0));
        assert_eq!(parse_measure("12pt", Unit::Inches), Some(12.0));
        assert_eq!(parse_measure("2", Unit::Inches), Some(144.0));
        assert_eq!(parse_measure("1,5 li", Unit::Inches), Some(18.0));
        let cm = parse_measure("2.54 cm", Unit::Inches).unwrap();
        assert!((cm - 72.0).abs() < 0.01);
        assert_eq!(parse_measure("abc", Unit::Inches), None);
        assert_eq!(parse_measure("", Unit::Inches), None);
        assert_eq!(parse_measure("1 furlong", Unit::Inches), None);
        // Both decimal separators, whatever the region (#838).
        let comma = parse_measure("2,54 cm", Unit::Inches).unwrap();
        assert!((comma - 72.0).abs() < 0.01);
        assert_eq!(parse_measure("1,5", Unit::Inches), Some(108.0));
        assert_eq!(parse_measure("1.5\"", Unit::Centimeters), Some(108.0));
        assert_eq!(parse_measure("\u{2212}1 in", Unit::Points), Some(-72.0));
        assert_eq!(parse_measure("1,2,3", Unit::Inches), None);
    }

    #[test]
    fn formats_measurements() {
        assert_eq!(Unit::Inches.format(72.0), "1\"");
        assert_eq!(Unit::Inches.format(90.0), "1.25\"");
        assert_eq!(Unit::Points.format(12.0), "12 pt");
        assert_eq!(Unit::Centimeters.format(72.0), "2.54 cm");
        assert_eq!(Unit::Centimeters.format_with(72.0, ','), "2,54 cm");
        assert_eq!(Unit::Inches.format_with(108.0, ','), "1,5\"");
        assert_eq!(Unit::Points.format_with(720.0, ','), "720 pt");
        assert_eq!(Unit::Inches.format_with(f32::NAN, ','), "0\"");
    }

    #[test]
    fn snap_to_grid_works() {
        // Snap to 0.25 inch grid (18 pt)
        assert!((snap_to_grid(20.0, 18.0) - 18.0).abs() < 0.01);
        assert!((snap_to_grid(27.0, 18.0) - 36.0).abs() < 0.01);
        // Exact values are unchanged
        assert_eq!(snap_to_grid(36.0, 18.0), 36.0);
        // Zero/negative grid returns the value unchanged
        assert_eq!(snap_to_grid(20.0, 0.0), 20.0);
        assert_eq!(snap_to_grid(20.0, -1.0), 20.0);
        // NaN grid
        assert_eq!(snap_to_grid(20.0, f32::NAN), 20.0);
        assert_eq!(snap_to_grid(f32::NAN, 18.0), 0.0);
    }

    #[test]
    fn clamp_margin_works() {
        assert_eq!(clamp_margin(36.0, 72.0), 36.0);
        assert_eq!(clamp_margin(-5.0, 72.0), 0.0);
        assert_eq!(clamp_margin(100.0, 72.0), 72.0);
        assert_eq!(clamp_margin(f32::NAN, 72.0), 0.0);
        assert_eq!(clamp_margin(50.0, f32::NAN), 0.0);
        assert_eq!(clamp_margin(50.0, -10.0), 0.0);
    }

    #[test]
    fn rect_ops() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        assert!(a.intersects(&b));
        assert_eq!(a.union(&b), Rect::new(0.0, 0.0, 15.0, 15.0));
        assert!(a.contains(Point::new(5.0, 5.0)));
        assert_eq!(Rect::new(0.0, 0.0, -5.0, f32::NAN).w, 0.0);
        assert_eq!(a.inset(1.0, 2.0, 3.0, 4.0), Rect::new(1.0, 2.0, 6.0, 4.0));
        // center
        let c = a.center();
        assert_eq!(c.x, 5.0);
        assert_eq!(c.y, 5.0);
        // intersection
        let i = a.intersection(&b).unwrap();
        assert_eq!(i, Rect::new(5.0, 5.0, 5.0, 5.0));
        let no_overlap = Rect::new(20.0, 20.0, 5.0, 5.0);
        assert!(a.intersection(&no_overlap).is_none());
    }

    #[test]
    fn paper_names() {
        assert_eq!(paper_name(612.0, 792.0), Some("Letter"));
        assert_eq!(paper_name(792.0, 612.0), Some("Letter"));
        assert_eq!(paper_name(100.0, 100.0), None);
    }

    #[test]
    fn paper_size_in_units() {
        // A4 in mm should be ~210 × 297
        let (w, h) = paper_size_in("A4", Unit::Millimeters).unwrap();
        assert!((w - 210.0).abs() < 0.5);
        assert!((h - 297.0).abs() < 0.5);
        // Letter in inches should be 8.5 × 11
        let (w, h) = paper_size_in("Letter", Unit::Inches).unwrap();
        assert!((w - 8.5).abs() < 0.01);
        assert!((h - 11.0).abs() < 0.01);
        // case-insensitive
        assert!(paper_size_in("a4", Unit::Points).is_some());
        assert!(paper_size_in("LETTER", Unit::Centimeters).is_some());
        // unknown
        assert!(paper_size_in("Tabloid XL", Unit::Inches).is_none());
    }
}
