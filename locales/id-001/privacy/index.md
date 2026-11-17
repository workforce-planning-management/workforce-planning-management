# Pemberitahuan privasi

Pemberitahuan ini menyatakan apa yang disimpan oleh situs web Workforce Planning Management (WPM) dan aplikasi WPM di
peramban Anda, mengapa, untuk berapa lama, dan cara menghapusnya. Situs webnya adalah
<https://workforce-planning-management.github.io/>; aplikasinya adalah perangkat lunak demonstrasi WPM, di mana pun
perangkat lunak itu dijalankan. (Diperbarui 2026-11-17, WPM-R253.)

Keduanya tidak memiliki analitik, iklan, atau skrip, fon, maupun gambar dari pihak ketiga. Tidak ada yang disimpan
keduanya di peramban Anda yang dipakai untuk mengikuti Anda dari situs ke situs.

## Situs web

Situs web ini adalah sekumpulan halaman statis. Situs ini tidak memiliki fitur masuk akun dan tidak memiliki formulir. Yang disimpannya berada di peramban Anda sendiri:

| Apa | Tempat penyimpanan | Nama | Yang disimpan | Mengapa |
| --- | --- | --- | --- | --- |
| Tema Anda | Penyimpanan lokal peramban Anda | `mxi.wpm.theme` | Nama tema, misalnya `dark` | Agar halaman berikutnya dan kunjungan Anda berikutnya tampil seperti yang Anda pilih |
| Tema Anda | Cookie khusus untuk situs web ini | `wpm-theme` | Nama tema, misalnya `dark` | Agar tema tetap kembali meskipun peramban Anda menyimpan cookie tetapi menghapus penyimpanan lokal, atau sebaliknya |
| Ukuran teks Anda | Penyimpanan lokal peramban Anda | `mxi.wpm.text-size` | Nama ukuran, misalnya `large` | Agar halaman berikutnya dan kunjungan Anda berikutnya memakai ukuran teks yang Anda pilih |
| Bahasa Anda | Tidak ada yang disimpan | | | Bahasa adalah bagian dari alamat setiap halaman (misalnya `/cy-001/`), jadi tidak ada yang perlu diingat |
| Posisi Anda di sebuah halaman | Penyimpanan sesi peramban Anda, hanya untuk tab ini | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Posisi gulir, dan urutan halaman yang Anda buka di tab ini | Agar Kembali dan Maju membawa Anda ke posisi semula. Kerangka kerja yang dipakai untuk membangun situs web ini (SvelteKit) yang menyimpannya |

Tidak ada pilihan yang disimpan sebelum Anda memilih. Sebelum Anda memilih, situs web mengikuti setelan terang atau gelap peramban
Anda dan ukuran teks normalnya, dan tidak menyimpan tema maupun ukuran teks. Pilihan disimpan saat Anda membuatnya di bilah pemilih,
dan memilih lagi akan menggantinya. Penyimpanan sesi ditulis saat Anda berpindah antarhalaman.

Bagian lain situs web tidak menyimpan apa pun:

- **Pencarian** berjalan di peramban Anda, pada indeks yang diterbitkan situs web, dan tidak menyimpan apa pun. Kata-kata Anda dimasukkan ke
  alamat halaman (`?q=`) agar halaman hasil dapat dibagikan; saat alamat itu dimuat dari host web (ketika
  memuat ulang, atau dari tautan yang dibagikan), host menerimanya bersama permintaan, seperti alamat lainnya.
- **Penggeser kesiapan** di jalan menuju produksi tidak menyimpan apa pun, dan memuat ulang halaman mengembalikan perkiraan
  penulis.
- **Bagikan** membuka situs yang Anda pilih (email, LinkedIn, Reddit, Bluesky, atau Mastodon) hanya saat Anda memilihnya. Setelah itu,
  pemberitahuan privasi situs tersebut yang berlaku.

## Aplikasi WPM

Aplikasi menyimpan jenis pilihan yang sama, ditambah cookie yang membuat Anda tetap masuk akun:

| Apa | Tempat penyimpanan | Nama | Yang disimpan | Mengapa |
| --- | --- | --- | --- | --- |
| Tema Anda | Penyimpanan lokal peramban Anda | `mxi.wpm.theme` | Nama tema | Agar setiap halaman tampil seperti yang Anda pilih |
| Ukuran teks Anda | Penyimpanan lokal peramban Anda | `mxi.wpm.text-size` | Nama ukuran | Agar setiap halaman memakai ukuran teks yang Anda pilih |
| Bahasa Anda | Penyimpanan lokal peramban Anda | `mxi.wpm.locale` | Kode bahasa halaman terakhir yang Anda buka, misalnya `cy-001` | Agar alamat tanpa bahasa terbuka dalam bahasa Anda |
| Bahasa Anda | Cookie khusus untuk aplikasi | `wpm-locale` | Kode bahasa yang sama | Agar server dapat mengarahkan alamat tanpa bahasa ke bahasa Anda |
| Status masuk akun Anda | Cookie khusus untuk aplikasi, yang tidak dapat dibaca oleh skrip halaman | `__Host-mxi_session` (`mxi_session` di komputer milik pengembang) | Angka acak yang mewakili sesi masuk akun Anda | Agar Anda tetap masuk akun dari halaman ke halaman; tanpanya Anda harus masuk akun di setiap halaman |
| Bahasa editor konten | Penyimpanan lokal peramban Anda, hanya jika Anda membuka editor konten (`/admin`) | `sveltia-cms.prefs` | Setelan editor, termasuk bahasanya | Agar editor terbuka dalam bahasa aplikasi |
| Posisi Anda di sebuah halaman | Penyimpanan sesi peramban Anda, hanya untuk tab ini | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Posisi gulir, dan urutan halaman yang Anda buka di tab ini | Agar Kembali dan Maju membawa Anda ke posisi semula |

