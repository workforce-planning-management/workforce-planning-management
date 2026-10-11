# Para la dirección de operaciones (COO)

**La pregunta que probablemente se hace:** ¿tenemos a las personas adecuadas, en el lugar adecuado y en el momento adecuado, y qué se rompe si alguien falta?

## Qué obtiene

| Quiere saber | Dónde responde WPM | Lea |
| --- | --- | --- |
| Si un equipo puede cubrir una ausencia | Suplentes por persona, el turno de guardia con cambios y recordatorios, y una vista de cobertura que dice con claridad cuándo no hay nadie disponible | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Tiempo, asistencia y permisos | Registros de tiempo, solicitudes de permiso con circuito de aprobación, devengos y turnos | [workforce-management.md](../../../../spec/workforce-management.md) |
| Si se respetan los límites de jornada | Salvaguardas de jornada de carácter consultivo; avisan y no bloquean | WPM-R27, el mismo archivo |
| Capacidad por habilidad, mes a mes | Bolsas de habilidades, oferta y demanda de programas, compromisos de socios, la bolsa restrictiva y una comprobación de inicio que es una sugerencia, con decisiones de inicio registradas | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Qué necesitaremos y dónde está la brecha | Planes de plantilla, previsión y análisis de brechas por departamento y por rol | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Qué contratos terminan pronto | Una fecha de fin según la base del contrato, prórrogas con historial, recordatorios antes del fin, una lista de contratos sin fecha de fin y planes de conversión | [engagements.md](../../../../spec/engagements.md) |
| Dónde el personal contingente es una parte grande | La parte de las FTE que son contratistas y la que es temporal, por separado, y la financiación que termina mes a mes | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Altas y bajas | Listas de comprobación con fechas y un traspaso el último día | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Brechas de habilidades y tiempo de formación | Análisis de brechas con la evidencia en que se apoya una recomendación | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Cómo leer la vista de capacidad

La capacidad se planifica por bolsa de habilidades, no por persona. Cada mes de una bolsa da uno de cuatro resultados, y una habilidad no declarada es *desconocida*, no un déficit. La comprobación de inicio es una sugerencia, y una persona registra la decisión de empezar. La planificación de cadena crítica y de ruta crítica no está aquí: pertenece a project-portfolio-management.

## Límites

- La parte contingente aún no tiene tendencia: necesita instantáneas de plantilla separadas por base (WPM-T139, no construido).
- Una bolsa en la vista contingente es el departamento hasta que los planes definan bolsas.
- La planificación de turnos y los cálculos del impuesto de nómina no sustituyen sus sistemas de turnos o de nómina.
- Varias vistas cargan en memoria los registros de una organización; no se ha medido ninguna cifra de carga ([readiness-assessment.md](../../../../spec/readiness-assessment.md), brecha 8).

Vea también la [dirección de personas](../chief-people-officer/index.md) y la [dirección general](../chief-executive-officer/index.md).

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
