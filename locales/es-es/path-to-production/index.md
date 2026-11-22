# El camino hacia la producción

WPM es software de demostración. Esta página dice qué significaría usarlo en producción y enumera lo que separa a WPM de ese uso. Cada elemento lleva una estimación de su grado de preparación, de 0 a 100 por ciento.

## Qué significa producción

El uso en producción significa que una organización depende de WPM para trabajo real:

- los registros son de personas reales: empleados, candidatos y sus contactos de emergencia;
- el salario que calcula WPM se paga, y se le aplica la legislación del lugar de trabajo;
- las personas inician sesión cada día, y parte de lo que leen es sensible, como las ausencias por enfermedad;
- la organización responde ante un regulador, un auditor y las personas de los registros por la forma en que se gestiona.

Esta página mide WPM frente al caso más exigente para el que se evaluó: una gran organización sanitaria o gubernamental ([la evaluación de preparación](../../../spec/readiness-assessment.md), en inglés). Una organización más pequeña puede necesitar menos de algunos elementos, pero necesita los mismos tipos de evidencia.

La producción necesita tres tipos de evidencia, y el software por sí solo aporta únicamente el primero:

1. **Software** que hace el trabajo de forma segura, con pruebas que lo demuestran.
2. **Quien lo despliega**, que lo aloja y lo opera: copias de seguridad, monitorización, soporte y respuesta a incidentes.
3. **Personas distintas del autor** que lo han comprobado: un auditor de pruebas de penetración, un auditor de accesibilidad, traductores y el delegado de protección de datos de quien lo despliega.

## Cómo leer las estimaciones

- Una estimación es el juicio del autor el 2026-11-13, hecho a partir de la especificación y las pruebas. No es una medición ni una certificación.
- 0 por ciento significa que todavía no existe nada. 100 por ciento significa construido, probado y comprobado por alguien independiente en un despliegue real. Ningún elemento está al 100 por ciento.
- Cada elemento dice en qué se basa su estimación y qué falta.
- Las estimaciones no se promedian. Basta un elemento con una estimación baja para mantener a WPM fuera de producción.

## El software

- **75%** **Inicio de sesión y acceso según la necesidad de conocer.** El inicio de sesión está activado por defecto, y el servicio se niega a arrancar en producción sin él (WPM-R72). Las lecturas se limitan a las personas con necesidad de conocer (WPM-T188), y un token tiene una antigüedad máxima (WPM-T200). Se basa en: las pruebas de aplicación de controles que se ejecutan en la CI. Falta: no hay lista de revocación de tokens, y no se ha realizado ninguna prueba de penetración independiente.
- **70%** **Un registro de auditoría completo e inalterado.** El registro de auditoría es de solo añadir, está encadenado por hash y puede exportarse a un sistema de monitorización (WPM-T189, WPM-T197). Se basa en: pruebas que se rompieron a propósito para demostrar que fallan. Falta: no se detecta un final de la cadena truncado, y el propietario de la base de datos todavía puede desactivar los disparadores.
- **70%** **Protección de datos desde el diseño.** Las personas se identifican mediante referencias, no hay columnas demográficas, y existen el acceso del interesado, la supresión y la conservación para cada tipo de registro. Se basa en: las pruebas y la sección 1 de la evaluación de preparación. Falta: la evaluación y la base jurídica propias de quien lo despliega (más abajo).
- **75%** **Conexiones y copias de seguridad cifradas.** En producción el servicio rechaza una conexión a la base de datos sin cifrar y corta por tiempo de espera una petición que se queda colgada (WPM-T191), y las copias de seguridad están cifradas (WPM-T192). Se basa en: las pruebas de ambas cosas. Falta: la gestión de claves y certificados corresponde a quien lo despliega.
- **10%** **Cálculos legales.** Los cálculos legales son ilustrativos, y el impuesto sobre la nómina es un esbozo ([la posición regulatoria](../../../spec/regulatory.md), en inglés). Se basa en: la especificación, que lo dice. Falta: las reglas de cada lugar de trabajo, escritas y comprobadas por una persona cualificada. No pague a nadie desde WPM.
- **50%** **Escala.** En un solo portátil, las páginas del directorio y la vista de capacidad no fallan con 50.000 trabajadores, y las vistas de planificación rechazan más de 10.000 (WPM-T196, WPM-T230). Se basa en: una ejecución medida ([BENCHMARKS.md](../../../BENCHMARKS.md), en inglés). Falta: la vista de capacidad sigue creciendo en proporción a la plantilla, y no se ha medido nada en el hardware de quien lo despliega.
- **65%** **Cadena de suministro.** Cada acción de la CI está fijada a un commit y el historial se analiza en busca de secretos. La imagen del contenedor está firmada y lleva su procedencia, y esto se ejecutó para la versión v0.1.2 (WPM-T194, WPM-T195). Se basa en: esa ejecución de la publicación. Falta: el paquete de la imagen es privado, así que nadie ajeno al proyecto puede verificarlo todavía.
- **60%** **Monitorización.** Cada respuesta lleva un identificador de petición, y las líneas del registro no llevan cadena de consulta ni identificador personal (WPM-T197). Se basa en: las pruebas del registro de peticiones. Falta: los paneles y las alertas corresponden a quien lo despliega, y los límites de frecuencia se aplican a una sola instancia (WPM-T198).
- **60%** **Copia de seguridad y restauración.** Copias de seguridad cifradas, un simulacro de restauración que se ejecuta en la CI y un manual de operaciones. Se basa en: el simulacro. Falta: un simulacro de restauración en la propia plataforma de quien lo despliega en los últimos 90 días naturales (puerta WPM-G4).
- **40%** **Comprobaciones de accesibilidad.** Se ejecutan comprobaciones automáticas frente a WCAG 2.2 AA en la aplicación y en este sitio, con los temas claro y oscuro. Se basa en: las pruebas de Playwright y axe. Falta: la auditoría independiente (más abajo); no se ha usado ningún lector de pantalla.
- **20%** **Traducciones.** La aplicación tiene 25 configuraciones regionales y las páginas para cada rol tienen 21. Se basa en: el texto existe. Falta: las traducciones las escribió una IA, y nadie las ha revisado profesionalmente (WPM-T155).

