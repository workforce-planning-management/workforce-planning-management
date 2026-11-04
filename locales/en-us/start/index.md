# Get started: a guided tour of WPM

Welcome! In this tutorial you run WPM on your own machine, fill it with fictitious people, and walk through every page with a guide at your side. Nothing leaves your computer.

> ⚠️ **Fictitious data only.** The tour runs with sign-in switched off, so never point it at a service that holds real data.

## What you need

- [Rust](https://www.rust-lang.org/tools/install), stable
- [Node.js](https://nodejs.org/) and [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/) with an empty database for the tour
- Python 3

The [install guide](../../../INSTALL.md) has the exact versions and how to point the service at your database.

## Step 1: Get the code

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Step 2: Start the service

In the service directory, prepare the empty database and start the API. Sign-in is off for this instance only, and the higher rate limit keeps up with a quick tour.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WPM_REQUIRE_AUTH=0 WPM_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Step 3: Add the fictitious people

In a second terminal, from the top of the repository:

```sh
python3 scripts/demo-data.py http://localhost:5150
```

## Step 4: Start the app

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WPM_DEMO_API_URL=http://localhost:5150 pnpm dev
```

## Step 5: Take the tour

Open the address the app prints, and sign in as **example@example.com**. No email is sent. You land on `/demo`, a tour of every page, and a banner on each page reminds you that the data is fictitious and offers Previous and Next.

## Where to go next

- See what WPM offers [your role](../for/index.md).
- Read [how WPM compares](../../../COMPARISONS.md).
- Explore [the specification](../../../spec/index.md), the single source of truth.
- Tell us what you think: [how to comment](../../../RFC.md).
