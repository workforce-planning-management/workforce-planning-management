# Tata kelola, risiko, kepatuhan

**Pertanyaan yang mungkin Anda ajukan:** siapa bertanggung jawab atas apa, apa yang bisa salah dan apa yang mencegahnya, dan bukti apa yang dapat saya tunjukkan kepada penilai? Halaman ini menghimpun bahan tata kelola, risiko, dan kepatuhan di satu tempat. Halaman [Direktur Risiko](../direktur-risiko/index.md) dan [Direktur Hukum](../direktur-hukum/index.md) mencakup bidang yang sama menurut peran.

## Tata kelola: siapa yang memutuskan, dan bagaimana dicatat

| Topik | Yang ada | Baca |
| --- | --- | --- |
| Bagaimana pekerjaan ditata | Spesifikasi lebih dulu: persyaratan, keputusan, tugas, kode, dan pengujian mendarat bersama; setiap tugas menyatakan apa yang **tidak** diverifikasi | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Tata kelola proyek | Tata kelola, pemelihara, proses RFC, dan berkas pemilik kode | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Keputusan tentang orang | Seseorang memutuskan; skor menasihati. Konversi, pengunduran diri, dan permohonan kerja fleksibel dicatat sebagai keputusan seseorang | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Akses dan peran | Peran dan atribut menentukan akses; tabel yang diklasifikasi memutuskan perlu-tahu pada setiap rute baca | [auth.md](../../../../spec/auth.md) |
| Penulisan dibantu AI | Pernyataan tentang bagaimana AI digunakan | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Risiko: apa yang bisa salah

| Topik | Yang ada | Baca |
| --- | --- | --- |
| Putusan | **Belum siap** untuk penerapan dengan jaminan tinggi; satu penghalang ditemukan lewat pengujian (pembacaan tidak mengikuti perlu-tahu) dan diperbaiki | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Ancaman | Model ancaman: aset, aktor, batas, kontrol sebagaimana dibangun, dan apa yang **tidak dinilai** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Bukti perusakan | Jejak audit hanya-tambah yang dirantai dengan hash, dengan titik akhir verifikasi; entri tidak memuat teks alasan atau nilai | [audit.md](../../../../spec/audit.md) |
| Mulai yang gagal dengan aman | Produksi menolak mulai dengan masuk akun dimatikan, tanpa kunci token, dengan koneksi basis data tak terenkripsi ke host lain, atau dengan Redis jarak jauh tak terenkripsi | [production-readiness.md](../../../../spec/production-readiness.md) |
| Pemulihan | Cadangan terenkripsi, latihan pemulihan yang dijalankan CI, buku besar penghapusan agar pemulihan tidak mengembalikan orang yang sudah dihapus | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Insiden | Buku panduan operasi berisi tindakan pertama; penyelenggara menambahkan kontak dan jalur eskalasi | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Bias dalam penskoran | Templat dampak kesetaraan dan register setiap skor, masing-masing dengan draf asesmen | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Risiko terbuka | Penyematan dan penandatanganan rantai pasok, bukti beban, pemeriksaan log untuk data pribadi, batas laju bersama, perangkat peran basis data dengan hak minimal, panduan pencabutan token | [readiness-assessment.md](../../../../spec/readiness-assessment.md), bagian 3 |

## Kepatuhan: apa yang dapat Anda tunjukkan kepada penilai

| Topik | Yang ada | Baca |
| --- | --- | --- |
| Penilaian mandiri atas kontrol | Daftar periksa yang setiap jawabannya terpenuhi, terpenuhi sebagian, tidak terpenuhi, atau dijawab oleh penyelenggara | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Asesmen dampak pelindungan data | Templat, diisi untuk demonstrasi, dengan bagian yang harus Anda lengkapi ditandai | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Catatan pemrosesan | Catatan yang dapat disesuaikan | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Alur data | Ke mana data pribadi bergerak | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Retensi | Jadwal per jenis catatan | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Hak subjek | Ekspor akses subjek dan penghapusan | [regulatory.md](../../../../spec/regulatory.md) |
| Data kategori khusus | Pemantauan kesetaraan dan persyaratan kesehatan di tempat kerja mati sampai dasar hukum dicatat, lalu dibatasi | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Aksesibilitas dan bahasa | Pernyataan aksesibilitas dan pernyataan bahasa resmi | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Lisensi dan dependensi | Berkas lisensi, kebijakan dependensi yang diperiksa oleh `cargo deny`, dan daftar bahan perangkat lunak pada setiap rilis | [production-readiness.md](../../../../spec/production-readiness.md) |

## Yang hanya dapat dilakukan oleh orang

Ini adalah gerbang, bukan kode, dan repositori tidak menandainya selesai: uji penetrasi independen, audit aksesibilitas independen, asesmen dampak pelindungan data Anda yang ditandatangani petugas Anda, latihan pemulihan di platform Anda, tinjauan profesional atas terjemahan, dan persetujuan Anda atas asesmen dampak kesetaraan.

## Batasan halaman ini

Tabel menunjuk ke dokumen. Apakah setiap dokumen benar untuk penerapan Anda adalah untuk Anda dan penilai Anda yang menilai. Pengujian penulis bukan tinjauan independen, dan tidak ada di sini yang merupakan nasihat hukum atau sertifikasi.

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
