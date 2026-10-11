# Untuk Direktur Operasional (COO)

**Pertanyaan yang mungkin Anda ajukan:** apakah kami punya orang yang tepat, di tempat yang tepat, pada waktu yang tepat, dan apa yang rusak jika seseorang tidak hadir?

## Apa yang Anda dapatkan

| Anda ingin tahu | Tempat WPM menjawab | Baca |
| --- | --- | --- |
| Apakah tim dapat menutup ketidakhadiran | Cadangan per orang, jadwal siaga dengan pertukaran dan pengingat, dan tampilan penggantian yang menyatakan dengan jelas saat tidak ada yang tersedia | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Waktu, kehadiran, dan cuti | Catatan waktu, permohonan cuti dengan jalur persetujuan, akrual, dan shift | [workforce-management.md](../../../../spec/workforce-management.md) |
| Apakah batas jam kerja dipatuhi | Pagar pengaman jam kerja yang bersifat menyarankan; memberi peringatan dan tidak memblokir | WPM-R27, berkas yang sama |
| Kapasitas per keterampilan, bulan demi bulan | Kolam keterampilan, pasokan dan kebutuhan program, komitmen mitra, kolam kendala, dan pemeriksaan awal yang berupa saran, dengan keputusan mulai yang dicatat | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Apa yang akan dibutuhkan dan di mana kesenjangannya | Rencana tenaga kerja, prakiraan, dan analisis kesenjangan per departemen dan per peran | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Kontrak mana yang segera berakhir | Tanggal berakhir menurut dasar penugasan, perpanjangan dengan riwayat, pengingat sebelum berakhir, daftar penugasan tanpa tanggal berakhir, dan rencana konversi | [engagements.md](../../../../spec/engagements.md) |
| Di mana tenaga kontingen menjadi bagian besar | Bagian FTE yang berstatus kontraktor dan yang berstatus waktu tertentu, terpisah, dan pendanaan yang berakhir bulan demi bulan | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Karyawan masuk dan keluar | Daftar periksa bertanggal dan serah terima pada hari terakhir | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Kesenjangan keterampilan dan waktu pelatihan | Analisis kesenjangan dengan bukti yang mendasari suatu rekomendasi | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Cara membaca tampilan kapasitas

Kapasitas direncanakan menurut kolam keterampilan, bukan menurut orang. Setiap bulan-kolam menghasilkan satu dari empat hasil, dan keterampilan yang tidak dinyatakan adalah *tidak diketahui*, bukan kekurangan. Pemeriksaan awal adalah saran, dan seseorang mencatat keputusan untuk memulai. Penjadwalan rantai kritis dan jalur kritis tidak ada di sini: keduanya milik project-portfolio-management.

## Batasan

- Bagian kontingen belum punya tren: perlu cuplikan jumlah karyawan yang dipisah menurut dasar (WPM-T139, belum dibangun).
- Dalam tampilan kontingen, kolam adalah departemen sampai rencana menetapkan kolam.
- Penjadwalan shift dan perhitungan pajak penggajian bukan pengganti sistem penjadwalan atau penggajian Anda.
- Beberapa tampilan memuat catatan suatu organisasi ke memori; belum ada angka beban yang diukur ([readiness-assessment.md](../../../../spec/readiness-assessment.md), kesenjangan 8).

Lihat juga [Direktur SDM](../chief-people-officer/index.md) dan [Direktur Utama](../chief-executive-officer/index.md).

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
