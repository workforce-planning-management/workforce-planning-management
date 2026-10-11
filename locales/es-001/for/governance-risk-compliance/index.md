# Gobernanza, riesgo y cumplimiento

**Las preguntas que probablemente se hace:** ¿quién es responsable de qué, qué podría salir mal y qué lo impide, y qué evidencia puedo mostrar a un evaluador? Esta página reúne en un solo lugar el material de gobernanza, riesgo y cumplimiento. Las páginas de la [dirección de riesgos](../chief-risk-officer/index.md) y de la [dirección jurídica](../chief-legal-officer/index.md) cubren el mismo terreno por rol.

## Gobernanza: quién decide y cómo queda registrado

| Tema | Qué existe | Lea |
| --- | --- | --- |
| Cómo se gobierna el trabajo | Primero la especificación: requisito, decisión, tarea, código y pruebas llegan juntos; cada tarea dice qué **no** se verificó | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Gobernanza del proyecto | Gobernanza, mantenedores, un proceso de RFC y un archivo de propietarios del código | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Decisiones sobre personas | Decide una persona; las puntuaciones asesoran. Las conversiones, dimisiones y solicitudes de trabajo flexible se registran como decisión de una persona | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Acceso y roles | Los roles y atributos deciden el acceso; una tabla clasificada decide la necesidad de conocer en cada ruta de lectura | [auth.md](../../../../spec/auth.md) |
| Autoría asistida por IA | Una declaración sobre cómo se usó la IA | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Riesgo: qué podría salir mal

| Tema | Qué existe | Lea |
| --- | --- | --- |
| Conclusión | **No está listo** para un despliegue de alta garantía; se encontró un bloqueo probando (las lecturas no seguían la necesidad de conocer) y se corrigió | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Amenazas | Un modelo de amenazas: activos, actores, fronteras, el control tal como está construido y lo que **no se evaluó** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Evidencia de manipulación | Un registro de auditoría de solo añadir, encadenado por hash, con un punto de verificación; las entradas no contienen texto de motivo ni valor | [audit.md](../../../../spec/audit.md) |
| Arranque que falla de forma segura | Producción se niega a arrancar con el inicio de sesión desactivado, sin claves de token, con una conexión de base de datos en claro a otro host o con un Redis remoto en claro | [production-readiness.md](../../../../spec/production-readiness.md) |
| Recuperación | Copias cifradas, un simulacro de restauración ejecutado en la CI, un libro de supresiones para que una restauración no devuelva a una persona suprimida | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Incidentes | Un manual operativo con las primeras acciones; quien despliega añade contactos y vías de escalado | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Sesgo en las puntuaciones | Una plantilla de impacto sobre la igualdad y un registro de cada puntuación, cada una con un borrador de evaluación | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Riesgos abiertos | Fijación y firma de la cadena de suministro, evidencia de carga, comprobaciones de los registros en busca de datos personales, límites de frecuencia compartidos, un conjunto de roles de base de datos de mínimo privilegio, guía sobre revocación de tokens | [readiness-assessment.md](../../../../spec/readiness-assessment.md), sección 3 |

## Cumplimiento: qué puede mostrar a un evaluador

| Tema | Qué existe | Lea |
| --- | --- | --- |
| Autoevaluación de controles | Una lista de comprobación en la que cada respuesta es cumplida, cumplida en parte, no cumplida o a responder por quien despliega | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Evaluación de impacto sobre la protección de datos | Una plantilla, completada para la demostración, con marcadas las partes que usted debe completar | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Registro de tratamientos | Un registro adaptable | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Flujo de datos | Adónde se mueven los datos personales | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Conservación | Un calendario por tipo de registro | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Derechos de los interesados | Exportación de acceso y supresión | [regulatory.md](../../../../spec/regulatory.md) |
| Datos de categoría especial | La monitorización de la igualdad y los requisitos de salud en el trabajo están desactivados hasta registrar una base jurídica, y luego acotados | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Accesibilidad e idioma | Una declaración de accesibilidad y una declaración de lengua oficial | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Licencias y dependencias | Archivos de licencia, una política de dependencias comprobada con `cargo deny` y una lista de materiales de software en cada publicación | [production-readiness.md](../../../../spec/production-readiness.md) |

## Lo que solo pueden hacer las personas

Son puertas, no código, y el repositorio no las marca como hechas: una prueba de penetración independiente, una auditoría de accesibilidad independiente, su evaluación de impacto sobre la protección de datos firmada por su responsable, un simulacro de restauración en su plataforma, una revisión profesional de las traducciones y su aprobación de las evaluaciones de impacto sobre la igualdad.

## Límites de esta página

Las tablas apuntan a documentos. Si cada documento es cierto para su despliegue lo deben juzgar usted y su evaluador. Las pruebas del autor no son una revisión independiente, y nada de lo aquí escrito es asesoramiento jurídico ni una certificación.

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