Berbeda dengan situs web, aplikasi menyimpan tema dan ukuran teks begitu menampilkannya, sebelum Anda
memilih, dan menyimpan bahasa setiap halaman yang Anda buka. Kunci dari sebelum proyek berganti nama (`mxi.hcm.theme`, `mxi.hcm.locale`) dipindahkan ke
nama-nama di atas lalu dihapus.

Cookie masuk akun adalah satu-satunya butir yang mewakili Anda. Angka acak di dalamnya tidak berarti apa pun dengan sendirinya: sesi
masuk akun yang ditunjuknya disimpan oleh server aplikasi, dan keluar dari akun mengakhirinya. Apa yang disimpan aplikasi
tentang orang-orang dalam catatannya dibahas di bagian kedua dari akhir, di bawah.

## Siapa yang menerimanya

Penyimpanan lokal dan penyimpanan sesi tidak pernah meninggalkan peramban Anda.

Cookie dikirim bersama setiap permintaan yang dibuat peramban Anda ke situs yang menetapkannya, dan tidak ke situs lain
(`SameSite=Lax`, dan `Secure` melalui HTTPS):

- Cookie `wpm-theme` milik situs web dikirim ke GitHub Pages, yang menghosting situs web. Tidak ada program milik proyek
  yang membacanya.
- Cookie `wpm-locale` dan cookie masuk akun milik aplikasi dikirim ke server tempat aplikasi berjalan, yang membacanya
  untuk memilih bahasa Anda dan untuk mengetahui bahwa Anda sudah masuk akun.

GitHub, yang menghosting situs web, menerima permintaan yang dibuat peramban Anda, seperti host web mana pun.
[Pernyataan privasi GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
menyatakan apa yang dicatatnya. Organisasi yang menjalankan aplikasi menentukan apa yang dicatat servernya.

## Berapa lama disimpan

- Cookie `wpm-theme` milik situs web: 400 hari kalender setelah terakhir kali Anda memilih tema, batas terlama peramban menyimpan
  cookie.
- Cookie `wpm-locale` milik aplikasi: 365 hari kalender setelah terakhir kali Anda membuka halaman.
- Cookie masuk akun: sampai Anda keluar dari akun atau menutup peramban.
- Penyimpanan lokal: sampai Anda menghapusnya.
- Penyimpanan sesi: sampai Anda menutup tab.

## Cara menghapusnya

Hapus data situs di setelan peramban Anda (cookie dan data situs), untuk situs web, aplikasi, atau
keduanya. Setelah itu situs web berperilaku seperti pada kunjungan pertama; aplikasi meminta Anda masuk akun lagi. Keluar dari
aplikasi menghapus cookie masuk akun.

## Mengapa tidak ada spanduk persetujuan

Setiap butir disimpan untuk memberi Anda layanan yang Anda minta: situs web atau aplikasi sesuai setelan Anda, dalam
bahasa Anda, dan, di aplikasi, dalam keadaan masuk akun. Itulah jenis penyimpanan yang oleh aturan Inggris Raya dan Uni Eropa tentang cookie (Peraturan
Privasi dan Komunikasi Elektronik Inggris Raya, dan Direktif Privasi Elektronik Uni Eropa) dianggap mutlak diperlukan, sehingga
tidak memerlukan persetujuan. Ini adalah tafsiran proyek atas aturan tersebut, bukan nasihat hukum.

## Catatan yang disimpan aplikasi

Pemberitahuan ini mencakup apa yang disimpan di peramban Anda. Aplikasi juga menyimpan catatan tentang orang-orang (karyawan,
pelamar, dan kontak mereka) di servernya. Siapa pun yang menjalankan aplikasi bertanggung jawab atas catatan itu dan
menulis pemberitahuan privasi untuk orang-orang di dalamnya
([daftar periksa penilaian](../../../spec/governance/assessment-checklist.md), dalam bahasa Inggris, pertanyaan 3.10). Perangkat lunak
demonstrasi hanya menyimpan data sintetis.

## Pertanyaan

Ajukan pertanyaan di repositori proyek, <https://github.com/workforce-planning-management/workforce-planning-management>,
atau tulis kepada pemelihara yang disebutkan di sana.
