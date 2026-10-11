# Para la dirección de tecnología (CTO)

**La pregunta que probablemente se hace:** ¿está bien construido, puede mi equipo cambiarlo con seguridad y cómo lo sé?

## Qué obtiene

| Asunto | Qué existe | Lea |
| --- | --- | --- |
| Pila | Rust con Loco (Axum, SeaORM) y PostgreSQL 18 para la API; SvelteKit 2 con Svelte 5 y TypeScript para el cliente | [architecture.md](../../../../spec/architecture.md) |
| Regla de diseño | Un núcleo puro: las reglas no dependen de la base de datos ni del reloj, se prueban de forma exhaustiva y luego se conectan a los controladores | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Primero la especificación | Un cambio es requisito, decisión, tarea, código y pruebas juntos; cada entrada de tarea dice qué **no** se verificó | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Pruebas | Pruebas unitarias de las reglas, pruebas de peticiones con base de datos, binarios de aplicación ejecutados con el inicio de sesión activado, pruebas unitarias de la interfaz y de navegador. Comprobaciones de mutación: romper el código a propósito y confirmar que falla una prueba | [testing.md](../../../../spec/testing.md) |
| Recuentos | Generados a partir de los archivos, nunca copiados a mano | [implementation-status.md](../../../../spec/implementation-status.md) |
| Cadena de suministro | `cargo deny` para licencias y prohibiciones, crates compartidos incluidos, una lista de materiales de software en cada publicación | [production-readiness.md](../../../../spec/production-readiness.md) |
| Contrato de la API | OpenAPI y una prueba que falla si una ruta registrada no está documentada | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Control de acceso | La política decide solo por acción; los manejadores y una tabla de rutas clasificada deciden la necesidad de conocer, y un análisis del código fuente falla ante una ruta de lectura sin clasificar | [auth.md](../../../../spec/auth.md) |
| Rendimiento | [BENCHMARKS.md](../../../../BENCHMARKS.md) recoge lo que se midió; no existe una prueba de carga | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Deuda de ingeniería conocida

- Varias vistas cargan organizaciones enteras en memoria.
- Los bloqueos de fila en los planes de conversión están razonados, no demostrados por una prueba.
- La CI, la publicación y el análisis dinámico están escritos pero no se han ejecutado en la plataforma de alojamiento.
- Las Actions se fijan por etiqueta, no por huella, y las imágenes no están firmadas.
- Algunas pruebas de Redis con contraseña y TLS necesitan variables y una autoridad de certificación, así que la CI no las ejecuta.
- El árbol `spec/` es grande; un archivo de estado generado es la forma de ver lo declarado frente a lo que pasa.

## Cómo contribuir

Lea primero [AGENTS.md](../../../../AGENTS.md): cinco reglas y luego un mapa por tarea.

Vea también la página de la [dirección de sistemas de información](../dirección-de-sistemas-de-información/index.md) para la visión operativa.

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
