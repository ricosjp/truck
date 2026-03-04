//! Font outline ingestion and conversion to `monstertruck` curves and wires.
//!
//! Requires the `font` feature flag. Parses TrueType/OpenType glyph outlines
//! via [`ttf_parser`] and maps line, quadratic Bezier, and cubic Bezier
//! segments into [`Curve`](crate::Curve) edges assembled into closed
//! [`Wire`](crate::Wire)s suitable for planar face construction.

use crate::{Curve, Result, builder, errors::Error};
use monstertruck_core::cgmath64::*;

type Edge = monstertruck_topology::Edge<Point3, Curve>;
type Wire = monstertruck_topology::Wire<Point3, Curve>;

/// Options for glyph/text profile extraction.
#[derive(Debug, Clone)]
pub struct TextOptions {
    /// Uniform scale applied to font units. Default: `1.0 / units_per_em`.
    pub scale: Option<f64>,
    /// Whether to flip the Y axis (font coordinates have Y-up, but many CAD
    /// systems use Y-down or want the baseline at `y = 0` growing upward).
    /// Default: `true` (flip so that glyphs grow in +Y).
    pub y_flip: bool,
    /// Z coordinate for the planar profile. Default: `0.0`.
    pub z: f64,
    /// Tolerance for closing contours: if the distance between the last point
    /// and the contour start is below this, they are snapped together.
    /// Default: `1e-7`.
    pub closure_tolerance: f64,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            scale: None,
            y_flip: true,
            z: 0.0,
            closure_tolerance: 1e-7,
        }
    }
}

/// A single outline segment extracted from a font glyph.
#[derive(Debug, Clone)]
enum Segment {
    /// Straight line to `(x, y)`.
    Line(f64, f64),
    /// Quadratic Bezier with control point `(cx, cy)` to endpoint `(x, y)`.
    Quad(f64, f64, f64, f64),
    /// Cubic Bezier with control points `(c1x, c1y)`, `(c2x, c2y)` to `(x, y)`.
    Cubic(f64, f64, f64, f64, f64, f64),
}

/// Collects glyph outline contours from [`ttf_parser::OutlineBuilder`] callbacks.
struct ContourCollector {
    contours: Vec<(f64, f64, Vec<Segment>)>,
    current_start: Option<(f64, f64)>,
    current_segments: Vec<Segment>,
}

impl ContourCollector {
    fn new() -> Self {
        Self {
            contours: Vec::new(),
            current_start: None,
            current_segments: Vec::new(),
        }
    }
}

impl ttf_parser::OutlineBuilder for ContourCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        // Start a new contour.
        if let Some((sx, sy)) = self.current_start.take() {
            if !self.current_segments.is_empty() {
                let segs = std::mem::take(&mut self.current_segments);
                self.contours.push((sx, sy, segs));
            }
        }
        self.current_start = Some((x as f64, y as f64));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.current_segments
            .push(Segment::Line(x as f64, y as f64));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.current_segments
            .push(Segment::Quad(x1 as f64, y1 as f64, x as f64, y as f64));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.current_segments.push(Segment::Cubic(
            x1 as f64, y1 as f64, x2 as f64, y2 as f64, x as f64, y as f64,
        ));
    }

    fn close(&mut self) {
        if let Some((sx, sy)) = self.current_start.take() {
            let segs = std::mem::take(&mut self.current_segments);
            if !segs.is_empty() {
                self.contours.push((sx, sy, segs));
            }
        }
    }
}

/// Applies scale, optional Y-flip, and Z coordinate to a 2D font point.
fn transform_point(x: f64, y: f64, scale: f64, y_flip: bool, z: f64) -> Point3 {
    let y = if y_flip { -y } else { y };
    Point3::new(x * scale, y * scale, z)
}

