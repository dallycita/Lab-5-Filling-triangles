mod fragment;
mod framebuffer;
mod line;
mod obj;
mod primitive;
mod pruebas;
mod rasterizer;
mod shaders;
mod triangle;
mod vertex;

use raylib::prelude::*;

use framebuffer::Framebuffer;
use obj::{Obj, load_obj};
use primitive::primitive_assembly;
use pruebas::{MODES, mode_name, test_vertex_array};
use rasterizer::rasterize;
use shaders::{fragment_shader, vertex_shader};
use triangle::TriangleMode;
use vertex::Vertex;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const MODEL_PATH: &str = "assets/teapot.obj";

pub struct Uniforms {
    pub model_matrix: Matrix,
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    mode: TriangleMode,
) {
    // 1. Procesamiento de vértices
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();

    // 2. Ensamblaje de primitivas
    let triangles = primitive_assembly(&transformed_vertices);

    // 3. Rasterización
    let fragments = rasterize(&triangles, mode);

    // 4. Procesamiento de fragmentos
    for fragment in fragments {
        let color = fragment_shader(&fragment);
        framebuffer.point(fragment.position.x as i32, fragment.position.y as i32, color);
    }
}

/// Matriz de modelo que escala y traslada el modelo para que quede centrado
/// y ocupe el 80% de la ventana. La escala en y es negativa porque en
/// pantalla y crece hacia abajo.
fn fit_model_matrix(obj: &Obj) -> Matrix {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for v in &obj.vertices {
        min_x = min_x.min(v.x);
        min_y = min_y.min(v.y);
        max_x = max_x.max(v.x);
        max_y = max_y.max(v.y);
    }

    let model_width = (max_x - min_x).max(f32::EPSILON);
    let model_height = (max_y - min_y).max(f32::EPSILON);
    let scale = (WIDTH as f32 * 0.8 / model_width).min(HEIGHT as f32 * 0.8 / model_height);
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    let mut matrix = Matrix::identity();
    matrix.m0 = scale;
    matrix.m5 = -scale;
    matrix.m10 = scale;
    matrix.m12 = WIDTH as f32 / 2.0 - center_x * scale;
    matrix.m13 = HEIGHT as f32 / 2.0 + center_y * scale;
    matrix
}

/// Corre el procesamiento de vértices y el ensamblaje (sin rasterizar).
/// Lo usan las pruebas y las mediciones.
fn assemble(vertex_array: &[Vertex], uniforms: &Uniforms) -> Vec<[Vertex; 3]> {
    let transformed_vertices: Vec<Vertex> = vertex_array
        .iter()
        .map(|vertex| vertex_shader(vertex, uniforms))
        .collect();
    primitive_assembly(&transformed_vertices)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let obj = match load_obj(MODEL_PATH) {
        Ok(obj) => obj,
        Err(e) => {
            eprintln!("Error al cargar el modelo: {e}");
            std::process::exit(1);
        }
    };

    // Dos escenas: la tetera y los triángulos de prueba del lab.
    let teapot_vertices = obj.get_vertex_array();
    let teapot_uniforms = Uniforms { model_matrix: fit_model_matrix(&obj) };

    // Los triángulos de prueba ya están en coordenadas de pantalla,
    // así que usan la matriz identidad.
    let test_vertices = test_vertex_array();
    let test_uniforms = Uniforms { model_matrix: Matrix::identity() };

    // --- Modos sin ventana ---

    if args.iter().any(|a| a == "--benchmark-triangles") {
        pruebas::benchmark("Tetera", &assemble(&teapot_vertices, &teapot_uniforms));
        pruebas::benchmark("Triángulos de prueba", &assemble(&test_vertices, &test_uniforms));
        return;
    }

    if args.iter().any(|a| a == "--compare") {
        pruebas::compare(&assemble(&teapot_vertices, &teapot_uniforms), WIDTH, HEIGHT);
        return;
    }

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0));

    if let Some(i) = args.iter().position(|a| a == "--screenshot") {
        let dir = args.get(i + 1).map(|s| s.as_str()).unwrap_or("capturas");
        std::fs::create_dir_all(dir).expect("No se pudo crear la carpeta");
        for mode in MODES {
            framebuffer.clear();
            render(&mut framebuffer, &teapot_uniforms, &teapot_vertices, mode);
            framebuffer.render_to_file(&format!("{dir}/tetera_{}.png", mode_name(mode)));

            framebuffer.clear();
            render(&mut framebuffer, &test_uniforms, &test_vertices, mode);
            framebuffer.render_to_file(&format!("{dir}/pruebas_{}.png", mode_name(mode)));
        }
        println!("Capturas guardadas en {dir}/");
        return;
    }

    // --- Ventana ---
    // 1, 2 y 3 cambian el modo; T cambia entre la tetera y las pruebas.

    let (mut window, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Lab - Rellenar triangulos")
        .build();
    window.set_target_fps(60);
    framebuffer.init_texture(&mut window, &thread);

    let mut mode = TriangleMode::FilledBarycentric;
    let mut show_tests = false;

    while !window.window_should_close() {
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            mode = TriangleMode::Wireframe;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            mode = TriangleMode::FilledBarycentric;
        }
        if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            mode = TriangleMode::FilledScanline;
        }
        if window.is_key_pressed(KeyboardKey::KEY_T) {
            show_tests = !show_tests;
        }

        let scene = if show_tests { "pruebas" } else { "tetera" };
        window.set_window_title(&thread, &format!("Lab - {scene} - {}", mode_name(mode)));

        framebuffer.clear();
        if show_tests {
            render(&mut framebuffer, &test_uniforms, &test_vertices, mode);
        } else {
            render(&mut framebuffer, &teapot_uniforms, &teapot_vertices, mode);
        }
        framebuffer.swap_buffers(&mut window, &thread);
    }
}
