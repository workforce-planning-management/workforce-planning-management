# Panduan demo

Panduan ini menunjukkan cara menjalankan WPM di komputer Anda sendiri, memuat data fiktif, dan mengikuti rute terpandu melalui setiap halaman.

> **Hanya data fiktif.** Selama panduan berjalan, proses masuk dinonaktifkan. Jangan menghubungkan aplikasi ke layanan yang berisi data nyata.

## Yang Anda perlukan

- [Rust](https://www.rust-lang.org/tools/install), versi stabil
- [Node.js](https://nodejs.org/) dan [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), dengan basis data kosong untuk panduan ini
- Python 3

[Panduan instalasi](../../../INSTALL.md) menyebutkan versi yang diperlukan dan penyiapan basis data.

## Langkah 1. Ambil kodenya

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Langkah 2. Jalankan layanan

Layanan membaca `DATABASE_URL`. Nilai bawaannya adalah `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; atur variabel itu jika nama basis data, pengguna, atau port Anda berbeda. Siapkan basis data kosong lalu jalankan API. Proses masuk hanya dinonaktifkan untuk instans ini, dan batas permintaan dinaikkan selama panduan.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Langkah 3. Muat data fiktif

Di terminal kedua, dari folder utama repositori:

```sh
workforce-demo-data http://localhost:1241
```

## Langkah 4. Jalankan aplikasi

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Langkah 5. Ikuti rutenya

Buka alamat yang ditampilkan aplikasi dan masuk sebagai **example@example.com**. Tidak ada email yang dikirim. Setelah masuk, `/demo` terbuka, yaitu rute terpandu melalui semua halaman. Spanduk di setiap halaman mengingatkan bahwa datanya fiktif, dan menyediakan tautan "Sebelumnya" dan "Berikutnya".

## Apa yang dibaca selanjutnya

- [Dokumentasi untuk setiap peran](../for/index.md)
- [Perbandingan per produk](../../../docs/compare/index.md)
- [Spesifikasi](../../../spec/index.md)
- [Cara memberi komentar](../../../RFC.md)
