# Pruebas de seguridad independientes

**La pregunta que probablemente se hace:** ¿cómo conseguimos que personas que no escribieron este software lo ataquen, hacia qué las dirigimos y qué hacemos con lo que encuentren?

No se ha hecho ninguna prueba independiente. El [modelo de amenazas](../../../../spec/governance/threat-model.md) lo escribió el mismo asistente que escribió el software y nadie más lo revisó; cada control que nombra lo ejercitó ese mismo autor. La [evaluación de preparación](../../../../spec/readiness-assessment.md) enumera una prueba de penetración independiente como una puerta que solo las personas pueden cerrar (WPM-T201, **sin hacer**). Esta página es un informe de contexto para quien encargue una. No es un plan de pruebas de un probador y no lo sustituye.

## Qué entregar a los probadores

| Elemento | Dónde está | Notas |
| --- | --- | --- |
| Un despliegue que atacar | El suyo: [guía de despliegue](../../../../spec/operations/deployment.md) | Ajustes de producción con el inicio de sesión **activado**, un proveedor de identidad de pruebas y **solo datos sintéticos**. No pruebe con registros de personas reales |
| El código fuente | Este repositorio | Una prueba con el código fuente es más exhaustiva que una de caja negra; diga cuál quiere |
| El modelo de amenazas | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Veinticuatro amenazas, el control tal como está construido y las brechas. Entréguelo **después** de una primera pasada de caja negra si quiere ver qué encuentran sin pistas |
| La descripción de la API | `GET /api-docs/openapi.json` | Todas las rutas, con la `Idempotency-Key` en cada operación que cambia algo |
| El diseño del inicio de sesión | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Personas, atributos y que la política decide solo por acción |
| Identidades de prueba | Las crea usted | Al menos: una persona sin atributos, un responsable directo, un responsable fuera de la cadena, RR. HH., nómina, una administración y un par de servicio |

## Qué pedirles que examinen

El autor no pudo comprobar esto. Cada fila nombra la entrada del propio modelo de amenazas, para poder rastrear un hallazgo.

| Área | Qué probar | Amenaza |
| --- | --- | --- |
| Tokens | Confusión de algoritmos; un conjunto de claves que rota mientras hay peticiones en curso; un token caducado o aún no válido; un token para otra audiencia | T1, T2 |
| Necesidad de conocer | Cada ruta de lectura como un llamante sin atributos y como un vecino; una ruta que la tabla de clasificación no conoce; una escritura que la política permite pero que el manejador debería rechazar | T3, T4, T6 |
| Atributos privilegiados | Si un atributo de un token puede añadirse, reproducirse o heredarse; quién puede emitirlo; una administración compartida o de larga duración | T24 |
| Inyección | Fuzzing de cada parámetro, segmento de ruta y campo del cuerpo; las sentencias de supresión y conservación se construyen con identificadores tipados y se leyeron, no se sometieron a fuzzing | T12 |
| El cliente de navegador | Gestión de sesiones por el servidor de la interfaz, cómo cambia una sesión por un token, scripting entre sitios mediante texto almacenado, elusión de la política de seguridad de contenido y la página del gestor de contenidos que carga un script de una red pública sin versión fijada | T13, T14 |
| Peticiones salientes | Falsificación de peticiones del lado del servidor mediante cualquier URL configurada (conjuntos de claves, los servicios hermanos, la caché compartida) | T15 |
| Límites | El límite del cuerpo, el tiempo de espera de la petición, los límites de frecuencia por instancia y las vistas que cargan en memoria los registros de una organización entera | T16, T17 |
| Registros | Si algún dato personal, token o cadena de consulta llega a un registro | T19 |
| Claves de idempotencia | Una clave usada por dos llamantes; la misma clave con un cuerpo, ruta o consulta cambiados; una clave casi válida; un cuerpo muy grande; dos peticiones en carrera bajo una misma clave (la reclamación está razonada, no probada en carrera); si alguien aparte de su llamante puede leer una respuesta almacenada | nuevo |
| El registro de auditoría | Cualquier forma de cambiar, quitar u omitir una entrada; el punto de verificación de la cadena | T7 |
| Copias de seguridad | Restaurar desde una copia manipulada o con la clave equivocada; si una persona suprimida regresa | T10, T11 |
| Cadena de suministro | Fijación de las acciones de compilación, análisis de secretos, firma de imágenes | T20, T21 |

## Reglas que fijar antes de empezar

Complételas con sus propios equipos de seguridad y jurídico. Ninguna se decide aquí.

- **Autoridad:** quién firma que la prueba está autorizada, y para qué sistemas y direcciones.
- **Ventana:** las fechas y horas, contadas en días naturales, y a quién llamar para detenerla.
- **Alcance:** qué entra (la API, la interfaz, el flujo de inicio de sesión, el script de copia de seguridad) y qué queda fuera (el propio servicio del proveedor de identidad, su red, cualquier cosa compartida con otros sistemas).
- **Datos:** solo sintéticos. Si un probador encuentra datos personales reales, a quién se avisa y cómo se tratan.
- **Pruebas destructivas:** si se permiten la denegación de servicio, las escrituras masivas y la supresión, y dónde.
- **Informe:** el formato, la escala de gravedad, quién lo recibe, cuánto tiempo pueden conservarlo y cómo se tratan los secretos que encuentren.
- **Repetición de la prueba:** que el contrato incluya comprobar las correcciones.

## Qué hacer con los hallazgos

1. Registre cada uno como una tarea en [tasks.md](../../../../spec/tasks.md) con la referencia del probador, la evidencia y la gravedad que dieron. Un hallazgo discutido sigue registrado como discutido, con el motivo.
2. Corrija con una prueba que falle primero. Si la corrección es una regla, añádala a la columna de brechas del modelo de amenazas y ciérrela allí solo cuando la repetición de la prueba pase.
3. No marque WPM-T201 como hecha hasta tener el informe, que cada hallazgo tenga responsable y que haya terminado la repetición de la prueba. Actualice la [evaluación de preparación](../../../../spec/readiness-assessment.md) y la lista de puertas (WPM-T203) en el mismo cambio.
4. Mantenga el informe fuera del repositorio si contiene algo explotable; enlace a donde se conserva.

## Qué significa un informe limpio y qué no

Una prueba sin hallazgos muestra que estos probadores, en esta ventana, con este alcance y estas identidades, no encontraron nada. No muestra la ausencia de fallos, no cubre una configuración que cambió después y no sustituye a las demás puertas: una auditoría de accesibilidad, su evaluación de impacto sobre la protección de datos firmada por su responsable y un simulacro de restauración en su plataforma.

## Qué no hace esta página

No nombra a un proveedor, no fija un precio ni promete un estándar. Qué esquema o acreditación espera su sector de un probador es decisión suya.

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
