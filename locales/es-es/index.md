# Gestión de la planificación de la plantilla

**Una plataforma de recursos humanos de código abierto que cubre el ciclo de vida del empleado, de la contratación a la jubilación.**

Workforce Planning Management (WPM) reúne en una sola aplicación la selección de personal, el tiempo y las ausencias, el desarrollo de los empleados, la planificación de la plantilla y la nómina. Es software de demostración, pensado para evaluarlo y como diseño de referencia.

Idioma: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Software de demostración, aún no listo para uso en producción.** WPM usa solo datos sintéticos y sus cálculos legales son ilustrativos. No introduzca datos personales reales. [El camino a producción](path-to-production/index.md) explica qué necesitaría el uso en producción y cuán preparada está cada parte. Véase también [la posición regulatoria](../../spec/regulatory.md).

[Ejecutar la demostración](start/index.md) [Informes por rol](for/index.md) [Código fuente en GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Ventajas

Los datos de recursos humanos suelen estar repartidos en herramientas separadas, como bandejas de candidatos, hojas de cálculo de vacaciones y tableros de turnos. Cada traspaso entre ellas añade trabajo manual y riesgo de error. WPM está diseñado para reducir esos traspasos.

- **Un único sistema.** La incorporación, las ausencias, la formación y los registros de pago están en una sola aplicación, lo que reduce la reintroducción de datos.
- **Minimización de datos.** Los datos que no deben conservarse no tienen campo, y los que no deben revelarse no tienen ruta en la API. Por ejemplo, no hay campo de diagnóstico y las respuestas de la encuesta de pulso no tienen autor.
- **Reglas de información.** Un valor desconocido no se informa como cero, los agregados no identifican a personas y las estimaciones indican en qué se basan.
- **Localización.** La aplicación está disponible en 17 locales y los informes por rol en 21, incluidos idiomas de derecha a izquierda. Las reglas de cada país se guardan como datos. Las traducciones las produjo una IA y ningún hablante nativo las ha revisado.
- **Trazabilidad.** El código fuente es abierto y cada comportamiento se vincula a un requisito, una decisión de diseño y una tarea numerados.
- **Despliegue independiente.** La pila es una API JSON en Rust, un cliente SvelteKit y PostgreSQL. Se compila desde un clon limpio y las integraciones con aplicaciones hermanas son opcionales.

## Funciones

- **Selección y fichas.** Vacantes, flujo de candidatos, incorporación, fichas de trabajadores y organigrama.
- **Tiempo y turnos.** Tiempo y ausencias, turnos con comprobación de jornada, el turno de guardia con intercambios y recordatorios, y un directorio de personal.
- **Desarrollo.** Evaluaciones, comentarios de 360°, formación, análisis de brechas de competencias, recomendaciones de tiempo de formación y formación obligatoria.
- **Planificación e informes.** Planificación estratégica de la plantilla, métricas y análisis, y un panel para la dirección general.
- **Altas y bajas.** Listas de tareas con fechas y un traspaso del último día con registro de auditoría.
- **Comunicación.** Anuncios con destinatarios, enlaces y recuentos de lectura.
- **Derechos y bienestar de los empleados.** Mensajes de bienestar, una encuesta de pulso anónima, adaptaciones, acceso a los propios datos, supresión y conservación.
- **Nómina.** Nómina y remuneración, con cálculos legales ilustrativos.

## Comparación

La tabla compara WPM con tres categorías de sistemas de recursos humanos. Es una orientación general, no una prueba comparativa. Véase también la [comparación por producto](../../docs/compare/index.md).

| | WPM | Suites todo en uno (Workday, SAP SuccessFactors, BambooHR) | RR. HH. de código abierto (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Herramientas puntuales (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Alcance** | Ciclo completo, de la contratación a la jubilación | Alcance similar en una plataforma | Las mismas funciones básicas | Una sola función |
| **Licencia** | Código abierto | Comercial | Código abierto | Comercial |
| **Principal punto fuerte** | Minimización de datos, diseño trazable, análisis de brechas de competencias y de tiempo de formación | Una plataforma integrada y con soporte | Más integraciones, localizaciones y despliegues | Profundidad en una función |
| **Uso típico** | Evaluación, diseño de referencia o desarrollo posterior | Organizaciones que necesitan un sistema con soporte | Organizaciones que quieren código abierto en producción | Organizaciones que necesitan profundidad en una función y pueden integrar el resto |

## A quién se dirige

- **Directivos.** Los [informes por rol](for/index.md) exponen qué ofrece WPM a cada rol directivo y cómo leer lo que muestra.
- **Equipos de recursos humanos.** El [tutorial de la demostración](start/index.md) recorre cada página con datos ficticios.
- **Desarrolladores y arquitectos.** Véanse la [especificación](../../spec/index.md) y la [guía de instalación](../../INSTALL.md).
- **Evaluadores.** La [evaluación de preparación](../../spec/readiness-assessment.md) enumera las carencias entre WPM y el uso en producción.

## Cómo obtener ayuda

- **Informe de un error o haga una pregunta** en el [gestor de incidencias](https://github.com/workforce-planning-management/workforce-planning-management/issues). Indique la página, lo que hizo, lo que esperaba y lo que ocurrió. No incluya datos personales reales.
- **Informe de un problema de seguridad** por correo electrónico, como indica la [política de seguridad](../../SECURITY.md), y no en una incidencia pública.
- **Aprenda una tarea en la aplicación.** El enlace **Ayuda** del encabezado de la aplicación abre la página de ayuda y las guías para empleados, responsables y recursos humanos. Cada mensaje de error también enlaza con ella.

## Próximos pasos

1. **Lea el informe de su rol.** Cada informe describe qué ofrece WPM y qué queda abierto.
2. **Ejecute la demostración.** El recorrido funciona en un equipo local con datos ficticios.
3. **Envíe comentarios.** Véanse [cómo comentar](../../RFC.md) y [cómo contribuir](../../CONTRIBUTING.md).

[Ejecutar la demostración](start/index.md) [Documentación](../../index.md)
