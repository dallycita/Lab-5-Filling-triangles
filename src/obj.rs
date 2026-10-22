use raylib::prelude::*;
use std::fs;

use crate::vertex::Vertex;

/// Modelo cargado desde un archivo OBJ.
/// `indices` guarda los vértices de cada triángulo de 3 en 3.
pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<usize>,
}

/// Lee un archivo OBJ y devuelve sus vértices (`v`) e índices de caras (`f`).
pub fn load_obj(path: &str) -> Result<Obj, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("No se pudo leer '{path}': {e}"))?;

    let mut vertices: Vec<Vector3> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();

    for (line_number, line) in content.lines().enumerate() {
        let mut parts = line.split_whitespace();

        match parts.next() {
            // Vértice: "v x y z"
            Some("v") => {
                let coords: Vec<f32> = parts
                    .take(3)
                    .map(|s| s.parse::<f32>())
                    .collect::<Result<_, _>>()
                    .map_err(|e| format!("Línea {}: número inválido ({e})", line_number + 1))?;

                if coords.len() != 3 {
                    return Err(format!("Línea {}: el vértice necesita x, y, z", line_number + 1));
                }
                vertices.push(Vector3::new(coords[0], coords[1], coords[2]));
            }

            // Cara: "f 1 2 3" o "f 1/4/2 5/7/2 3/2/2"
            Some("f") => {
                let mut face: Vec<usize> = Vec::new();
                for group in parts {
                    // Solo nos interesa el primer número de cada grupo.
                    let first = group.split('/').next().unwrap_or("");
                    let obj_index: usize = first
                        .parse()
                        .map_err(|e| format!("Línea {}: índice inválido ({e})", line_number + 1))?;
                    // OBJ empieza en 1, un Vec de Rust empieza en 0.
                    let index = obj_index
                        .checked_sub(1)
                        .ok_or(format!("Línea {}: los índices OBJ empiezan en 1", line_number + 1))?;
                    face.push(index);
                }

                if face.len() < 3 {
                    return Err(format!("Línea {}: la cara necesita 3 vértices", line_number + 1));
                }

                // Lo normal es que la cara ya sea un triángulo.
                // Si trae más vértices, la dividimos en abanico: (0,1,2), (0,2,3), ...
                for i in 1..face.len() - 1 {
                    indices.push(face[0]);
                    indices.push(face[i]);
                    indices.push(face[i + 1]);
                }
            }

            // Cualquier otra línea (vn, vt, comentarios, etc.) se ignora por ahora.
            _ => {}
        }
    }

    // Comprobamos que ningún índice apunte fuera de la lista de vértices.
    if let Some(&bad) = indices.iter().find(|&&i| i >= vertices.len()) {
        return Err(format!(
            "El índice {} no existe (solo hay {} vértices)",
            bad + 1,
            vertices.len()
        ));
    }

    Ok(Obj { vertices, indices })
}

impl Obj {
    /// Devuelve los vértices ordenados según las caras: cada grupo de tres
    /// vértices consecutivos es un triángulo.
    pub fn get_vertex_array(&self) -> Vec<Vertex> {
        let mut vertex_array = Vec::new();
        for &index in &self.indices {
            let position = self.vertices[index];
            vertex_array.push(Vertex::new_with_color(position, Vector3::new(1.0, 1.0, 1.0)));
        }
        vertex_array
    }
}
