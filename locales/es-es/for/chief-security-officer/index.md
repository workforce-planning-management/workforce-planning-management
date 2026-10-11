# Para la dirección de seguridad

**La pregunta que probablemente se hace:** ¿cómo está protegido esto, cómo sabríamos si lo atacaran o se hiciera un mal uso, y qué no ha probado nadie?

## Qué obtiene

| Asunto | Qué existe | Lea |
| --- | --- | --- |
| Amenazas | Un modelo de amenazas: activos, actores, fronteras de confianza, el control tal como está construido y lo que **no se evaluó** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Autenticación | Inicio de sesión activado por defecto; producción se niega a arrancar si está desactivado o sin fuente de claves de token; inicio de sesión con directorio corporativo (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Autorización | La política decide solo por acción; una tabla clasificada decide la necesidad de conocer en cada ruta de lectura, y una prueba de análisis del código fuente falla ante una ruta sin clasificar | [auth.md](../../../../spec/auth.md) |
| Auditoría | Registro de solo añadir, encadenado por hash, con un punto de verificación; las entradas no llevan texto de motivo ni valor | [audit.md](../../../../spec/audit.md) |
| Transporte y navegador | Cabeceras de seguridad en cada respuesta de la API, una lista de CORS explícita que impide el arranque si falta, `no-store` en `/api` | WPM-R73 en [production-readiness.md](../../../../spec/production-readiness.md) |
| Abuso | Límites de frecuencia por llamante, más estrictos para supresión, importación, exportación y rutas de token | WPM-R74, el mismo archivo |
| Configuración insegura | El arranque rechaza una conexión de base de datos o de Redis en claro a otro host y un origen CORS ausente | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Datos en reposo | Copias cifradas; la caché compartida guarda solo consultas no personales y falla en abierto | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Cadena de suministro | Comprobaciones de licencias y prohibiciones con `cargo deny`, crates compartidos incluidos, una lista de materiales de software en cada publicación | [production-readiness.md](../../../../spec/production-readiness.md) |
| Incidentes | Un manual operativo con las primeras acciones; usted añade contactos y obligaciones de notificación | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Autoevaluación | Una lista de comprobación de controles: cumplida, cumplida en parte, no cumplida, a responder por quien despliega | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Brechas conocidas, tal como constan

- No se ha hecho una prueba de penetración independiente. Las pruebas las escribió el autor.
- Las Actions se fijan por etiqueta, no por huella, y las imágenes no están firmadas.
- Los límites de frecuencia son por instancia.
- La aplicación usa el rol de base de datos que se le dé; no se ofrece un conjunto de roles de mínimo privilegio.
- La revocación de tokens y su edad máxima son de su proveedor de identidad.
- La ruta del gestor de contenidos carga un script de un CDN público sin versión fijada ni hash de integridad.
- No se han comprobado los registros en busca de datos personales, y el registro de auditoría no se exporta a un sistema de monitorización.
- No se ha probado Redis gestionado (clúster, centinela, usuarios de control de acceso).

## Lo que solo pueden hacer las personas

Encargar la prueba de penetración independiente, revisar el modelo de amenazas para su entorno, ejecutar un simulacro de restauración en su plataforma y configurar la protección de ramas en la plataforma de alojamiento. Vea la [evaluación de preparación](../../../../spec/readiness-assessment.md), cuya conclusión es **no está listo** para un despliegue de alta garantía.

Vea también las páginas de la [dirección de riesgos](../chief-risk-officer/index.md) y de la [dirección de sistemas de información](../chief-information-officer/index.md).

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