/// Converts a collected contour into a closed [`Wire`].
///
/// Each segment becomes one [`Edge`]; the wire is closed by snapping the
/// last endpoint to the first vertex when within `closure_tolerance`.
fn contour_to_wire(
    start_x: f64,
    start_y: f64,
    segments: &[Segment],
    scale: f64,
    y_flip: bool,
    z: f64,
    closure_tolerance: f64,
) -> Result<Wire> {
    if segments.is_empty() {
        return Err(Error::FromTopology(
            monstertruck_topology::errors::Error::EmptyWire,
        ));
    }

    let first_pt = transform_point(start_x, start_y, scale, y_flip, z);
    let first_vertex = builder::vertex(first_pt);

    // Build intermediate vertices and edges.
    let mut edges: Vec<Edge> = Vec::with_capacity(segments.len());
    let mut prev_vertex = first_vertex.clone();

    for (i, seg) in segments.iter().enumerate() {
        let is_last = i + 1 == segments.len();

        // Extract the endpoint and optional control points for this segment.
        let (inter_points, raw_endpoint) = match *seg {
            Segment::Line(x, y) => (vec![], (x, y)),
            Segment::Quad(cx, cy, x, y) => {
                let cp = transform_point(cx, cy, scale, y_flip, z);
                (vec![cp], (x, y))
            }
            Segment::Cubic(c1x, c1y, c2x, c2y, x, y) => {
                let cp1 = transform_point(c1x, c1y, scale, y_flip, z);
                let cp2 = transform_point(c2x, c2y, scale, y_flip, z);
                (vec![cp1, cp2], (x, y))
            }
        };

        let pt = transform_point(raw_endpoint.0, raw_endpoint.1, scale, y_flip, z);
        let next = if is_last && first_pt.distance(pt) < closure_tolerance {
            first_vertex.clone()
        } else {
            builder::vertex(pt)
        };

        let edge: Edge = if inter_points.is_empty() {
            builder::line(&prev_vertex, &next)
        } else {
            builder::bezier(&prev_vertex, &next, inter_points)
        };
        let endpoint = next;

        edges.push(edge);
        prev_vertex = endpoint;
    }

    // If the last vertex is not the first vertex, close with a line segment.
    if prev_vertex.id() != first_vertex.id() {
        let close_edge: Edge = builder::line(&prev_vertex, &first_vertex);
        edges.push(close_edge);
    }

    Ok(edges.into())
}

/// Skips degenerate contours where all segments collapse to a single point.
fn is_degenerate_contour(start_x: f64, start_y: f64, segments: &[Segment]) -> bool {
    segments.iter().all(|seg| match *seg {
        Segment::Line(x, y) => (x - start_x).abs() < 1e-12 && (y - start_y).abs() < 1e-12,
        Segment::Quad(_, _, x, y) => (x - start_x).abs() < 1e-12 && (y - start_y).abs() < 1e-12,
        Segment::Cubic(_, _, _, _, x, y) => {
            (x - start_x).abs() < 1e-12 && (y - start_y).abs() < 1e-12
        }
    })
}

/// Extracts the outline of a single glyph as a set of closed [`Wire`]s.
///
/// Each contour in the glyph becomes one wire. For glyphs with holes (e.g.
/// `O`, `B`, `8`), multiple wires are returned: the outer contour and
/// inner hole contour(s). These wires can then be passed to
/// [`profile::attach_plane_normalized`](crate::profile::attach_plane_normalized)
/// for planar face construction.
///
/// # Arguments
///
/// * `face` - A parsed font face from [`ttf_parser`].
/// * `glyph_id` - The glyph identifier to outline.
/// * `opts` - Conversion options (scale, y-flip, z-plane, tolerance).
///
/// # Errors
///
/// Returns [`Error::FromTopology`] if the glyph has no outline data or
/// produces degenerate contours.
pub fn glyph_profile(
    face: &ttf_parser::Face<'_>,
    glyph_id: ttf_parser::GlyphId,
    opts: &TextOptions,
) -> Result<Vec<Wire>> {
    let scale = opts
        .scale
        .unwrap_or_else(|| 1.0 / face.units_per_em() as f64);

    let mut collector = ContourCollector::new();
    face.outline_glyph(glyph_id, &mut collector)
        .ok_or(Error::FromTopology(
            monstertruck_topology::errors::Error::EmptyWire,
        ))?;

    collector
        .contours
        .iter()
        .filter(|(sx, sy, segs)| !is_degenerate_contour(*sx, *sy, segs))
        .map(|(sx, sy, segs)| {
            contour_to_wire(
                *sx,
                *sy,
                segs,
                scale,
                opts.y_flip,
                opts.z,
                opts.closure_tolerance,
            )
        })
        .collect()
}

