# Proyecto 2: Diorama con Raytracing

Proyecto en Rust para construir un pequeno diorama con cubos texturizados usando raytracing.

## Objetivo

Crear una escena tipo diorama con materiales distintos, reflexion, refraccion, skybox, rotacion de la escena y control de zoom de camara.

## Restricciones

- No se usan librerias externas.
- El proyecto se desarrolla en fases y cada fase debe quedar en un commit separado.
- La entrega final debe incluir un video del diorama en este README.

## Plan por fases

1. **Base del proyecto**
   - Inicializar Cargo, Git y README.
   - Por que: deja una base limpia y verificable antes de implementar el raytracer.

2. **Raytracer minimo**
   - Implementar vectores, rayos, camara, color y salida a imagen PPM.
   - Por que: primero necesitamos comprobar que podemos generar imagenes sin depender de librerias externas.

3. **Cubos y texturas**
   - Agregar interseccion con cubos y texturas procedurales por material.
   - Por que: el diorama se evalua con cubos texturizados, parecido a bloques tipo Minecraft.

4. **Diorama completo**
   - Construir una escena con piso, paredes, objetos y al menos 5 materiales.
   - Por que: cubre complejidad visual y prepara la parte subjetiva de la nota.

5. **Efectos de raytracing**
   - Agregar iluminacion, sombras, reflexion, refraccion y skybox.
   - Por que: estos son puntos directos de la rubrica y hacen la escena mas atractiva.

6. **Camara y animacion**
   - Implementar rotacion del diorama y zoom de camara; renderizar frames.
   - Por que: la rubrica pide rotacion y acercamiento/alejamiento, y esos frames serviran para el video.

7. **Pulido y entrega**
   - Ajustar composicion, rendimiento, README, instrucciones y enlace/video final.
   - Por que: deja el proyecto listo para GitHub y para calificacion.

## Como ejecutar

```bash
cargo run
```

La Fase 1 genera una imagen PPM en:

```text
renders/fase1_cielo.ppm
```

La Fase 2 genera cubos texturizados en:

```text
renders/fase2_cubos.ppm
```

La Fase 3 genera el diorama con cinco materiales en:

```text
renders/fase3_diorama.ppm
```

## Estado

- [x] Fase 0: base del proyecto
- [x] Fase 1: raytracer minimo
- [x] Fase 2: cubos y texturas
- [x] Fase 3: diorama completo
- [ ] Fase 4: efectos de raytracing
- [ ] Fase 5: camara y animacion
- [ ] Fase 6: pulido y entrega
