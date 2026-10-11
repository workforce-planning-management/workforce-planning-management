# Untuk Direktur Teknologi (CTO)

**Pertanyaan yang mungkin Anda ajukan:** apakah dibangun dengan baik, dapatkah tim saya mengubahnya dengan aman, dan bagaimana saya tahu?

## Apa yang Anda dapatkan

| Perhatian | Yang ada | Baca |
| --- | --- | --- |
| Tumpukan teknologi | Rust dengan Loco (Axum, SeaORM) dan PostgreSQL 18 untuk API; SvelteKit 2 dengan Svelte 5 dan TypeScript untuk klien | [architecture.md](../../../../spec/architecture.md) |
| Aturan rancangan | Inti murni: aturan bebas basis data dan bebas jam, diuji tuntas, baru kemudian dihubungkan ke pengendali | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Spesifikasi lebih dulu | Satu perubahan adalah persyaratan, keputusan, tugas, kode, dan pengujian bersama; setiap entri tugas menyatakan apa yang **tidak** diverifikasi | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Pengujian | Uji unit atas aturan, uji permintaan berbasis basis data, biner penegakan yang dijalankan dengan masuk akun aktif, uji unit front-end, dan uji peramban. Pemeriksaan mutasi: merusak kode dengan sengaja dan memastikan suatu pengujian gagal | [testing.md](../../../../spec/testing.md) |
| Hitungan | Dihasilkan dari berkas, tidak pernah disalin dengan tangan | [implementation-status.md](../../../../spec/implementation-status.md) |
| Rantai pasok | `cargo deny` untuk lisensi dan larangan, crate bersama yang disertakan, daftar bahan perangkat lunak pada setiap rilis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Kontrak API | OpenAPI, dan pengujian yang gagal jika rute terdaftar tidak terdokumentasi | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Kendali akses | Kebijakan memutuskan hanya berdasarkan tindakan; penangan dan tabel rute yang diklasifikasi memutuskan perlu-tahu, dan pemindaian sumber gagal pada rute baca yang tidak diklasifikasi | [auth.md](../../../../spec/auth.md) |
| Kinerja | [BENCHMARKS.md](../../../../BENCHMARKS.md) mencatat apa yang diukur; tidak ada uji beban | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Utang rekayasa yang diketahui

- Beberapa tampilan memuat seluruh organisasi ke memori.
- Kunci baris pada rencana konversi dinalar, belum dibuktikan oleh pengujian.
- CI, rilis, dan pemindaian dinamis sudah ditulis tetapi belum berjalan di platform hosting.
- Actions disematkan dengan tag, bukan dengan digest, dan citra tidak ditandatangani.
- Beberapa uji Redis dengan kata sandi dan TLS memerlukan variabel dan otoritas sertifikat, sehingga CI tidak menjalankannya.
- Pohon `spec/` besar; berkas status yang dihasilkan adalah cara untuk melihat apa yang dinyatakan dibandingkan dengan apa yang lulus.

## Cara berkontribusi

Baca [AGENTS.md](../../../../AGENTS.md) lebih dulu: lima aturan, lalu peta menurut tugas.

Lihat juga halaman [Direktur Informasi](../direktur-informasi/index.md) untuk sudut pandang operasi.

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
