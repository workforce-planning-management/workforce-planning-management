# Untuk Direktur Strategi

**Pertanyaan yang mungkin Anda ajukan:** apakah tenaga kerja kami dibentuk untuk melayani strategi, apa yang akan kami butuhkan, dan apa yang sebaiknya kami lakukan tentang kesenjangannya?

## Apa yang Anda dapatkan

| Anda ingin tahu | Tempat WPM menjawab | Baca |
| --- | --- | --- |
| Apa yang dituntut suatu peran | Profil peran dengan keterampilan yang diperlukan dan tingkat kemahiran minimum, sehingga kesenjangan diukur terhadap peran, bukan terhadap orang | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Apa yang akan kami butuhkan | Rencana tenaga kerja sebagai dunia draf, beberapa berdampingan, dengan asumsi yang dinyatakan dan kebutuhan menurut departemen, peran, dan periode | WPM-R36, berkas yang sama |
| Di mana kesenjangannya | Proyeksi pasokan terhadap kebutuhan yang direncanakan: kesenjangan jumlah dan kesenjangan kompetensi, dengan tuas yang didaftar sebagai saran beserta buktinya | WPM-R37 |
| Apakah melayani strategi | Sasaran yang dikaitkan dengan kebutuhan: sasaran tanpa sumber daya, kebutuhan yang tak selaras, peran kritis tanpa cadangan | WPM-R38 |
| Apakah kami dapat menyelesaikan suatu program | Kolam keterampilan, kebutuhan program, komitmen mitra, kolam kendala, pemeriksaan awal yang berupa saran | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Seberapa besar kami bergantung pada staf non-tetap | Bagian FTE yang berstatus kontraktor dan yang berstatus waktu tertentu, terpisah, dan berapa banyak pendanaan yang berakhir pada setiap dari dua belas bulan ke depan | `GET /api/workforce-intelligence/contingent` (WPM-T109, sebagian) |
| Apa yang kami rencanakan saat kontrak berakhir | Rencana konversi: keputusan seseorang untuk mengonversi, memperpanjang, atau mengakhiri, dengan pendanaan di samping akhir kontrak | [engagements.md](../../../../spec/engagements.md) |
| Apa lagi yang direncanakan | Peta jalan dan daftar tugas yang terbuka | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Cara membaca prakiraan

- Ini adalah aturan yang dinyatakan yang diterapkan pada masukan yang dinyatakan, bukan model, dan setiap hasil menyebut asumsinya.
- Dengan riwayat jumlah karyawan yang terlalu sedikit, ia menyatakan `insufficient_history` dan tidak mengekstrapolasi.
- Rencana hanya menyimpan jumlah hipotetis. Rencana tidak pernah membaca atau mengubah catatan seseorang.
- Tuas adalah saran. Perencana yang memilih, dan perangkat lunak tidak memutuskan apa pun.

## Batasan

- Tuas "ubah tenaga kontingen menjadi tetap" dalam rencana telah dispesifikasikan (WPM-R63) dan belum ada dalam kode.
- Bagian kontingen belum punya tren: perlu cuplikan yang dipisah menurut dasar (WPM-T139, belum dibangun).
- Penjadwalan rantai kritis dan jalur kritis milik project-portfolio-management, bukan di sini.
- Belum ada antarmuka pengguna untuk pendanaan atau rencana konversi.

Lihat juga halaman [Direktur Utama](../direktur-utama/index.md) dan [Direktur Operasional](../direktur-operasional/index.md).

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
