# Para la dirección de riesgos (CRO)

**La pregunta que probablemente se hace:** ¿qué podría salir mal con este sistema, qué lo impide y qué no ha comprobado nadie?

## El punto de partida honesto

La [evaluación de preparación](../../../../spec/readiness-assessment.md) del 2026-10-11 da la conclusión **no está listo** para un despliegue de alta garantía. Encontró un bloqueo probando el servicio en marcha: cualquier persona con sesión iniciada podía leer la baja por enfermedad de un compañero y su motivo en texto libre. Está corregido y probado. Las puertas que solo las personas pueden cerrar siguen abiertas.

## Qué obtiene

| Asunto | Qué existe | Lea |
| --- | --- | --- |
| Amenazas en cada frontera | Un modelo de amenazas: activos, actores, fronteras, el control tal como está construido y lo que **no se evaluó** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Lecturas no autorizadas | Una protección de necesidad de conocer que clasifica cada ruta de lectura; una prueba falla ante una sin clasificar | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Manipulación y rendición de cuentas | Un registro de auditoría de solo añadir, encadenado por hash, con un punto de verificación; las entradas no contienen texto de motivo ni valor | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Datos sensibles | No se guarda ningún diagnóstico; funciones de categoría especial desactivadas por defecto; un umbral para grupos pequeños | [regulatory.md](../../../../spec/regulatory.md) |
| Caminos inseguros | Producción se niega a arrancar con el inicio de sesión desactivado, sin fuente de claves de token, con una conexión de base de datos en claro a otro host o con un Redis remoto en claro | [production-readiness.md](../../../../spec/production-readiness.md) |
| Pérdida de datos | Copias cifradas, un simulacro de restauración ejecutado en la CI, un libro de supresiones | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Sesgo en las puntuaciones | Una plantilla de impacto sobre la igualdad y un registro de cada puntuación con un borrador de evaluación | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Una autoevaluación de controles | Una lista de comprobación en la que cada respuesta es cumplida, cumplida en parte, no cumplida o a responder por quien despliega | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Riesgos abiertos, tal como constan

La fijación y firma de la cadena de suministro, la evidencia de carga, las comprobaciones de los registros en busca de datos personales, los límites de frecuencia compartidos entre instancias, un conjunto de roles de base de datos de mínimo privilegio y la guía sobre revocación de tokens son puntos abiertos de la evaluación de preparación, cada uno con una tarea. Las pruebas las escribió el autor; ninguna es una revisión independiente.

## Lo que solo pueden hacer las personas

Una prueba de penetración independiente, una auditoría de accesibilidad independiente, la evaluación de impacto de quien despliega firmada, un simulacro de restauración en su plataforma y la aprobación de las evaluaciones de impacto sobre la igualdad. El repositorio las declara como puertas y no las marca como hechas.

Vea también la [dirección de sistemas de información](../dirección-de-sistemas-de-información/index.md) y la [dirección jurídica](../dirección-jurídica/index.md).

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
