# Dechrau arni: taith dywys o WPM

Croeso! Yn y tiwtorial hwn byddwch yn rhedeg WPM ar eich peiriant eich hun, yn ei lenwi â phobl ffug, ac yn cerdded trwy bob tudalen gyda chanllaw wrth eich ochr. Nid oes dim yn gadael eich cyfrifiadur.

> ⚠️ **Data ffug yn unig.** Mae'r daith yn rhedeg gyda mewngofnodi wedi'i ddiffodd, felly peidiwch byth â'i chysylltu â gwasanaeth sy'n dal data go iawn.

## Beth sydd ei angen arnoch

- [Rust](https://www.rust-lang.org/tools/install), sefydlog
- [Node.js](https://nodejs.org/) a [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/) gyda chronfa ddata wag ar gyfer y daith
- Python 3

Mae gan [y canllaw gosod](../../../INSTALL.md) (yn Saesneg) yr union fersiynau a sut i gysylltu'r gwasanaeth â'ch cronfa ddata.

## Cam 1: Cael y cod

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Cam 2: Cychwyn y gwasanaeth

Yng nghyfeiriadur y gwasanaeth, paratowch y gronfa ddata wag a chychwyn yr API. Mae mewngofnodi wedi'i ddiffodd ar gyfer yr enghraifft hon yn unig, ac mae'r terfyn cyfradd uwch yn cadw i fyny â thaith gyflym.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WPM_REQUIRE_AUTH=0 WPM_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Cam 3: Ychwanegu'r bobl ffug

Mewn ail derfynell, o frig yr ystorfa:

```sh
python3 scripts/demo-data.py http://localhost:5150
```

## Cam 4: Cychwyn yr ap

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WPM_DEMO_API_URL=http://localhost:5150 pnpm dev
```

## Cam 5: Mynd ar y daith

Agorwch y cyfeiriad y mae'r ap yn ei argraffu, a mewngofnodwch fel **example@example.com**. Ni anfonir e-bost. Byddwch yn cyrraedd `/demo`, taith o bob tudalen, ac mae baner ar bob tudalen yn eich atgoffa bod y data'n ffug ac yn cynnig Blaenorol a Nesaf.

## Ble nesaf

- Gwelwch beth mae WPM yn ei gynnig i [eich rôl chi](../for/index.md).
- Darllenwch [sut mae WPM yn cymharu](../../../COMPARISONS.md) (yn Saesneg).
- Archwiliwch [y fanyleb](../../../spec/index.md) (yn Saesneg), yr unig ffynhonnell wirionedd.
- Dywedwch wrthym beth yw eich barn: [sut i wneud sylw](../../../RFC.md) (yn Saesneg).