/// A contour with its pre-computed X offset for parallel conversion.
struct OffsetContour {
    start_x: f64,
    start_y: f64,
    segments: Vec<Segment>,
    offset_x: f64,
}

/// Extracts glyph outlines for an entire text string as a flat set of
/// closed [`Wire`]s, positioned along the baseline with correct glyph advance.
///
/// Each character's contours are offset along the X axis by the accumulated
/// horizontal advance. The resulting wires can be passed directly to
/// [`profile::attach_plane_normalized`](crate::profile::attach_plane_normalized)
/// or [`profile::solid_from_planar_profile`](crate::profile::solid_from_planar_profile).
///
/// On non-WASM targets with the `font` feature, contour-to-wire conversion
/// is parallelized via [`rayon`].
///
/// Characters with no outline (e.g. space) are silently skipped.
///
/// # Errors
///
/// Returns an error if any outlined glyph produces an invalid wire.
pub fn text_profile(
    face: &ttf_parser::Face<'_>,
    text: &str,
    opts: &TextOptions,
) -> Result<Vec<Wire>> {
    let scale = opts
        .scale
        .unwrap_or_else(|| 1.0 / face.units_per_em() as f64);

    // Pass 1: collect all contours with their cursor offsets (sequential).
    let mut contours = Vec::new();
    let mut cursor_x = 0.0_f64;

    for ch in text.chars() {
        let Some(glyph_id) = face.glyph_index(ch) else {
            continue;
        };

        let mut collector = ContourCollector::new();
        if face.outline_glyph(glyph_id, &mut collector).is_none() {
            if let Some(advance) = face.glyph_hor_advance(glyph_id) {
                cursor_x += advance as f64 * scale;
            }
            continue;
        }

        for (sx, sy, segs) in collector.contours {
            if !is_degenerate_contour(sx, sy, &segs) {
                contours.push(OffsetContour {
                    start_x: sx,
                    start_y: sy,
                    segments: segs,
                    offset_x: cursor_x,
                });
            }
        }

        if let Some(advance) = face.glyph_hor_advance(glyph_id) {
            cursor_x += advance as f64 * scale;
        }
    }

    // Pass 2: convert contours to wires (parallel on non-WASM).
    let y_flip = opts.y_flip;
    let z = opts.z;
    let closure_tolerance = opts.closure_tolerance;

    #[cfg(not(target_arch = "wasm32"))]
    {
        use rayon::prelude::*;
        contours
            .par_iter()
            .map(|c| {
                contour_to_wire(
                    c.start_x + c.offset_x / scale,
                    c.start_y,
                    &c.segments,
                    scale,
                    y_flip,
                    z,
                    closure_tolerance,
                )
            })
            .collect()
    }

    #[cfg(target_arch = "wasm32")]
    {
        contours
            .iter()
            .map(|c| {
                contour_to_wire(
                    c.start_x + c.offset_x / scale,
                    c.start_y,
                    &c.segments,
                    scale,
                    y_flip,
                    z,
                    closure_tolerance,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal synthetic font test: builds contours manually via the collector.
    #[test]
    fn contour_collector_line_segments() {
        use ttf_parser::OutlineBuilder;
        let mut c = ContourCollector::new();
        // Square contour.
        c.move_to(0.0, 0.0);
        c.line_to(100.0, 0.0);
        c.line_to(100.0, 100.0);
        c.line_to(0.0, 100.0);
        c.close();
        assert_eq!(c.contours.len(), 1);
        // 3 explicit line_to segments; the close back to start is handled
        // implicitly by `contour_to_wire` when assembling the wire.
        assert_eq!(c.contours[0].2.len(), 3);
    }

    #[test]
    fn contour_to_wire_square() {
        use ttf_parser::OutlineBuilder;
        let mut c = ContourCollector::new();
        c.move_to(0.0, 0.0);
        c.line_to(1.0, 0.0);
        c.line_to(1.0, 1.0);
        c.line_to(0.0, 1.0);
        c.close();

        let (sx, sy, segs) = &c.contours[0];
        let wire = contour_to_wire(*sx, *sy, segs, 1.0, false, 0.0, 1e-7).unwrap();
        assert!(wire.is_closed());
        assert_eq!(wire.len(), 4);
    }

    #[test]
    fn contour_to_wire_with_bezier() {
        use ttf_parser::OutlineBuilder;
        let mut c = ContourCollector::new();
        c.move_to(0.0, 0.0);
        c.quad_to(0.5, 1.0, 1.0, 0.0);
        c.line_to(0.0, 0.0);
        c.close();

        let (sx, sy, segs) = &c.contours[0];
        let wire = contour_to_wire(*sx, *sy, segs, 1.0, false, 0.0, 1e-7).unwrap();
        assert!(wire.is_closed());
        assert_eq!(wire.len(), 2);
    }

    #[test]
    fn contour_to_wire_cubic() {
        use ttf_parser::OutlineBuilder;
        let mut c = ContourCollector::new();
        c.move_to(0.0, 0.0);
        c.curve_to(0.3, 1.0, 0.7, 1.0, 1.0, 0.0);
        c.line_to(0.0, 0.0);
        c.close();

        let (sx, sy, segs) = &c.contours[0];
        let wire = contour_to_wire(*sx, *sy, segs, 1.0, false, 0.0, 1e-7).unwrap();
        assert!(wire.is_closed());
        assert_eq!(wire.len(), 2);
    }

    #[test]
    fn contour_y_flip() {
        let pt = transform_point(10.0, 20.0, 0.01, true, 5.0);
        assert!((pt.x - 0.1).abs() < 1e-10);
        assert!((pt.y - (-0.2)).abs() < 1e-10);
        assert!((pt.z - 5.0).abs() < 1e-10);
    }

    #[test]
    fn contour_no_y_flip() {
        let pt = transform_point(10.0, 20.0, 0.01, false, 0.0);
        assert!((pt.x - 0.1).abs() < 1e-10);
        assert!((pt.y - 0.2).abs() < 1e-10);
    }

    #[test]
    fn multiple_contours() {
        use ttf_parser::OutlineBuilder;
        let mut c = ContourCollector::new();
        // Outer square.
        c.move_to(0.0, 0.0);
        c.line_to(10.0, 0.0);
        c.line_to(10.0, 10.0);
        c.line_to(0.0, 10.0);
        c.close();
        // Inner square (hole).
        c.move_to(2.0, 2.0);
        c.line_to(8.0, 2.0);
        c.line_to(8.0, 8.0);
        c.line_to(2.0, 8.0);
        c.close();

        assert_eq!(c.contours.len(), 2);

        let wires: Vec<Wire> = c
            .contours
            .iter()
            .map(|(sx, sy, segs)| contour_to_wire(*sx, *sy, segs, 0.1, false, 0.0, 1e-7).unwrap())
            .collect();

        assert_eq!(wires.len(), 2);
        assert!(wires.iter().all(|w| w.is_closed()));
    }

    #[test]
    fn degenerate_contour_filtered() {
        assert!(is_degenerate_contour(
            5.0,
            5.0,
            &[Segment::Line(5.0, 5.0), Segment::Line(5.0, 5.0)]
        ));
        assert!(!is_degenerate_contour(0.0, 0.0, &[Segment::Line(1.0, 0.0)]));
    }
}
