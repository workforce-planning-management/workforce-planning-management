# Untuk Direktur Keamanan

**Pertanyaan yang mungkin Anda ajukan:** bagaimana ini dilindungi, bagaimana kami tahu jika diserang atau disalahgunakan, dan apa yang belum diuji siapa pun?

## Apa yang Anda dapatkan

| Perhatian | Yang ada | Baca |
| --- | --- | --- |
| Ancaman | Model ancaman: aset, aktor, batas kepercayaan, kontrol sebagaimana dibangun, dan apa yang **tidak dinilai** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Autentikasi | Masuk akun aktif secara bawaan; produksi menolak mulai jika dimatikan atau tanpa sumber kunci token; masuk lewat direktori perusahaan (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Otorisasi | Kebijakan memutuskan hanya berdasarkan tindakan; tabel yang diklasifikasi memutuskan perlu-tahu pada setiap rute baca, dan pengujian pemindaian sumber gagal pada rute yang tidak diklasifikasi | [auth.md](../../../../spec/auth.md) |
| Audit | Jejak hanya-tambah yang dirantai dengan hash, dengan titik akhir verifikasi; entri tidak membawa teks alasan dan tidak membawa nilai | [audit.md](../../../../spec/audit.md) |
| Transportasi dan peramban | Header keamanan pada setiap respons API, daftar izin CORS yang eksplisit yang menghentikan mulai bila tidak diatur, `no-store` pada `/api` | WPM-R73 di [production-readiness.md](../../../../spec/production-readiness.md) |
| Penyalahgunaan | Batas laju per pemanggil, lebih ketat untuk penghapusan, impor, ekspor, dan rute token | WPM-R74, berkas yang sama |
| Konfigurasi tak aman | Saat mulai, menolak koneksi basis data atau Redis tak terenkripsi ke host lain dan asal CORS yang hilang | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Data saat diam | Cadangan terenkripsi; cache bersama hanya menyimpan pencarian non-personal dan gagal secara terbuka | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Rantai pasok | Pemeriksaan lisensi dan larangan dengan `cargo deny`, crate bersama yang disertakan, daftar bahan perangkat lunak pada setiap rilis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Insiden | Buku panduan operasi berisi tindakan pertama; Anda menambahkan kontak dan kewajiban pemberitahuan | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Penilaian mandiri | Daftar periksa kontrol: terpenuhi, terpenuhi sebagian, tidak terpenuhi, dijawab oleh penyelenggara | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Kesenjangan yang diketahui, sebagaimana dicatat

- Belum ada uji penetrasi independen. Pengujian ditulis oleh penulis.
- Actions disematkan dengan tag, bukan dengan digest, dan citra tidak ditandatangani.
- Batas laju berlaku per instans.
- Aplikasi memakai peran basis data apa pun yang diberikan kepadanya; perangkat peran dengan hak minimal tidak disediakan.
- Pencabutan token dan usia maksimum token adalah urusan penyedia identitas Anda.
- Jalur pengelola konten memuat skrip dari CDN publik tanpa versi yang disematkan atau hash integritas.
- Log belum diperiksa untuk data pribadi, dan jejak audit tidak diekspor ke sistem pemantauan.
- Redis terkelola (klaster, sentinel, pengguna kendali akses) belum dicoba.

## Yang hanya dapat dilakukan oleh orang

Memesan uji penetrasi independen, meninjau model ancaman untuk lingkungan Anda, menjalankan latihan pemulihan di platform Anda, dan mengatur perlindungan cabang di platform hosting. Lihat [penilaian kesiapan](../../../../spec/readiness-assessment.md), yang putusannya **belum siap** untuk penerapan dengan jaminan tinggi.

Lihat juga halaman [Direktur Risiko](../direktur-risiko/index.md) dan [Direktur Informasi](../direktur-informasi/index.md).

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
