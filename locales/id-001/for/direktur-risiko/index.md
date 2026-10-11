# Untuk Direktur Risiko (CRO)

**Pertanyaan yang mungkin Anda ajukan:** apa yang bisa salah pada sistem ini, apa yang mencegahnya, dan apa yang belum diperiksa siapa pun?

## Titik awal yang jujur

[Penilaian kesiapan](../../../../spec/readiness-assessment.md) tanggal 2026-10-11 memberi putusan **belum siap** untuk penerapan dengan jaminan tinggi. Penilaian itu menemukan satu penghalang dengan menguji layanan yang berjalan: setiap orang yang sudah masuk dapat membaca cuti sakit rekan kerja beserta alasannya yang berupa teks bebas. Itu sudah diperbaiki dan diuji. Gerbang yang hanya dapat ditutup oleh orang masih terbuka.

## Apa yang Anda dapatkan

| Perhatian | Yang ada | Baca |
| --- | --- | --- |
| Ancaman di setiap batas | Model ancaman: aset, aktor, batas, kontrol sebagaimana dibangun, dan apa yang **tidak dinilai** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Pembacaan tanpa wewenang | Pengaman perlu-tahu yang mengklasifikasi setiap rute baca; pengujian gagal pada rute yang tidak diklasifikasi | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Perusakan dan akuntabilitas | Jejak audit hanya-tambah yang dirantai dengan hash, dengan titik akhir verifikasi; entri tidak memuat teks alasan atau nilai | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Data sensitif | Tidak ada diagnosis yang disimpan; fitur kategori khusus mati secara bawaan; batas minimum untuk kelompok kecil | [regulatory.md](../../../../spec/regulatory.md) |
| Jalur tak aman | Produksi menolak mulai dengan masuk akun dimatikan, tanpa sumber kunci token, dengan koneksi basis data tak terenkripsi ke host lain, atau dengan Redis jarak jauh tak terenkripsi | [production-readiness.md](../../../../spec/production-readiness.md) |
| Kehilangan data | Cadangan terenkripsi, latihan pemulihan yang dijalankan di CI, buku besar penghapusan | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Bias dalam penskoran | Templat dampak kesetaraan, dan register setiap skor dengan draf asesmen | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Penilaian mandiri atas kontrol | Daftar periksa yang setiap jawabannya terpenuhi, terpenuhi sebagian, tidak terpenuhi, atau dijawab oleh penyelenggara | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Risiko terbuka, sebagaimana dicatat

Penyematan dan penandatanganan rantai pasok, bukti beban, pemeriksaan log untuk data pribadi, batas laju bersama lintas instans, perangkat peran basis data dengan hak minimal, dan panduan pencabutan token adalah butir terbuka dalam penilaian kesiapan, masing-masing dengan satu tugas. Pengujian ditulis oleh penulis; tidak ada yang merupakan tinjauan independen.

## Yang hanya dapat dilakukan oleh orang

Uji penetrasi independen, audit aksesibilitas independen, asesmen dampak penyelenggara yang ditandatangani, latihan pemulihan di platform Anda, dan persetujuan atas asesmen dampak kesetaraan. Repositori menyatakan ini sebagai gerbang dan tidak menandainya selesai.

Lihat juga [Direktur Informasi](../direktur-informasi/index.md) dan [Direktur Hukum](../direktur-hukum/index.md).

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
