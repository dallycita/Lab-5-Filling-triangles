use raylib::prelude::*;

use crate::fragment::Fragment;
use crate::vertex::Vertex;

/// Línea entre dos vértices con el algoritmo de Bresenham.
/// Usa la posición transformada de cada vértice y genera fragmentos blancos.
pub fn line(a: &Vertex, b: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let color = Vector3::new(1.0, 1.0, 1.0);

    let mut x0 = a.transformed_position.x.round() as i32;
    let mut y0 = a.transformed_position.y.round() as i32;
    let x1 = b.transformed_position.x.round() as i32;
    let y1 = b.transformed_position.y.round() as i32;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        fragments.push(Fragment::new(x0 as f32, y0 as f32, color, 0.0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    fragments
}
