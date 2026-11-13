# Jalan menuju produksi

WPM adalah perangkat lunak demonstrasi. Halaman ini menyatakan apa arti penggunaan produksi bagi WPM, dan mendaftar apa yang masih memisahkan WPM dari penggunaan itu. Setiap butir disertai perkiraan seberapa siap butir itu, dari 0 sampai 100 persen.

## Apa arti produksi

Penggunaan produksi berarti sebuah organisasi mengandalkan WPM untuk pekerjaan sungguhan:

- catatannya adalah catatan orang sungguhan: karyawan, pelamar, dan kontak darurat mereka;
- gaji yang dihitung oleh WPM benar-benar dibayarkan, dan hukum di tempat kerja berlaku atasnya;
- orang masuk akun setiap hari, dan sebagian yang mereka baca bersifat sensitif, seperti ketidakhadiran karena sakit;
- organisasi bertanggung jawab kepada regulator, auditor, dan orang-orang dalam catatan atas cara WPM dijalankan.

Halaman ini mengukur WPM terhadap kasus tersulit yang menjadi dasar penilaiannya: organisasi medis atau pemerintahan yang besar ([penilaian kesiapan](../../../spec/readiness-assessment.md), dalam bahasa Inggris). Organisasi yang lebih kecil mungkin memerlukan lebih sedikit untuk beberapa butir, tetapi memerlukan jenis bukti yang sama.

Produksi memerlukan tiga jenis bukti, dan perangkat lunak saja hanya menyediakan yang pertama:

1. **Perangkat lunak** yang melakukan pekerjaannya dengan aman, dengan pengujian yang menunjukkannya.
2. **Penyelenggara** yang menghosting dan mengoperasikannya: cadangan, pemantauan, dukungan, dan penanganan insiden.
3. **Orang selain penulis** yang telah memeriksanya: penguji penetrasi, auditor aksesibilitas, penerjemah, dan petugas pelindungan data milik penyelenggara.

## Cara membaca perkiraan

- Perkiraan adalah penilaian penulis pada 2026-11-13, dibuat dari spesifikasi dan pengujian. Perkiraan bukan pengukuran dan bukan sertifikasi.
- 0 persen berarti belum ada apa pun. 100 persen berarti sudah dibangun, diuji, dan diperiksa oleh pihak independen pada penerapan sungguhan. Tidak ada butir yang mencapai 100 persen.
- Setiap butir menyatakan dasar perkiraannya dan apa yang masih tersisa.
- Perkiraan tidak dirata-ratakan. Satu butir dengan perkiraan rendah sudah cukup untuk menahan WPM dari produksi.

## Perangkat lunak

- **75%** **Masuk akun dan akses perlu-tahu.** Masuk akun aktif secara bawaan, dan layanan menolak dijalankan di produksi tanpanya (WPM-R72). Pembacaan dibatasi pada orang yang perlu tahu (WPM-T188), dan token memiliki usia maksimum (WPM-T200). Dasar: pengujian penegakan yang berjalan di CI. Tersisa: belum ada daftar pencabutan token, dan belum ada uji penetrasi independen yang dilakukan.
- **70%** **Jejak audit yang lengkap dan tidak diubah.** Jejak audit bersifat hanya-tambah dan dirantai dengan hash, dan dapat diekspor ke sistem pemantauan (WPM-T189, WPM-T197). Dasar: pengujian yang sengaja dirusak untuk membuktikan bahwa pengujian itu gagal. Tersisa: ujung rantai yang terpotong tidak terdeteksi, dan pemilik basis data masih dapat mematikan pemicunya.
- **70%** **Pelindungan data sejak perancangan.** Orang diidentifikasi dengan rujukan, tidak ada kolom demografis, dan tersedia akses subjek, penghapusan, dan retensi untuk setiap jenis catatan. Dasar: pengujian, dan bagian 1 penilaian kesiapan. Tersisa: asesmen dan dasar hukum milik penyelenggara sendiri (lihat di bawah).
- **75%** **Koneksi dan cadangan terenkripsi.** Di produksi, layanan menolak koneksi basis data tanpa enkripsi dan menghentikan permintaan yang macet setelah batas waktu (WPM-T191), dan cadangan dienkripsi (WPM-T192). Dasar: pengujian atas keduanya. Tersisa: pengelolaan kunci dan sertifikat menjadi tanggung jawab penyelenggara.
- **10%** **Perhitungan menurut undang-undang.** Perhitungan menurut undang-undang bersifat ilustratif, dan pajak penggajian masih berupa stub ([posisi regulasi](../../../spec/regulatory.md), dalam bahasa Inggris). Dasar: spesifikasi, yang menyatakan demikian. Tersisa: aturan setiap tempat kerja, ditulis dan diperiksa oleh orang yang berkualifikasi. Jangan membayar siapa pun dari WPM.
- **50%** **Skala.** Pada satu laptop, halaman direktori dan tampilan kapasitas tidak gagal pada 50.000 pekerja, dan tampilan perencanaan menolak lebih dari 10.000 (WPM-T196, WPM-T230). Dasar: satu kali pengukuran ([BENCHMARKS.md](../../../BENCHMARKS.md), dalam bahasa Inggris). Tersisa: tampilan kapasitas masih tumbuh sebanding dengan jumlah tenaga kerja, dan belum ada yang diukur pada perangkat keras penyelenggara.
- **65%** **Rantai pasok.** Setiap action CI disematkan ke sebuah commit dan riwayatnya dipindai untuk mencari rahasia. Citra kontainer ditandatangani dan membawa asal-usulnya, dan hal ini dijalankan untuk rilis v0.1.2 (WPM-T194, WPM-T195). Dasar: proses rilis tersebut. Tersisa: paket citra bersifat privat, sehingga belum ada pihak di luar proyek yang dapat memverifikasinya.
- **60%** **Pemantauan.** Setiap respons membawa id permintaan, dan baris log tidak memuat string kueri atau pengenal pribadi (WPM-T197). Dasar: pengujian atas log permintaan. Tersisa: dasbor dan peringatan menjadi tanggung jawab penyelenggara, dan batas laju hanya berlaku untuk satu instans (WPM-T198).
- **60%** **Pencadangan dan pemulihan.** Cadangan terenkripsi, latihan pemulihan yang berjalan di CI, dan runbook. Dasar: latihan tersebut. Tersisa: latihan pemulihan di platform milik penyelenggara sendiri dalam 90 hari kalender terakhir (gerbang WPM-G4).
- **40%** **Pemeriksaan aksesibilitas.** Pemeriksaan otomatis terhadap WCAG 2.2 AA dijalankan pada aplikasi dan pada situs ini, dalam tema terang dan gelap. Dasar: pengujian Playwright dan axe. Tersisa: audit independen (lihat di bawah); belum ada pembaca layar yang digunakan.
- **20%** **Terjemahan.** Aplikasi memiliki 17 lokal dan ringkasan peran memiliki 21. Dasar: teksnya ada. Tersisa: terjemahan ditulis oleh AI, dan belum ada yang meninjaunya secara profesional (WPM-T155).

