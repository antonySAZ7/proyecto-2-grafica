# Proyecto
# Antony Saz



## Como ejecutar

```bash
cargo run
```

Esto abre una ventana con `minifb`.

Controles:

- `A` / `D` o flechas izquierda/derecha: rotar alrededor del diorama.
- `W` / `S` o flechas arriba/abajo: acercar y alejar la camara.
- `R`: reiniciar la camara.
- `Esc`: cerrar la ventana.

Para renderizar sin abrir ventana:

```bash
cargo run -- --no-window
```

Para generar frames de una animacion orbital con zoom:

```bash
cargo run -- --frames 60
```

Los frames quedan en:

```text
frames/frame_0000.ppm
frames/frame_0001.ppm
...
```

## Video

![Animacion del diorama](media/diorama_gatos.gif)


