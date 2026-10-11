# Para la dirección de sistemas de información (CIO)

**La pregunta que probablemente se hace:** ¿cómo encaja en nuestro entorno, quién puede iniciar sesión, dónde viven los datos y cómo lo ejecutamos y recuperamos?

## Qué obtiene

| Asunto | Qué existe | Lea |
| --- | --- | --- |
| Forma | Una API JSON y un cliente de navegador separado; PostgreSQL guarda los datos | [architecture.md](../../../../spec/architecture.md) |
| Identidad | WPM no guarda identidades. Una persona, un trabajador y una organización son referencias a otros servicios, resueltas solo por nombre y guardadas en caché | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Inicio de sesión | Activado por defecto. Inicio de sesión con directorio corporativo (OIDC, compatible con Entra, directo o mediante Keycloak); los roles y atributos deciden el acceso | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Despliegue | Una compilación de contenedor y un archivo compose para una demostración; para producción, un proxy que termina TLS, ajustes obligatorios y comprobaciones de arranque que rechazan una configuración insegura | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Caché compartida | Redis opcional solo para consultas no personales; cada llamada falla en abierto | `REDIS_URL` en la guía de despliegue (WPM-D78) |
| Copia de seguridad y recuperación | Copias cifradas, un script de restauración y un simulacro de restauración que ejecuta la CI | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Actualización | Migraciones y una guía de actualización | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Cuando algo falla | Un manual operativo con las primeras acciones | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Movimiento de datos | Un documento de flujo de datos | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Qué conviene saber antes de planificar

- La interfaz **no está en contenedor**; usted elige un adaptador para su plataforma.
- Los límites de frecuencia son por instancia. Un almacén compartido corre de su cuenta.
- La aplicación usa el rol de base de datos que nombre `DATABASE_URL`; aún no se ofrece un conjunto de roles de mínimo privilegio.
- La revocación de tokens y su edad máxima son de su proveedor de identidad.
- No se ha probado Redis gestionado (clúster, centinela, usuarios de control de acceso).
- No hay medición de carga, y algunas vistas leen una organización en memoria.
- La compilación de la imagen y el archivo compose se escribieron, pero conviene probarlos primero en su entorno.

Consulte [readiness-assessment.md](../../../../spec/readiness-assessment.md) para la lista completa de brechas y la conclusión, y la página de la [dirección de tecnología](../chief-technology-officer/index.md) para la visión de ingeniería.

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
