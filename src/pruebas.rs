// Comparación de los dos rellenos y medición de tiempos.
// Se activa con --compare y --benchmark-triangles (no abren la ventana).

use std::hint::black_box;
use std::time::{Duration, Instant};

use raylib::prelude::*;

use crate::fragment::Fragment;
use crate::rasterizer::rasterize;
use crate::triangle::{TriangleMode, triangle_barycentric, triangle_scanline};
use crate::vertex::Vertex;

pub const MODES: [TriangleMode; 3] = [
    TriangleMode::Wireframe,
    TriangleMode::FilledBarycentric,
    TriangleMode::FilledScanline,
];

pub fn mode_name(mode: TriangleMode) -> &'static str {
    match mode {
        TriangleMode::Wireframe => "wireframe",
        TriangleMode::FilledBarycentric => "baricentrico",
        TriangleMode::FilledScanline => "scanline",
    }
}

// ---------------------------------------------------------------------------
// Triángulos de prueba (ya en coordenadas de pantalla)
// ---------------------------------------------------------------------------

/// Vértice en coordenadas de pantalla: copiamos la posición a
/// transformed_position, igual que en la prueba del primer triángulo.
fn screen_vertex(x: f32, y: f32) -> Vertex {
    let position = Vector3::new(x, y, 0.0);
    let mut vertex = Vertex::new_with_color(position, Vector3::new(1.0, 1.0, 1.0));
    vertex.transformed_position = position;
    vertex
}

fn tri(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> [Vertex; 3] {
    [screen_vertex(a.0, a.1), screen_vertex(b.0, b.1), screen_vertex(c.0, c.1)]
}

pub fn test_cases() -> Vec<(&'static str, [Vertex; 3])> {
    vec![
        ("Pequeño", tri((100.0, 100.0), (108.0, 104.0), (103.0, 110.0))),
        ("Grande", tri((150.0, 80.0), (650.0, 150.0), (300.0, 550.0))),
        ("Borde horizontal", tri((450.0, 400.0), (700.0, 400.0), (600.0, 550.0))),
        ("Borde vertical", tri((50.0, 300.0), (50.0, 550.0), (250.0, 450.0))),
        ("Grande en orden inverso", tri((300.0, 550.0), (650.0, 150.0), (150.0, 80.0))),
        ("Parcialmente fuera", tri((650.0, 450.0), (950.0, 500.0), (700.0, 750.0))),
        ("Tres vértices en línea", tri((100.0, 50.0), (200.0, 100.0), (300.0, 150.0))),
    ]
}

/// Los vértices de las pruebas como una sola secuencia (de tres en tres).
pub fn test_vertex_array() -> Vec<Vertex> {
    let mut vertex_array = Vec::new();
    for (_, t) in test_cases() {
        vertex_array.extend(t);
    }
    vertex_array
}

// ---------------------------------------------------------------------------
// --compare
// ---------------------------------------------------------------------------

/// Posiciones (x, y) de los fragmentos, ordenadas para poder compararlas.
fn sorted_positions(fragments: &[Fragment]) -> Vec<(i32, i32)> {
    let mut positions = Vec::new();
    for f in fragments {
        positions.push((f.position.x as i32, f.position.y as i32));
    }
    positions.sort();
    positions
}

/// Cuántas posiciones están en una lista y no en la otra.
fn count_differences(a: &[(i32, i32)], b: &[(i32, i32)]) -> usize {
    let mut differences = 0;
    for p in a {
        if b.binary_search(p).is_err() {
            differences += 1;
        }
    }
    for p in b {
        if a.binary_search(p).is_err() {
            differences += 1;
        }
    }
    differences
}

fn count_duplicates(sorted: &[(i32, i32)]) -> usize {
    let mut duplicates = 0;
    for i in 1..sorted.len() {
        if sorted[i] == sorted[i - 1] {
            duplicates += 1;
        }
    }
    duplicates
}

fn count_inside(positions: &[(i32, i32)], width: i32, height: i32) -> usize {
    let mut inside = 0;
    for &(x, y) in positions {
        if x >= 0 && x < width && y >= 0 && y < height {
            inside += 1;
        }
    }
    inside
}

pub fn compare(model_triangles: &[[Vertex; 3]], width: i32, height: i32) {
    println!("| Caso | Baricéntrico | Scanline | Diferencias | Duplicados (B / S) | Dentro de la ventana |");
    println!("|---|---|---|---|---|---|");

    for (name, t) in test_cases() {
        let b = sorted_positions(&triangle_barycentric(&t[0], &t[1], &t[2]));
        let s = sorted_positions(&triangle_scanline(&t[0], &t[1], &t[2]));
        println!(
            "| {name} | {} | {} | {} | {} / {} | {} |",
            b.len(),
            s.len(),
            count_differences(&b, &s),
            count_duplicates(&b),
            count_duplicates(&s),
            count_inside(&b, width, height)
        );
    }

    // En la tetera comparamos triángulo por triángulo.
    let mut b_total = 0;
    let mut s_total = 0;
    let mut differences = 0;
    let mut b_duplicates = 0;
    let mut s_duplicates = 0;
    for t in model_triangles {
        let b = sorted_positions(&triangle_barycentric(&t[0], &t[1], &t[2]));
        let s = sorted_positions(&triangle_scanline(&t[0], &t[1], &t[2]));
        b_total += b.len();
        s_total += s.len();
        differences += count_differences(&b, &s);
        b_duplicates += count_duplicates(&b);
        s_duplicates += count_duplicates(&s);
    }
    println!(
        "| Tetera ({} triángulos) | {b_total} | {s_total} | {differences} | {b_duplicates} / {s_duplicates} | {b_total} |",
        model_triangles.len()
    );

    // El orden inverso debe dar las mismas posiciones que el original.
    let cases = test_cases();
    let normal = [cases[1].1.clone()];
    let reversed = [cases[4].1.clone()];
    for mode in [TriangleMode::FilledBarycentric, TriangleMode::FilledScanline] {
        let a = sorted_positions(&rasterize(&normal, mode));
        let b = sorted_positions(&rasterize(&reversed, mode));
        println!(
            "\n{}: orden normal vs inverso → {} diferencias",
            mode_name(mode),
            count_differences(&a, &b)
        );
    }
}

// ---------------------------------------------------------------------------
// --benchmark-triangles
// ---------------------------------------------------------------------------

const RUNS: usize = 100;
const ROUNDS: usize = 7;

pub fn benchmark(scene: &str, triangles: &[[Vertex; 3]]) {
    println!(
        "\n{scene}: {} triángulos, {RUNS} ejecuciones por ronda, {ROUNDS} rondas\n",
        triangles.len()
    );
    println!("| Modo | Fragmentos | Mediana (100 ejecuciones) | Por ejecución |");
    println!("|---|---|---|---|");

    for mode in MODES {
        let fragment_count = rasterize(triangles, mode).len();

        let mut times: Vec<Duration> = Vec::new();
        for _ in 0..ROUNDS {
            let start = Instant::now();
            for _ in 0..RUNS {
                let fragments = rasterize(black_box(triangles), mode);
                black_box(fragments);
            }
            times.push(start.elapsed());
        }
        times.sort();
        let median = times[ROUNDS / 2];

        println!(
            "| {} | {} | {:.1} ms | {:.3} ms |",
            mode_name(mode),
            fragment_count,
            median.as_secs_f64() * 1000.0,
            median.as_secs_f64() * 1000.0 / RUNS as f64
        );
    }
}