## Pemeriksaan oleh orang selain penulis

- **0%** **Uji penetrasi independen.** Belum ada uji penetrasi atas instans yang diterapkan (WPM-T201). Dasar: tugasnya masih terbuka. Tersisa: pengujian itu, dan perbaikan atas temuannya.
- **0%** **Audit aksesibilitas independen.** Belum ada audit terhadap WCAG 2.2 AA bersama orang yang memakai teknologi bantu (WPM-T202). Dasar: tugasnya masih terbuka. Tersisa: audit itu, dan perbaikan atas temuannya.
- **25%** **Model ancaman.** Model ancaman awal sudah ada ([threat-model.md](../../../spec/governance/threat-model.md), dalam bahasa Inggris), ditulis oleh penulis. Dasar: dokumen tersebut. Tersisa: tim independen menyusunnya ulang untuk penerapan sungguhan.

## Bagian penyelenggara

- **20%** **Asesmen dampak pelindungan data.** Tersedia paket untuk disesuaikan: asesmen dampak, alur data, catatan pemrosesan, dan jadwal retensi ([spec/governance](../../../spec/governance/dpia.md), dalam bahasa Inggris). Dasar: paket tersebut. Tersisa: belum ada penyelenggara, sehingga belum ada petugas pelindungan data yang menyetujui asesmen, dasar hukum, jadwal retensi, atau perjanjian dengan pemroses (gerbang WPM-G5).
- **15%** **Asesmen dampak kesetaraan.** Tersedia templat dan register ([equality-impact](../../../spec/equality-impact/template.md), dalam bahasa Inggris). Dasar: templat tersebut. Tersisa: penyelenggara menulis dan menyetujui asesmen untuk penggunaannya sendiri.
- **40%** **Hosting dan pengoperasian.** Tersedia konfigurasi produksi, citra kontainer, serta dokumen untuk penerapan dan peningkatan versi. Dasar: dokumen-dokumen tersebut dan latihan pemulihan. Tersisa: belum ada yang dijalankan pada host yang diperkeras, dan penyelenggara harus mengoperasikan layanan.
- **10%** **Dukungan.** Tersedia pelacak isu dan kebijakan keamanan ([SECURITY.md](../../../SECURITY.md), dalam bahasa Inggris). Dasar: keduanya. Tersisa: tidak ada kontrak dukungan, tingkat layanan, jadwal siaga, atau organisasi di belakang WPM.

## Apa yang tidak diklaim halaman ini

Halaman ini tidak mencatat keputusan apa pun untuk mulai beroperasi. Keputusan itu milik penyelenggara dan dibuat di luar WPM. Perkiraan hanya mencakup butir yang didaftar, dan daftar milik penilaian kesiapan sendiri tentang apa yang tidak diperiksanya juga berlaku di sini. Jika sistem klinis disambungkan, tinjauan keselamatan klinis adalah keputusan penyelenggara.
