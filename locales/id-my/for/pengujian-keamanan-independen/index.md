# Pengujian keamanan independen

**Pertanyaan yang mungkin Anda ajukan:** bagaimana membuat orang yang tidak menulis perangkat lunak ini menyerangnya, ke mana mereka perlu diarahkan, dan apa yang kami lakukan dengan temuan mereka?

Belum ada pengujian independen yang dilakukan. [Model ancaman](../../../../spec/governance/threat-model.md) ditulis oleh asisten yang sama yang menulis perangkat lunaknya dan tidak ditinjau oleh siapa pun; setiap kontrol yang disebutnya diuji oleh penulis yang sama. [Penilaian kesiapan](../../../../spec/readiness-assessment.md) mendaftar uji penetrasi independen sebagai gerbang yang hanya dapat ditutup oleh orang (WPM-T201, **belum dikerjakan**). Halaman ini adalah ringkasan bagi yang memesannya. Ini bukan rencana pengujian dari seorang penguji dan tidak menggantikannya.

## Apa yang diberikan kepada penguji

| Butir | Di mana | Catatan |
| --- | --- | --- |
| Penerapan untuk diserang | Milik Anda: [panduan penerapan](../../../../spec/operations/deployment.md) | Pengaturan produksi dengan masuk akun **aktif**, penyedia identitas uji, dan **hanya data sintetis**. Jangan menguji terhadap catatan orang sungguhan |
| Kode sumber | Repositori ini | Pengujian dengan kode sumber lebih menyeluruh daripada kotak hitam; sebutkan mana yang Anda mau |
| Model ancaman | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Dua puluh empat ancaman, kontrol sebagaimana dibangun, dan kesenjangannya. Berikan **setelah** putaran kotak hitam pertama bila Anda ingin melihat apa yang mereka temukan tanpa petunjuk |
| Deskripsi API | `GET /api-docs/openapi.json` | Setiap rute, dengan `Idempotency-Key` pada setiap operasi yang mengubah sesuatu |
| Rancangan masuk akun | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Persona, atribut, dan kenyataan bahwa kebijakan memutuskan hanya berdasarkan tindakan |
| Identitas uji | Anda yang membuatnya | Setidaknya: pekerja tanpa atribut, manajer langsung, manajer di luar rantai, SDM, penggajian, seorang administrator, dan rekan layanan |

## Apa yang diminta untuk mereka periksa

Penulis tidak dapat memeriksa hal-hal ini. Setiap baris menyebut entri model ancaman itu sendiri, sehingga temuan dapat ditelusuri.

| Bidang | Apa yang dicoba | Ancaman |
| --- | --- | --- |
| Token | Kekeliruan algoritma; kumpulan kunci yang berputar saat permintaan berjalan; token kedaluwarsa atau belum berlaku; token untuk audiens lain | T1, T2 |
| Perlu-tahu | Setiap rute baca sebagai pemanggil tanpa atribut dan sebagai tetangga; rute yang tidak dikenal tabel klasifikasi; penulisan yang diizinkan kebijakan tetapi seharusnya ditolak penangan | T3, T4, T6 |
| Atribut istimewa | Apakah atribut dalam token dapat ditambahkan, diputar ulang atau diwariskan; siapa yang dapat menerbitkannya; administrator bersama atau berumur panjang | T24 |
| Injeksi | Fuzzing setiap parameter, segmen jalur, dan bidang isi; pernyataan penghapusan dan retensi dibangun dari pengenal bertipe dan dibaca, bukan di-fuzz | T12 |
| Klien peramban | Penanganan sesi oleh server antarmuka, cara menukar sesi dengan token, skrip lintas situs lewat teks tersimpan, melewati kebijakan keamanan konten, dan halaman pengelola konten yang memuat skrip dari jaringan publik tanpa versi yang disematkan | T13, T14 |
| Permintaan keluar | Pemalsuan permintaan sisi server lewat URL terkonfigurasi apa pun (kumpulan kunci, layanan saudara, cache bersama) | T15 |
| Batas | Batas isi, batas waktu permintaan, batas laju per instans, dan tampilan yang memuat catatan seluruh organisasi ke memori | T16, T17 |
| Log | Apakah data pribadi, token, atau string kueri sampai ke log | T19 |
| Kunci idempotensi | Satu kunci dipakai dua pemanggil; kunci yang sama dengan isi, jalur, atau kueri yang diubah; kunci yang hampir valid; isi yang sangat besar; dua permintaan berpacu di bawah satu kunci (klaimnya dinalar, tidak diuji dalam balapan); apakah selain pemanggilnya ada yang dapat membaca respons tersimpan | baru |
| Jejak audit | Cara apa pun mengubah, menghapus, atau melewati entri; titik akhir verifikasi rantai | T7 |
| Cadangan | Memulihkan dari cadangan yang dirusak atau berkunci salah; apakah orang yang sudah dihapus kembali | T10, T11 |
| Rantai pasok | Penyematan action build, pemindaian rahasia, penandatanganan citra | T20, T21 |

