# Lab: Rellenar triángulos

En este lab la rasterización ya no solo genera el contorno de los triángulos: ahora también puede rellenarlos. Hice dos algoritmos de relleno (coordenadas baricéntricas y líneas horizontales), comprobé que generan los mismos fragmentos y comparé cuánto tarda cada uno.

Para probar uso la tetera (6320 triángulos) y los triángulos de prueba que pide el lab.

## Imágenes de cada modo

| Wireframe | Baricéntrico | Scanline |
|---|---|---|
| ![](capturas/tetera_wireframe.png) | ![](capturas/tetera_baricentrico.png) | ![](capturas/tetera_scanline.png) |
| ![](capturas/pruebas_wireframe.png) | ![](capturas/pruebas_baricentrico.png) | ![](capturas/pruebas_scanline.png) |

Por ahora todos los fragmentos son blancos, así que la tetera rellena se ve como una silueta.

## Cómo correrlo

```
cargo run --release                              # ventana
cargo run --release -- --compare                 # compara los dos rellenos
cargo run --release -- --benchmark-triangles     # mediciones
cargo run --release -- --screenshot capturas     # guarda las imágenes de arriba
cargo flamegraph -- --benchmark-triangles        # perfil
```

En la ventana: 1 es wireframe, 2 baricéntrico, 3 scanline y T cambia entre la tetera y los triángulos de prueba.

## Cambios en el pipeline

`render` sigue igual, con sus cuatro etapas: procesamiento de vértices, ensamblaje de primitivas, rasterización y procesamiento de fragmentos. Lo único que cambia es que `rasterize` ahora recibe el modo:

```rust
#[derive(Clone, Copy)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
    FilledScanline,
}

pub fn rasterize(triangles: &[[Vertex; 3]], mode: TriangleMode) -> Vec<Fragment>
```

Según el modo, llama a una de estas funciones. Las tres reciben los tres vértices y devuelven un `Vec<Fragment>`:

- `triangle`: el wireframe de antes (tres llamadas a `line`).
- `triangle_barycentric`: relleno con coordenadas baricéntricas.
- `triangle_scanline`: relleno con líneas horizontales.

## Los dos algoritmos

**Baricéntrico:** recorre el rectángulo más pequeño que contiene al triángulo y evalúa el centro de cada píxel `(x + 0.5, y + 0.5)`. Para cada centro calcula las tres coordenadas baricéntricas. Si las tres son `>= 0`, el punto está dentro y se genera un fragmento. Como se divide entre el área con signo, el orden de los vértices no importa. Si el área es 0, no genera nada.

**Scanline:** avanza fila por fila. En cada fila calcula dónde cruza cada arista la línea `y + 0.5` y genera fragmentos entre el cruce más a la izquierda y el más a la derecha. Las aristas horizontales no se pueden usar para calcular un cruce (se dividiría entre 0), así que si están justo en la fila se toma todo su tramo y si no, se ignoran. Un vértice que cae justo en una fila no genera fragmentos repetidos, porque cada fila produce un solo tramo.

Los dos usan la misma regla: un píxel se pinta si su centro está dentro del triángulo o justo sobre un borde. Los cálculos los hice con `f64`. Con `f32` me salió 1 píxel de diferencia en la tetera, por redondeo en un centro que caía casi sobre una arista.

## Comparación

Con `--compare` se comparan las posiciones de los fragmentos de los dos algoritmos, no solo la imagen:

| Caso | Fragmentos (B / S) | Diferencias |
|---|---|---|
| Pequeño | 34 / 34 | 0 |
| Grande | 112255 / 112255 | 0 |
| Borde horizontal | 18825 / 18825 | 0 |
| Borde vertical | 25000 / 25000 | 0 |
| Grande en orden inverso | 112255 / 112255 | 0 |
| Parcialmente fuera | 43875 / 43875 | 0 |
| Tres vértices en línea | 0 / 0 | 0 |
| Tetera (6320 triángulos) | 230846 / 230846 | 0 |

Los dos generan exactamente las mismas posiciones, sin repetidos, y el orden inverso da el mismo resultado que el normal. No hay diferencias en los bordes porque los dos usan la misma regla.

En el triángulo parcialmente fuera se generan fragmentos para todo el triángulo, pero `framebuffer.point` descarta los que quedan fuera de la ventana. Los tres vértices en línea aparecen como una línea en wireframe, pero en los rellenos no generan nada porque no tienen interior.

## Mediciones

Solo se mide `rasterize`, sin la textura ni la ventana. Cada modo se ejecuta 100 veces por ronda, en 7 rondas, y se toma la mediana. Se usa `black_box` para que el compilador no se salte trabajo.

**Tetera (6320 triángulos pequeños)**

| Modo | Fragmentos | Mediana (100 ejecuciones) |
|---|---|---|
| Wireframe | 227558 | 268.2 ms |
| Baricéntrico | 230846 | 362.5 ms |
| Scanline | 230846 | 224.3 ms |

**Triángulos de prueba (7 triángulos, algunos grandes)**

| Modo | Fragmentos | Mediana (100 ejecuciones) |
|---|---|---|
| Wireframe | 5235 | 1.8 ms |
| Baricéntrico | 312244 | 193.3 ms |
| Scanline | 312244 | 90.4 ms |

Los dos rellenos generan la misma cantidad de fragmentos, así que la diferencia de tiempo no viene de que uno haga menos trabajo.

## Perfil

![Flamegraph](flamegraph.svg)

Generado con `cargo flamegraph -- --benchmark-triangles`. Para que salgan los nombres de las funciones agregué `debug = true` en `[profile.release]`.

- `triangle_barycentric` ocupa 43.7% de las muestras.
- `triangle_scanline` ocupa 22.7%.
- `triangle` (wireframe) ocupa 20.7%.
- Hacer crecer el `Vec` (`grow_amortized`, `realloc`) aparece en un 20.2%.
- El cálculo de las coordenadas (`edge`) solo aparece en un 10.8%.

## Qué encontré

- **El scanline es más rápido** (unas 2.1 veces con triángulos grandes y 1.6 con los de la tetera). El baricéntrico revisa todo el rectángulo, y cerca de la mitad de esos píxeles quedan fuera del triángulo. El scanline hace las cuentas una vez por fila y solo agrega los píxeles que sí están dentro.
- **El wireframe no es mucho más rápido** aunque solo dibuja bordes. Cada `line` crea su propio `Vec`, que después se copia en el de `triangle` y luego en el de `rasterize`.
- **Se gasta más tiempo en memoria que en geometría.** Hacer crecer y copiar los `Vec` pesa más que calcular las coordenadas. Se podría mejorar reservando espacio con `Vec::with_capacity` o escribiendo todo en un solo `Vec`.
