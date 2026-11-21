# Aviso de privacidad

Este aviso dice qué guardan en su navegador el sitio web de Workforce Planning Management (WPM) y la aplicación WPM, por qué, durante cuánto tiempo y cómo eliminarlo. El sitio web es <https://workforce-planning-management.github.io/>; la aplicación es el software de demostración WPM, dondequiera que se ejecute. (Actualizado el 2026-11-17, WPM-R253.)

Ninguno de los dos tiene analítica, publicidad ni scripts, fuentes o imágenes de terceros. Nada de lo que cualquiera de los dos guarda en su navegador se usa para seguirle de un sitio a otro.

## El sitio web

El sitio web es un conjunto de páginas estáticas. No tiene inicio de sesión ni formularios. Lo que guarda está en su propio navegador:

| Qué | Dónde se guarda | Nombre | Qué se guarda | Por qué |
| --- | --- | --- | --- | --- |
| Su tema | El almacenamiento local de su navegador | `mxi.wpm.theme` | El nombre del tema, por ejemplo `dark` | Para que la página siguiente y su próxima visita se vean como usted eligió |
| Su tema | Una cookie solo para este sitio web | `wpm-theme` | El nombre del tema, por ejemplo `dark` | Para que el tema vuelva aunque su navegador conserve las cookies pero borre el almacenamiento local, o al revés |
| Su tamaño de texto | El almacenamiento local de su navegador | `mxi.wpm.text-size` | El nombre del tamaño, por ejemplo `large` | Para que la página siguiente y su próxima visita usen el tamaño de texto que usted eligió |
| Su idioma | No se guarda nada | | | El idioma forma parte de la dirección de cada página (por ejemplo `/cy-001/`), así que no hay nada que recordar |
| Dónde estaba en una página | El almacenamiento de sesión de su navegador, solo para esta pestaña | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Las posiciones de desplazamiento y el orden de las páginas que abrió en esta pestaña | Para que Atrás y Adelante le devuelvan a donde estaba. Los guarda el framework con el que está hecho el sitio web (SvelteKit) |

No se guarda ninguna elección hasta que usted la hace. Hasta que elija, el sitio web sigue el ajuste claro u oscuro de su navegador y su tamaño de texto normal, y no guarda tema ni tamaño de texto. Una elección se guarda cuando la hace en la barra de selección, y elegir de nuevo la sustituye. El almacenamiento de sesión se escribe a medida que pasa de una página a otra.

Otras partes del sitio web no guardan nada:

- **La búsqueda** se ejecuta en su navegador, sobre un índice que publica el sitio web, y no guarda nada. Sus palabras se ponen en la dirección de la página (`?q=`) para que se pueda compartir una página de resultados; cuando esa dirección se carga desde el servidor web (al recargar, o desde un enlace compartido), el servidor la recibe con la solicitud, como cualquier dirección.
- **Los controles deslizantes de preparación** del camino hacia la producción no guardan nada, y al recargar se restauran las estimaciones del autor.
- **Compartir** abre el sitio que usted elige (correo electrónico, LinkedIn, Reddit, Bluesky o Mastodon) solo cuando lo elige. Entonces se aplica el aviso de privacidad de ese sitio.

## La aplicación WPM

La aplicación guarda los mismos tipos de elecciones, y la cookie que mantiene su sesión iniciada:

| Qué | Dónde se guarda | Nombre | Qué se guarda | Por qué |
| --- | --- | --- | --- | --- |
| Su tema | El almacenamiento local de su navegador | `mxi.wpm.theme` | El nombre del tema | Para que cada página se vea como usted eligió |
| Su tamaño de texto | El almacenamiento local de su navegador | `mxi.wpm.text-size` | El nombre del tamaño | Para que cada página use el tamaño de texto que usted eligió |
| Su idioma | El almacenamiento local de su navegador | `mxi.wpm.locale` | El código de idioma de la última página que abrió, por ejemplo `cy-001` | Para que una dirección sin idioma se abra en el suyo |
| Su idioma | Una cookie solo para la aplicación | `wpm-locale` | El mismo código de idioma | Para que el servidor pueda enviar una dirección sin idioma al suyo |
| Su inicio de sesión | Una cookie solo para la aplicación, que los scripts de la página no pueden leer | `__Host-mxi_session` (`mxi_session` en el propio ordenador de un desarrollador) | Un número aleatorio que representa su sesión iniciada | Para que siga con la sesión iniciada de una página a otra; sin ella tendría que iniciar sesión en cada página |
| El idioma del editor de contenido | El almacenamiento local de su navegador, solo si abre el editor de contenido (`/admin`) | `sveltia-cms.prefs` | Los ajustes del editor, entre ellos su idioma | Para que el editor se abra en el idioma de la aplicación |
| Dónde estaba en una página | El almacenamiento de sesión de su navegador, solo para esta pestaña | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Las posiciones de desplazamiento y el orden de las páginas que abrió en esta pestaña | Para que Atrás y Adelante le devuelvan a donde estaba |

