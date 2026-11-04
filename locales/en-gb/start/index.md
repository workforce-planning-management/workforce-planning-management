# Demo tutorial

This tutorial describes how to run WPM on a local machine, load fictitious data and follow the guided tour of each page.

> **Fictitious data only.** Sign-in is turned off for the tour. Do not connect it to a service that holds real data.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install), stable
- [Node.js](https://nodejs.org/) and [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), with an empty database for the tour
- Python 3

The [install guide](../../../INSTALL.md) gives the required versions and the database settings.

## Step 1: Get the code

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Step 2: Start the service

Prepare the empty database and start the API. Sign-in is turned off for this instance only, and the rate limit is raised for the tour.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WPM_REQUIRE_AUTH=0 WPM_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Step 3: Load the fictitious data

In a second terminal, from the repository root:

```sh
python3 scripts/demo-data.py http://localhost:5150
```

## Step 4: Start the application

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WPM_DEMO_API_URL=http://localhost:5150 pnpm dev
```

## Step 5: Follow the tour

Open the address shown by the application and sign in as **example@example.com**. No email is sent. The sign-in opens `/demo`, a tour of every page. A banner on each page states that the data is fictitious and provides Previous and Next links.

## Further reading

- [Role briefs](../for/index.md)
- [Comparison by product](../../../docs/compare/index.md)
- [Specification](../../../spec/index.md)
- [How to comment](../../../RFC.md)
