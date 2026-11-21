# Демонстрационное руководство

Это руководство объясняет, как запустить WPM на локальном компьютере, загрузить вымышленные данные и пройти управляемый маршрут по каждой странице.

> **Только вымышленные данные.** Во время прохождения вход отключён. Не подключайте приложение к сервису, в котором есть настоящие данные.

## Требования

- [Rust](https://www.rust-lang.org/tools/install), стабильная версия
- [Node.js](https://nodejs.org/) и [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/) с пустой базой данных для прохождения
- Python 3

[Руководство по установке](../../../INSTALL.md) указывает необходимые версии и настройки базы данных.

## Шаг 1. Получите код

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Шаг 2. Запустите сервис

Сервис читает `DATABASE_URL`. Значение по умолчанию — `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; задайте переменную, если у вашей базы другое имя, другой пользователь или другой порт. Подготовьте пустую базу и запустите API. Вход отключается только для этого экземпляра, а ограничение числа запросов повышается на время прохождения.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Шаг 3. Загрузите вымышленные данные

Во втором терминале, из корня репозитория:

```sh
workforce-demo-data http://localhost:1241
```

## Шаг 4. Запустите приложение

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Шаг 5. Пройдите маршрут

Откройте адрес, который показывает приложение, и войдите как **example@example.com**. Письма не отправляются. После входа открывается `/demo` — маршрут по всем страницам. Баннер на каждой странице напоминает, что данные вымышленные, и предлагает ссылки «Назад» и «Далее».

## Что читать дальше

- [Записки по ролям](../for/index.md)
- [Сравнение продукт за продуктом](../../../docs/compare/index.md)
- [Спецификация](../../../spec/index.md)
- [Как оставить отзыв](../../../RFC.md)
