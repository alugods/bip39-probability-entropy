# 🔐 BIP39 Mnemonic Recovery & Probability Analysis (Rust)

Proyek ini merupakan eksplorasi berbasis BIP39 untuk memahami bagaimana kemungkinan (probability) dan proses recovery bekerja pada mnemonic phrase yang sebagian informasinya hilang.

Fokus utama dari proyek ini adalah:

* Analisis ruang kemungkinan (search space)
* Eksperimen recovery mnemonic dengan informasi parsial
* Evaluasi performa komputasi (parallel processing)

---

## 🧠 Konsep Utama

Mnemonic BIP39 terdiri dari 12 kata yang merepresentasikan entropy tertentu.
Jika beberapa kata hilang, maka total kemungkinan kombinasi dapat dihitung sebagai:

```id="fkp9az"
jumlah_kombinasi = jumlah_kandidat ^ jumlah_slot_kosong
```

Proyek ini mengeksplorasi bagaimana ruang kemungkinan tersebut dapat diproses secara efisien menggunakan paralelisme.

---

## ⚙️ Cara Kerja (High-Level)

1. Membaca mnemonic yang sebagian diketahui
2. Mengisi slot kosong dengan kandidat kata
3. Memvalidasi mnemonic sesuai standar BIP39
4. Menghasilkan seed
5. Melakukan derivasi key menggunakan path standar Ethereum:

   ```
   m/44'/60'/0'/0/0
   ```
6. Membandingkan hasil dengan address referensi

Seluruh proses dijalankan secara paralel untuk efisiensi.

---

## 📂 Struktur Proyek

```id="x7d9l2"
.
├── known.txt      # Mnemonic dengan slot kosong ("__")
├── wordlist.txt   # Kandidat kata
├── target.txt     # Address referensi
├── src/
│   └── main.rs
```

---

## ▶️ Menjalankan Project

```bash id="8y9j2l"
cargo run --release
```

Mode `--release` digunakan untuk mendapatkan performa optimal.

---

## 🚀 Fitur

* Parallel computation (Rayon)
* Progress monitoring (speed & estimasi waktu)
* Early termination saat kondisi terpenuhi
* Validasi mnemonic sesuai standar

---

## 📊 Fokus Analisis

Beberapa hal yang dapat diamati dari eksperimen ini:

* Dampak jumlah kata yang hilang terhadap kompleksitas
* Skala pertumbuhan kombinasi (eksponensial)
* Performa CPU terhadap brute-force terkontrol
* Efisiensi parallel processing di Rust

---

## ⚠️ Disclaimer

Proyek ini dibuat untuk:

* Edukasi
* Eksperimen kriptografi
* Pemahaman konsep probabilitas dalam mnemonic recovery

Tidak ditujukan untuk:

* Akses tidak sah ke wallet pihak lain
* Aktivitas yang melanggar hukum atau etika

Penggunaan sepenuhnya menjadi tanggung jawab pengguna.

---

## 🔍 Catatan Teknis

* Menggunakan standar BIP39 untuk mnemonic
* Derivasi key mengikuti standar Ethereum (BIP44)
* Performa sangat bergantung pada ukuran search space

---

## 📌 Pengembangan Selanjutnya

* Analisis probabilistik yang lebih formal
* Benchmark performa lintas hardware
* Eksplorasi optimasi lanjutan
* Visualisasi data hasil eksperimen

---

## 👨‍💻 Author

Hanif Fathoni Abdurrahman
