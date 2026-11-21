# Tutorial de la demostración

Este tutorial describe cómo ejecutar WPM en una computadora local, cargar datos ficticios y seguir el recorrido guiado por cada página.

> **Solo datos ficticios.** El inicio de sesión está desactivado durante el recorrido. No lo conecte a un servicio que contenga datos reales.

## Requisitos previos

- [Rust](https://www.rust-lang.org/tools/install), versión estable
- [Node.js](https://nodejs.org/) y [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), con una base de datos vacía para el recorrido
- Python 3

La [guía de instalación](../../../INSTALL.md) indica las versiones necesarias y los ajustes de la base de datos.

## Paso 1: Obtener el código

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Paso 2: Iniciar el servicio

El servicio lee `DATABASE_URL`. Su valor por defecto es `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; defina la variable si su base de datos tiene otro nombre, usuario o puerto. Prepare la base de datos vacía e inicie la API. El inicio de sesión se desactiva solo en esta instancia y el límite de peticiones se eleva para el recorrido.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Paso 3: Cargar los datos ficticios

En una segunda terminal, desde la raíz del repositorio:

```sh
workforce-demo-data http://localhost:1241
```

## Paso 4: Iniciar la aplicación

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Paso 5: Seguir el recorrido

Abra la dirección que muestra la aplicación e inicie sesión como **example@example.com**. No se envía ningún correo. El inicio de sesión abre `/demo`, un recorrido por todas las páginas. Un aviso en cada página indica que los datos son ficticios y ofrece los enlaces Anterior y Siguiente.

## Lecturas adicionales

- [Informes por rol](../for/index.md)
- [Comparación por producto](../../../docs/compare/index.md)
- [Especificación](../../../spec/index.md)
- [Cómo comentar](../../../RFC.md)