## Comprobaciones por personas distintas del autor

- **0%** **Prueba de penetración independiente.** No se ha realizado ninguna prueba de penetración de una instancia desplegada (WPM-T201). Se basa en: la tarea está abierta. Falta: la prueba, y corregir lo que encuentre.
- **0%** **Auditoría de accesibilidad independiente.** No se ha hecho ninguna auditoría frente a WCAG 2.2 AA con personas que usan tecnología de apoyo (WPM-T202). Se basa en: la tarea está abierta. Falta: la auditoría, y corregir lo que encuentre.
- **25%** **Modelo de amenazas.** Existe un modelo de amenazas inicial ([threat-model.md](../../../spec/governance/threat-model.md), en inglés), escrito por el autor. Se basa en: el documento. Falta: que un equipo independiente lo rehaga para un despliegue real.

## La parte de quien lo despliega

- **20%** **Evaluación de impacto sobre la protección de datos.** Existe un paquete para adaptar: una evaluación de impacto, un flujo de datos, un registro de actividades de tratamiento y un calendario de conservación ([spec/governance](../../../spec/governance/dpia.md), en inglés). Se basa en: el paquete. Falta: no hay nadie que lo despliegue, así que ningún delegado de protección de datos ha firmado una evaluación, una base jurídica, un calendario de conservación ni un contrato de encargado del tratamiento (puerta WPM-G5).
- **15%** **Evaluaciones de impacto sobre la igualdad.** Existen una plantilla y un registro ([equality-impact](../../../spec/equality-impact/template.md), en inglés). Se basa en: la plantilla. Falta: que quien lo despliega redacte y apruebe una evaluación para su propio uso.
- **40%** **Alojamiento y operación.** Existen una configuración de producción, una imagen de contenedor y documentos para el despliegue y las actualizaciones. Se basa en: esos documentos y el simulacro de restauración. Falta: no se ha ejecutado nada en un servidor reforzado, y alguien que lo despliegue tiene que operar el servicio.
- **10%** **Soporte.** Hay un gestor de incidencias y una política de seguridad ([SECURITY.md](../../../SECURITY.md), en inglés). Se basa en: esos dos. Falta: no hay contrato de soporte, nivel de servicio, turno de guardia ni organización detrás de WPM.

## Lo que esta página no afirma

La página no registra ninguna decisión de pasar a producción. Esa decisión corresponde a quien lo despliega y se toma fuera de WPM. Las estimaciones cubren solo los elementos enumerados, y la lista propia de la evaluación de preparación de lo que no comprobó también se aplica aquí. Cuando se conecta un sistema clínico, una revisión de seguridad clínica es decisión de quien lo despliega.
