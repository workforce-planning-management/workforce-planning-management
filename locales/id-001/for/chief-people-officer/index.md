# Untuk Direktur SDM (CPO)

**Pertanyaan yang mungkin Anda ajukan:** apakah ini mendukung seluruh siklus hidup karyawan, dan dapatkah saya memakainya tanpa menyimpan lebih banyak tentang orang daripada yang semestinya?

## Apa yang Anda dapatkan

| Tahap | Apa yang disimpan WPM | Baca |
| --- | --- | --- |
| Perekrutan | Permintaan posisi, alur pelamar, kolam kandidat, daftar periksa orientasi | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| Catatan | Satu catatan kepegawaian per pekerja, bagan organisasi, garis pelaporan, kelompok, perpindahan antarorganisasi | [hr-core.md](../../../../spec/hr-core.md) |
| Layanan mandiri | Pekerja melihat dan memperbarui datanya sendiri, mengajukan kerja fleksibel, menyampaikan pemberitahuan, dan melihat waktu liburnya | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Kehidupan kerja | Cuti, waktu, shift, tunjangan, pengingat kesejahteraan, penyesuaian yang wajar, asesmen ergonomi, denyut anonim | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Pengembangan | Penilaian kinerja, penilaian 360, kursus melalui registri kursus, asesmen, mentoring, rencana pengembangan, suksesi, karier awal | [talent-development.md](../../../../spec/talent-development.md) |
| Penggajian | Proses penggajian, slip gaji, tolok ukur gaji, pencarian skala gaji nasional, tingkat jabatan | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Staf non-tetap | Tanggal berakhir menurut dasar, perpanjangan, asesmen status, rencana konversi yang diputuskan oleh seseorang, pendanaan di samping akhir kontrak | [engagements.md](../../../../spec/engagements.md) |
| Keluar | Pengunduran diri dengan masa pemberitahuan, serah terima pada hari terakhir, retensi per jenis catatan, penghapusan | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Bagaimana ia melindungi orang

- **Perlu-tahu saat membaca.** Manajer melihat bawahannya, pekerja melihat dirinya sendiri, SDM melihat yang SDM perlukan. Orang asing yang sudah masuk tidak melihat apa pun tentang rekan kerja ([auth.md](../../../../spec/auth.md)).
- **Tidak ada diagnosis yang disimpan.** Cuti sakit mencatat jenis, bukan alasan.
- **Fitur kategori khusus mati secara bawaan.** Pemantauan kesetaraan dan persyaratan kesehatan di tempat kerja memerlukan dasar hukum yang tercatat, bersifat sukarela atau dipegang oleh peran kesehatan kerja, dan hanya menampilkan kelompok di atas batas minimum. Keduanya tidak pernah menjadi masukan bagi skor atau keputusan tentang seseorang.
- **Skor menasihati.** Kesiapan suksesi, kesenjangan keterampilan, dan prakiraan hanya menyarankan; seseorang memutuskan, dan keputusan seperti mengonversi kontraktor dicatat sebagai keputusan seseorang, tidak pernah sebagai hasil otomatis.
- **Subjek dapat melihat dan menghapus.** Ekspor akses subjek dan penghapusan mencakup catatan di atas, dengan buku besar agar pemulihan tidak mengembalikan orang yang sudah dihapus.

## Batasan

- Belum dibangun: antarmuka pengguna untuk rencana konversi dan pendanaan, tren kontingen, dan halaman "data saya" milik pekerja. Lihat [tasks.md](../../../../spec/tasks.md) untuk keadaan terkini.
- Terjemahan dibantu mesin; tinjauan profesional adalah gerbang yang masih terbuka.
- Perjanjian kerja bersama dan badan yang diakui hanyalah riset dan usulan ([worker-unions.md](../../../../spec/worker-unions.md)).
- Pajak penggajian hanyalah rintisan.

Lihat juga [Direktur Hukum](../chief-legal-officer/index.md) dan [Direktur Utama](../chief-executive-officer/index.md).

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
