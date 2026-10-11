# Para la dirección de estrategia

**La pregunta que probablemente se hace:** ¿está nuestra plantilla configurada para servir a la estrategia, qué necesitaremos y qué debemos hacer con la brecha?

## Qué obtiene

| Quiere saber | Dónde responde WPM | Lea |
| --- | --- | --- |
| Qué exige un rol | Perfiles de rol con las habilidades requeridas y una competencia mínima, de modo que una brecha se mide frente a un rol y no frente a una persona | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Qué necesitaremos | Planes de plantilla como mundos en borrador, varios en paralelo, con supuestos declarados y demanda por departamento, rol y periodo | WPM-R36, el mismo archivo |
| Dónde está la brecha | Oferta proyectada frente a demanda planificada: una brecha de personas y una de competencias, con palancas listadas como sugerencias y su evidencia | WPM-R37 |
| Si sirve a la estrategia | Objetivos vinculados a la demanda: objetivos sin recursos, demanda sin alinear, roles críticos sin banquillo | WPM-R38 |
| Si podemos entregar un programa | Bolsas de habilidades, demanda de programas, compromisos de socios, la bolsa restrictiva, una comprobación de inicio que es una sugerencia | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Cuánto dependemos de personal no fijo | La parte de las FTE que son contratistas y la que es temporal, por separado, y cuánta financiación termina en cada uno de los próximos doce meses | `GET /api/workforce-intelligence/contingent` (WPM-T109, parcial) |
| Qué pensamos hacer cuando terminen los contratos | Planes de conversión: la decisión de una persona de convertir, prorrogar o terminar, con la financiación junto al fin del contrato | [engagements.md](../../../../spec/engagements.md) |
| Qué más está planificado | La hoja de ruta y la lista de tareas abiertas | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Cómo leer una previsión

- Es una regla declarada aplicada a entradas declaradas, no un modelo, y cada resultado nombra sus supuestos.
- Con poco historial de plantilla dice `insufficient_history` en lugar de extrapolar.
- Un plan solo guarda personas hipotéticas. Nunca lee ni cambia el registro de una persona.
- Las palancas son sugerencias. Elige quien planifica, y el software no decide nada.

## Límites

- La palanca «convertir personal contingente en fijo» en los planes está especificada (WPM-R63) y aún no está en el código.
- La parte contingente aún no tiene tendencia: necesita instantáneas separadas por base (WPM-T139, no construido).
- La planificación de cadena crítica y de ruta crítica pertenece a project-portfolio-management, no a esta aplicación.
- Aún no hay interfaz de usuario para la financiación ni para los planes de conversión.

Vea también las páginas de la [dirección general](../chief-executive-officer/index.md) y de la [dirección de operaciones](../chief-operating-officer/index.md).

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
