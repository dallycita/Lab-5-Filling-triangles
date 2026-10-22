use raylib::prelude::*;

use crate::fragment::Fragment;
use crate::line::line;
use crate::vertex::Vertex;

#[derive(Clone, Copy)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
    FilledScanline,
}

/// Color uniforme para los rellenos. Más adelante se podrán interpolar
/// los atributos de los vértices con las coordenadas baricéntricas.
const FILL_COLOR: Vector3 = Vector3::new(1.0, 1.0, 1.0);

// ---------------------------------------------------------------------------
// Contorno: el triangle de antes, con tres llamadas a line
// ---------------------------------------------------------------------------

pub fn triangle(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    fragments.extend(line(a, b));
    fragments.extend(line(b, c));
    fragments.extend(line(c, a));
    fragments
}

// ---------------------------------------------------------------------------
// Regla común para los dos rellenos
// ---------------------------------------------------------------------------
//
// Un píxel (x, y) genera un fragmento si su centro (x + 0.5, y + 0.5) está
// dentro del triángulo o exactamente sobre uno de sus bordes. Los dos
// algoritmos usan esta misma regla para generar las mismas posiciones.
// Los cálculos se hacen en f64: con f32, un centro que cae casi justo sobre
// una arista puede quedar dentro en un algoritmo y fuera en el otro.

/// Dos veces el área (con signo) del triángulo a, b, p.
/// Es positiva si p está a un lado de la recta ab y negativa si está al otro.
fn edge(ax: f64, ay: f64, bx: f64, by: f64, px: f64, py: f64) -> f64 {
    (bx - ax) * (py - ay) - (by - ay) * (px - ax)
}

/// Primer píxel cuyo centro es mayor o igual que `value`.
fn first_pixel(value: f64) -> i32 {
    (value - 0.5).ceil() as i32
}

/// Último píxel cuyo centro es menor o igual que `value`.
fn last_pixel(value: f64) -> i32 {
    (value - 0.5).floor() as i32
}

/// Posición transformada (x, y) de un vértice, en f64.
fn screen_position(v: &Vertex) -> (f64, f64) {
    (v.transformed_position.x as f64, v.transformed_position.y as f64)
}

// ---------------------------------------------------------------------------
// Algoritmo 1: coordenadas baricéntricas
// ---------------------------------------------------------------------------

pub fn triangle_barycentric(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    let (ax, ay) = screen_position(a);
    let (bx, by) = screen_position(b);
    let (cx, cy) = screen_position(c);

    // Área del triángulo (por 2). Si es 0, los tres vértices están en una línea.
    let area = edge(ax, ay, bx, by, cx, cy);
    if area == 0.0 {
        return fragments;
    }

    // Rectángulo más pequeño que contiene al triángulo.
    let min_x = first_pixel(ax.min(bx).min(cx));
    let max_x = last_pixel(ax.max(bx).max(cx));
    let min_y = first_pixel(ay.min(by).min(cy));
    let max_y = last_pixel(ay.max(by).max(cy));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f64 + 0.5;
            let py = y as f64 + 0.5;

            // Coordenadas baricéntricas: cuánto "pesa" cada vértice en el punto.
            // Al dividir entre el área, el signo se corrige solo,
            // así que no importa el orden de los vértices.
            let w0 = edge(bx, by, cx, cy, px, py) / area;
            let w1 = edge(cx, cy, ax, ay, px, py) / area;
            let w2 = edge(ax, ay, bx, by, px, py) / area;

            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                fragments.push(Fragment::new(x as f32, y as f32, FILL_COLOR, 0.0));
            }
        }
    }

    fragments
}

// ---------------------------------------------------------------------------
// Algoritmo 2: líneas horizontales (scanline)
// ---------------------------------------------------------------------------

pub fn triangle_scanline(a: &Vertex, b: &Vertex, c: &Vertex) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    let points = [screen_position(a), screen_position(b), screen_position(c)];
    let (ax, ay) = points[0];
    let (bx, by) = points[1];
    let (cx, cy) = points[2];

    // Mismo caso degenerado que en el baricéntrico.
    if edge(ax, ay, bx, by, cx, cy) == 0.0 {
        return fragments;
    }

    let edges = [(0, 1), (1, 2), (2, 0)];

    let min_y = first_pixel(ay.min(by).min(cy));
    let max_y = last_pixel(ay.max(by).max(cy));

    for y in min_y..=max_y {
        let row = y as f64 + 0.5;

        // Dónde cruza cada arista la línea horizontal de esta fila.
        // Nos quedamos con el cruce más a la izquierda y el más a la derecha.
        let mut left = f64::MAX;
        let mut right = f64::MIN;

        for (i, j) in edges {
            let (x0, y0) = points[i];
            let (x1, y1) = points[j];

            // La arista no toca esta fila.
            if row < y0.min(y1) || row > y0.max(y1) {
                continue;
            }

            if y0 == y1 {
                // Arista horizontal justo sobre la fila: todo su tramo cuenta.
                left = left.min(x0.min(x1));
                right = right.max(x0.max(x1));
            } else {
                let x = x0 + (row - y0) * (x1 - x0) / (y1 - y0);
                left = left.min(x);
                right = right.max(x);
            }
        }

        // Ninguna arista cruzó la fila.
        if left > right {
            continue;
        }

        // Cada fila se resuelve con un solo tramo [left, right], así que un
        // vértice que cae justo en la fila (tocado por dos aristas) no
        // genera fragmentos duplicados.
        for x in first_pixel(left)..=last_pixel(right) {
            fragments.push(Fragment::new(x as f32, y as f32, FILL_COLOR, 0.0));
        }
    }

    fragments
}
