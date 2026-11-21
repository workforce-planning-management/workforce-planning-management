# Untuk Direktur Informasi (CIO)

**Pertanyaan yang mungkin Anda ajukan:** bagaimana ini cocok dengan lingkungan kami, siapa yang dapat masuk, di mana data berada, dan bagaimana kami menjalankan serta memulihkannya?

## Apa yang Anda dapatkan

| Perhatian | Yang ada | Baca |
| --- | --- | --- |
| Bentuk | API JSON dan klien peramban terpisah; PostgreSQL menyimpan data | [architecture.md](../../../../spec/architecture.md) |
| Identitas | WPM tidak menyimpan identitas. Orang, pekerja, dan organisasi adalah rujukan ke layanan lain, diselesaikan hanya berdasarkan nama dan di-cache | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Masuk akun | Aktif secara bawaan. Masuk lewat direktori perusahaan (OIDC, kompatibel Entra, langsung atau lewat Keycloak); peran dan atribut menentukan akses | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Penerapan | Build kontainer dan berkas compose untuk demonstrasi; untuk produksi, proksi yang mengakhiri TLS, pengaturan wajib, dan pemeriksaan saat mulai yang menolak penyiapan tak aman | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Cache bersama | Redis opsional hanya untuk pencarian non-personal; setiap panggilan gagal secara terbuka | `REDIS_URL` di panduan penerapan (WPM-D78) |
| Cadangan dan pemulihan | Cadangan terenkripsi, skrip pemulihan, dan latihan pemulihan yang dijalankan CI | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Peningkatan versi | Migrasi dan panduan peningkatan | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Saat ada yang salah | Buku panduan operasi berisi tindakan pertama | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Pergerakan data | Dokumen alur data | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Hal yang perlu diketahui sebelum merencanakan

- Antarmuka pengguna **tidak dikontainerkan**; Anda memilih adaptor untuk platform Anda.
- Batas laju berlaku per instans. Penyimpanan bersama Anda yang menyediakan.
- Aplikasi memakai peran basis data yang disebut `DATABASE_URL`; perangkat peran dengan hak minimal belum disediakan.
- Pencabutan token dan usia maksimum token adalah urusan penyedia identitas Anda.
- Redis terkelola (klaster, sentinel, pengguna kendali akses) belum dicoba.
- Tidak ada pengukuran beban, dan beberapa tampilan membaca satu organisasi ke memori.
- Build citra dan berkas compose telah ditulis tetapi sebaiknya dicoba lebih dulu di lingkungan Anda.

Lihat [readiness-assessment.md](../../../../spec/readiness-assessment.md) untuk daftar kesenjangan lengkap dan putusannya, dan halaman [Direktur Teknologi](../direktur-teknologi/index.md) untuk sudut pandang rekayasa.

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
