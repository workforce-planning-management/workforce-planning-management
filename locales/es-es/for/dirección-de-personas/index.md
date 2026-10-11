# Para la dirección de personas (CPO)

**La pregunta que probablemente se hace:** ¿respalda esto todo el ciclo de vida del empleado, y puedo usarlo sin guardar sobre las personas más de lo que debo?

## Qué obtiene

| Etapa | Qué guarda WPM | Lea |
| --- | --- | --- |
| Contratación | Solicitudes de puesto, un flujo de candidaturas, una bolsa de candidatos, listas de comprobación de incorporación | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| El registro | Un registro de empleo por trabajador, el organigrama, líneas de dependencia, grupos, traslados entre organizaciones | [hr-core.md](../../../../spec/hr-core.md) |
| Autoservicio | La persona ve y actualiza sus propios datos, solicita trabajo flexible, da el preaviso y ve su tiempo libre | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Vida laboral | Permisos, tiempo, turnos, beneficios, avisos de bienestar, ajustes razonables, evaluaciones ergonómicas, el pulso anónimo | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Desarrollo | Evaluaciones, evaluaciones de 360 grados, cursos a través del registro de cursos, valoraciones, mentorías, planes de desarrollo, sucesión, primeros empleos | [talent-development.md](../../../../spec/talent-development.md) |
| Retribución | Ejecuciones de nómina, nóminas, comparativas salariales, consulta de una escala salarial nacional, niveles de puesto | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Personal no fijo | Fechas de fin por base, prórrogas, evaluaciones de estado, planes de conversión decididos por una persona, financiación junto al fin del contrato | [engagements.md](../../../../spec/engagements.md) |
| Salida | Dimisión con preaviso, un traspaso el último día, conservación por tipo de registro, supresión | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Cómo protege a las personas

- **Necesidad de conocer en las lecturas.** Un responsable ve a su equipo, una persona se ve a sí misma, RR. HH. ve lo que necesita. Un desconocido con sesión iniciada no ve nada de un compañero ([auth.md](../../../../spec/auth.md)).
- **No se guarda ningún diagnóstico.** La baja por enfermedad registra un tipo, no un motivo.
- **Las funciones de categoría especial están desactivadas por defecto.** La monitorización de la igualdad y los requisitos de salud en el trabajo necesitan una base jurídica registrada, son voluntarios o los guarda el rol de salud laboral, y solo muestran grupos por encima de un umbral. Nunca son una entrada de ninguna puntuación ni decisión sobre una persona.
- **Las puntuaciones asesoran.** La preparación para la sucesión, las brechas de habilidades y las previsiones sugieren; una persona decide, y decisiones como convertir a un contratista se registran como decisión de una persona, nunca como resultado automático.
- **La persona interesada puede consultar y suprimir.** La exportación de acceso y la supresión cubren los registros anteriores, con un libro para que una restauración no devuelva a una persona suprimida.

## Límites

- Aún sin construir: una interfaz para planes de conversión y financiación, la tendencia contingente y la página «Mis datos» del trabajador. Consulte [tasks.md](../../../../spec/tasks.md) para ver el estado actual.
- Las traducciones se hacen con ayuda de máquina; una revisión profesional es una puerta abierta.
- Los convenios colectivos y los órganos reconocidos son solo investigación y propuesta ([worker-unions.md](../../../../spec/worker-unions.md)).
- El impuesto de nómina es un marcador de posición.

Vea también la [dirección jurídica](../dirección-jurídica/index.md) y la [dirección general](../dirección-general/index.md).

## Qué tan actual es esto

Escrito el 2026-10-11 a partir de la especificación, no ejecutando el software para esta página. Las cifras que cambian (pruebas, migraciones, tareas hechas) no se copian aquí; léalas en [implementation-status.md](../../../../spec/implementation-status.md), que genera un script. WPM es software de demostración con datos sintéticos: esta página dice lo que hace y lo que no hace, y no afirma que sea adecuado para su organización. Nada de lo aquí escrito es asesoramiento jurídico.

> Esta traducción la ha escrito una IA y ningún hablante nativo la ha revisado. Si difiere de la versión en inglés (`locales/en-gb-oxendict/for/`), prevalece la versión en inglés.
