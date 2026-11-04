# Tiwtorial arddangos

Mae'r tiwtorial hwn yn disgrifio sut i redeg WPM ar beiriant lleol, llwytho data ffug a dilyn y daith dywys o bob tudalen.

> **Data ffug yn unig.** Mae mewngofnodi wedi'i ddiffodd ar gyfer y daith. Peidiwch â'i chysylltu â gwasanaeth sy'n dal data go iawn.

## Gofynion

- [Rust](https://www.rust-lang.org/tools/install), fersiwn sefydlog
- [Node.js](https://nodejs.org/) a [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), gyda chronfa ddata wag ar gyfer y daith
- Python 3

Mae'r [canllaw gosod](../../../INSTALL.md) (Saesneg) yn rhoi'r fersiynau gofynnol a gosodiadau'r gronfa ddata.

## Cam 1: Cael y cod

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Cam 2: Cychwyn y gwasanaeth

Mae'r gwasanaeth yn darllen `DATABASE_URL`. Ei ragosodiad yw `postgres://loco:loco@localhost:5432/workforce_planning_management_development`; gosodwch y newidyn os oes gan eich cronfa ddata enw, defnyddiwr neu borth arall. Paratowch y gronfa ddata wag a chychwyn yr API. Mae mewngofnodi wedi'i ddiffodd ar gyfer yr enghraifft hon yn unig, a chodir y terfyn cyfradd ar gyfer y daith.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WPM_REQUIRE_AUTH=0 WPM_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Cam 3: Llwytho'r data ffug

Mewn ail derfynell, o wraidd yr ystorfa:

```sh
python3 scripts/demo-data.py http://localhost:5150
```

## Cam 4: Cychwyn y rhaglen

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WPM_DEMO_API_URL=http://localhost:5150 pnpm dev
```

## Cam 5: Dilyn y daith

Agorwch y cyfeiriad a ddangosir gan y rhaglen a mewngofnodwch fel **example@example.com**. Ni anfonir e-bost. Mae'r mewngofnodi'n agor `/demo`, taith o bob tudalen. Mae baner ar bob tudalen yn nodi bod y data'n ffug ac yn darparu dolenni Blaenorol a Nesaf.

## Darllen pellach

- [Dogfennau rôl](../for/index.md)
- [Cymhariaeth fesul cynnyrch](../../../docs/compare/index.md) (Saesneg)
- [Manyleb](../../../spec/index.md) (Saesneg)
- [Sut i wneud sylw](../../../RFC.md) (Saesneg)
