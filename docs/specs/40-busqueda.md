---
id: spec-40
title: Búsqueda dentro del libro
status: borrador-inicial
version: 0.1.0
updated: 2026-11-03
milestone: M4
depends_on: [spec-10, spec-30]
---

# Spec 40 — Búsqueda dentro del libro (borrador)

> **Estado: borrador.** Falta decidir alcance antes de escribir tareas.

## Objetivo

Encontrar una frase en el libro abierto y saltar al resultado. No es búsqueda
global de la biblioteca (eso es otro problema y probablemente otro ADR).

## Diseño candidato

1. **Índice efímero por libro**: al abrir, o al primer `⌘F`, recorrer los capítulos
   y guardar `Vec<(capítulo, índice de bloque, texto normalizado)>`. Un libro de
   100 capítulos tarda ~1 s en la primera búsqueda; después queda en memoria.
2. **Normalización**: minúsculas + eliminación de acentos (`á→a`, `ñ→n`) + colapso
   de espacios. Nada de stemming ni sinónimos.
3. **Resultados**: lista con capítulo, título y 60 caracteres de contexto a cada
   lado, con la coincidencia resaltada. Click → mismo mecanismo de salto de
   spec 30.
4. **Resaltado en el texto**: pintar el match con el color de acento (requiere
   partir el `Run` en tres: antes, match, después) y permitir `n`/`N` entre
   resultados.

## Presupuesto

- Búsqueda en libro de 500 KB: < 300 ms con índice caliente.
- Memoria del índice: aceptable hasta ~5 MB de texto por libro (unos 2 libros
  abiertos).

## Decisiones abiertas (para Hugo)

1. ¿Búsqueda también por autor/título en la biblioteca (M2 ya lo hace sobre
   metadatos) o solo dentro del texto?
2. ¿Índice persistente en `library.json` (crece mucho) o efímero por sesión?
3. ¿Hace falta regex o alcanza substring?

## Fuera de alcance

- Búsqueda entre todos los libros a la vez.
- Búsqueda difusa o por semántica.
