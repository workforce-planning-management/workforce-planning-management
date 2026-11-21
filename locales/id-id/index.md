# Manajemen Perencanaan Tenaga Kerja

**Platform SDM sumber terbuka yang mencakup siklus hidup karyawan dari perekrutan hingga pensiun.**

Workforce Planning Management (WPM) menyatukan rekrutmen, waktu dan cuti, pengembangan karyawan, perencanaan tenaga kerja, dan penggajian dalam satu aplikasi. Ini adalah perangkat lunak demonstrasi, yang dimaksudkan untuk evaluasi dan sebagai rancangan acuan.

Bahasa: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Perangkat lunak demonstrasi, belum siap untuk penggunaan produksi.** WPM hanya memakai data sintetis, dan perhitungan hukumnya bersifat ilustratif. Jangan memasukkan data pribadi yang nyata. [Jalan menuju produksi](path-to-production/index.md) menjelaskan apa yang dibutuhkan penggunaan produksi dan seberapa siap setiap bagiannya. Lihat juga [posisi regulasi](../../spec/regulatory.md).

[Jalankan demo](start/index.md) [Dokumentasi peran](for/index.md) [Kode sumber di GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Manfaat

Data SDM sering tersebar di berbagai alat terpisah, seperti kotak masuk pelamar, lembar kerja cuti, dan papan jadwal. Setiap perpindahan di antaranya menambah pekerjaan manual dan risiko kesalahan. WPM dirancang untuk mengurangi perpindahan itu.

- **Satu sistem.** Catatan orientasi, cuti, pelatihan, dan gaji disimpan dalam satu aplikasi, sehingga tidak perlu memasukkan ulang data.
- **Minimalisasi data.** Data yang tidak semestinya disimpan tidak memiliki kolom, dan data yang tidak semestinya diungkapkan tidak memiliki rute API. Misalnya, tidak ada kolom diagnosis, dan jawaban survei denyut tidak memiliki penulis.
- **Aturan pelaporan.** Nilai yang tidak diketahui tidak dilaporkan sebagai nol, agregat tidak mengidentifikasi individu, dan perkiraan menyebutkan dasarnya.
- **Lokalisasi.** Aplikasi tersedia dalam 17 lokal dan dokumentasi peran dalam 21, termasuk bahasa yang ditulis dari kanan ke kiri. Aturan khusus setiap negara disimpan sebagai data. Terjemahan dibuat oleh AI dan belum ditinjau oleh penutur asli.
- **Keterlacakan.** Kode sumbernya terbuka, dan setiap perilaku dapat ditelusuri ke persyaratan, keputusan rancangan, dan tugas yang bernomor.
- **Penerapan mandiri.** Tumpukan teknologinya adalah API JSON Rust, klien SvelteKit, dan PostgreSQL. Ia dibangun dari klon baru, dan integrasi dengan aplikasi saudara bersifat opsional.

## Fitur

- **Rekrutmen dan catatan.** Permintaan posisi, alur pelamar, orientasi, catatan pekerja, dan bagan organisasi.
- **Waktu dan penjadwalan.** Waktu dan cuti, shift dengan pemeriksaan jam kerja, jadwal siaga dengan pertukaran dan pengingat, serta direktori staf.
- **Pengembangan.** Penilaian kinerja, umpan balik 360°, pembelajaran, analisis kesenjangan keterampilan, rekomendasi waktu pelatihan, dan pembelajaran wajib.
- **Perencanaan dan pelaporan.** Perencanaan tenaga kerja strategis, metrik dan wawasan, serta dasbor Direktur Utama.
- **Karyawan masuk dan keluar.** Daftar periksa bertanggal dan serah terima pada hari terakhir dengan jejak audit.
- **Komunikasi.** Pengumuman dengan sasaran, tautan, dan jumlah pembaca.
- **Hak dan kesejahteraan karyawan.** Pengingat kesejahteraan, survei denyut anonim, penyesuaian, akses subjek, penghapusan, dan retensi.
- **Gaji.** Penggajian dan kompensasi, dengan perhitungan hukum yang ilustratif.

## Perbandingan

Tabel ini membandingkan WPM dengan tiga kategori sistem SDM. Ini adalah orientasi umum, bukan tolok ukur. Lihat juga [perbandingan per produk](../../docs/compare/index.md).

| | WPM | Suite serba ada (Workday, SAP SuccessFactors, BambooHR) | SDM sumber terbuka (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Alat khusus (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Cakupan** | Siklus hidup penuh, dari perekrutan hingga pensiun | Cakupan serupa dalam satu platform | Fungsi inti yang sama | Satu fungsi |
| **Lisensi** | Sumber terbuka | Komersial | Sumber terbuka | Komersial |
| **Kekuatan utama** | Minimalisasi data, rancangan yang dapat ditelusuri, analisis kesenjangan keterampilan dan waktu pelatihan | Satu platform terpadu dengan dukungan | Lebih banyak integrasi, lokalisasi, dan penerapan | Kedalaman dalam satu fungsi |
| **Kecocokan umum** | Evaluasi, rancangan acuan, atau pengembangan lebih lanjut | Organisasi yang membutuhkan sistem dengan dukungan | Organisasi yang menginginkan sumber terbuka di produksi | Organisasi yang membutuhkan kedalaman dalam satu fungsi dan dapat mengintegrasikan sisanya |

## Untuk siapa

- **Eksekutif.** [Dokumentasi peran](for/index.md) menjelaskan apa yang disediakan WPM bagi setiap peran kepemimpinan dan cara membaca keluarannya.
- **Tim SDM.** [Panduan demo](start/index.md) membahas setiap halaman dengan data fiktif.
- **Pengembang dan arsitek.** Lihat [spesifikasi](../../spec/index.md) dan [panduan instalasi](../../INSTALL.md).
- **Penilai.** [Penilaian kesiapan](../../spec/readiness-assessment.md) mencantumkan kesenjangan antara WPM dan penggunaan produksi.

## Cara mendapatkan bantuan

- **Laporkan bug atau ajukan pertanyaan** di [pelacak isu](https://github.com/workforce-planning-management/workforce-planning-management/issues). Sebutkan halamannya, apa yang Anda lakukan, apa yang Anda harapkan, dan apa yang terjadi. Jangan menyertakan data pribadi yang nyata.
- **Laporkan masalah keamanan** melalui email, seperti dijelaskan dalam [kebijakan keamanan](../../SECURITY.md), bukan dalam isu publik.
- **Pelajari tugas di dalam aplikasi.** Tautan **Bantuan** di header aplikasi membuka halaman bantuan dan panduan langkah demi langkah untuk karyawan, manajer, dan SDM. Setiap pesan kesalahan juga menautkan ke sana.

## Langkah berikutnya

1. **Baca dokumentasi untuk peran Anda.** Setiap dokumentasi menjelaskan apa yang disediakan WPM dan apa yang masih terbuka.
2. **Jalankan demo.** Tur ini berjalan di mesin lokal dengan data fiktif.
3. **Kirim masukan.** Lihat [cara memberi komentar](../../RFC.md) dan [cara berkontribusi](../../CONTRIBUTING.md).

[Jalankan demo](start/index.md) [Dokumentasi](../../index.md)