## Aturan yang harus disepakati sebelum mulai

Isi bersama tim keamanan dan hukum Anda sendiri. Tidak satu pun diputuskan di sini.

- **Wewenang:** siapa yang menandatangani bahwa pengujian diizinkan, dan untuk sistem serta alamat mana.
- **Jangka waktu:** tanggal dan jamnya, dihitung dalam hari kalender, dan siapa yang dihubungi untuk menghentikannya.
- **Ruang lingkup:** apa yang termasuk (API, antarmuka, alur masuk akun, skrip cadangan) dan apa yang di luar (layanan penyedia identitas itu sendiri, jaringan Anda, apa pun yang dibagi dengan sistem lain).
- **Data:** hanya sintetis. Jika penguji menemukan data pribadi sungguhan, siapa yang diberi tahu dan bagaimana penanganannya.
- **Pengujian yang merusak:** apakah penolakan layanan, penulisan massal, dan penghapusan diizinkan, dan di mana.
- **Pelaporan:** formatnya, skala tingkat keparahan, siapa yang menerima, berapa lama boleh disimpan, dan bagaimana rahasia yang ditemukan ditangani.
- **Uji ulang:** bahwa kontrak mencakup pemeriksaan perbaikan.

## Apa yang dilakukan dengan temuan

1. Catat masing-masing sebagai tugas di [tasks.md](../../../../spec/tasks.md) dengan rujukan penguji, bukti, dan tingkat keparahan yang mereka berikan. Temuan yang disanggah tetap dicatat sebagai disanggah, beserta alasannya.
2. Perbaiki dengan pengujian yang gagal lebih dulu. Jika perbaikannya berupa aturan, tambahkan ke kolom kesenjangan model ancaman dan tutup di sana hanya bila uji ulang lulus.
3. Jangan tandai WPM-T201 selesai sampai laporan ada di tangan, setiap temuan punya pemilik, dan uji ulang tuntas. Perbarui [penilaian kesiapan](../../../../spec/readiness-assessment.md) dan daftar gerbang (WPM-T203) dalam perubahan yang sama.
4. Simpan laporan di luar repositori jika berisi sesuatu yang dapat dieksploitasi; tautkan ke tempat penyimpanannya.

## Arti laporan yang bersih dan yang bukan artinya

Pengujian tanpa temuan hanya menunjukkan bahwa penguji ini, dalam jangka waktu ini, dengan ruang lingkup dan identitas ini, tidak menemukan apa pun. Ia tidak menunjukkan ketiadaan cacat, tidak mencakup konfigurasi yang Anda ubah sesudahnya, dan tidak menggantikan gerbang lain: audit aksesibilitas, asesmen dampak pelindungan data Anda yang ditandatangani petugas Anda, dan latihan pemulihan di platform Anda.

## Apa yang tidak dilakukan halaman ini

Halaman ini tidak menyebut pemasok, tidak menetapkan harga, dan tidak menjanjikan standar. Skema atau akreditasi apa yang diharapkan sektor Anda dari penguji adalah keputusan Anda.

## Seberapa mutakhir ini

Ditulis pada 2026-10-11 dari spesifikasi, bukan dengan menjalankan perangkat lunak untuk halaman ini. Angka yang berubah (pengujian, migrasi, tugas selesai) tidak disalin ke sini; bacalah di [implementation-status.md](../../../../spec/implementation-status.md), yang dihasilkan oleh sebuah skrip. WPM adalah perangkat lunak demonstrasi dengan data sintetis: halaman ini menyatakan apa yang dilakukan dan tidak dilakukannya, dan tidak menyatakan bahwa ia cocok untuk organisasi Anda. Tidak ada di sini yang merupakan nasihat hukum.

> Terjemahan ini ditulis oleh AI dan belum ditinjau oleh penutur asli. Jika berbeda dari versi bahasa Inggris (`locales/en-gb-oxendict/for/`), versi bahasa Inggris yang berlaku.