A diferencia del sitio web, la aplicación guarda el tema y el tamaño de texto en cuanto los muestra, antes de que usted elija, y guarda el idioma de cada página que abre. Las claves de antes del cambio de nombre del proyecto (`mxi.hcm.theme`, `mxi.hcm.locale`) se trasladan a los nombres de arriba y se eliminan.

La cookie de inicio de sesión es el único elemento que le representa a usted. El número aleatorio que contiene no significa nada por sí solo: la sesión iniciada a la que apunta la guarda el servidor de la aplicación, y cerrar la sesión la termina. Lo que la aplicación guarda sobre las personas de sus registros se trata en la penúltima sección, más abajo.

## Quién lo recibe

El almacenamiento local y el almacenamiento de sesión nunca salen de su navegador.

Una cookie se envía con cada solicitud que su navegador hace al sitio que la creó, y a ningún otro sitio (`SameSite=Lax`, y `Secure` sobre HTTPS):

- La cookie `wpm-theme` del sitio web va a GitHub Pages, que aloja el sitio web. Ningún programa del proyecto la lee.
- La cookie `wpm-locale` y la cookie de inicio de sesión de la aplicación van al servidor en el que se ejecuta la aplicación, que las lee para elegir su idioma y saber que tiene la sesión iniciada.

GitHub, que aloja el sitio web, recibe las solicitudes que hace su navegador, como cualquier servidor web. [La declaración de privacidad de GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) dice qué registra. La organización que ejecuta la aplicación decide qué registra su servidor.

## Cuánto tiempo se guarda

- La cookie `wpm-theme` del sitio web: 400 días naturales después de la última vez que eligió un tema, el máximo que un navegador guarda una cookie.
- La cookie `wpm-locale` de la aplicación: 365 días naturales después de la última vez que abrió una página.
- La cookie de inicio de sesión: hasta que cierre la sesión o cierre el navegador.
- El almacenamiento local: hasta que lo borre.
- El almacenamiento de sesión: hasta que cierre la pestaña.

## Cómo eliminarlo

Borre los datos del sitio en los ajustes de su navegador (cookies y datos de sitios), para el sitio web, la aplicación o ambos. El sitio web se comporta entonces como en una primera visita; la aplicación le pide que vuelva a iniciar sesión. Cerrar la sesión en la aplicación elimina la cookie de inicio de sesión.

## Por qué no hay aviso de consentimiento

Cada elemento se guarda para darle el servicio que usted pidió: el sitio web o la aplicación tal como los configuró, en su idioma y, en la aplicación, con la sesión iniciada. Ese es el tipo de almacenamiento que las normas del Reino Unido y de la UE sobre cookies (las Privacy and Electronic Communications Regulations británicas, y la Directiva sobre la privacidad y las comunicaciones electrónicas) consideran estrictamente necesario, y que no necesita consentimiento. Esta es la interpretación que hace el proyecto de esas normas, no asesoramiento jurídico.

## Los registros que guarda la aplicación

Este aviso trata de lo que se guarda en su navegador. La aplicación también guarda en su servidor registros sobre personas (empleados, candidatos y sus contactos). Quien ejecuta la aplicación es responsable de esos registros y redacta el aviso de privacidad para las personas que figuran en ellos ([la lista de comprobación de la evaluación](../../../spec/governance/assessment-checklist.md), en inglés, pregunta 3.10). El software de demostración solo contiene datos sintéticos.

## Preguntas

Pregunte en el repositorio del proyecto, <https://github.com/workforce-planning-management/workforce-planning-management>, o escriba a la persona encargada del mantenimiento que figura allí.
