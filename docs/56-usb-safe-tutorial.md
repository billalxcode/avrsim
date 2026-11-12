# 56 - Tutorial Aman Virtual USB Serial

Tutorial ini menggantikan urutan manual yang mudah meninggalkan gadget
setengah terhapus atau membuat `/dev/ttyACM0` sibuk.

**Status:** urutan cleanup sudah diuji dengan driver palsu. Verifikasi
cleanup pada gadget kernel nyata masih perlu dilakukan setelah perubahan
script.

---

## 1. Prasyarat

- Linux dengan `dummy_hcd`, `libcomposite`, `usb_f_acm`, dan `cdc_acm`.
- `sudo` untuk setup gadget.
- Driver C++ dan CLI release sudah dibangun.

```bash
./scripts/build-drivers.sh
cargo build --release -p sim-cli
```

Arsitektur singkat:

```text
Arduino IDE -> /dev/ttyACM0 -> cdc_acm -> dummy_hcd
            -> usb_f_acm -> /dev/ttyGS0 -> embed-emu-usb
            -> Unix socket -> embed-emu usb
```

---

## 2. Mulai dari kondisi bersih

Jangan mulai kalau masih ada bridge atau simulator lama.

Cek:

```bash
pgrep -af "embed-emu-usb"
pgrep -af "target/release/embed-emu usb"
```

Jika ada, hentikan bridge lebih dulu:

```bash
kill -TERM <PID_BRIDGE>
```

Tunggu sampai proses hilang. Setelah itu bersihkan gadget:

```bash
sudo drivers/linux/scripts/usb-gadget.sh stop
```

Sejak perubahan terbaru, `stop` akan gagal cepat dengan pesan:

```text
error: /dev/ttyGS0 is still open; stop the bridge first
```

Pesan itu berarti masih ada bridge yang memegang tty. Hentikan bridge,
lalu ulangi `stop`. Jangan memaksa `rmdir` atau meng-unbind UDC secara
manual.

Cek sisa device:

```bash
ls -l /dev/ttyACM0 /dev/ttyGS0
```

Device boleh tidak ada sebelum gadget dimulai.

---

## 3. Mulai gadget dan bridge

Terminal 1:

```bash
drivers/linux/scripts/run-usb.sh
```

Masukkan password sudo jika diminta. Tunggu sampai muncul:

```text
USB gadget ready: /dev/ttyACM0
```

Script sekarang:

- Memulai gadget ConfigFS ACM.
- Memperbaiki gadget yang setengah terhapus.
- Tidak mengikat UDC dua kali jika sudah terikat.
- Menunggu bridge benar-benar keluar sebelum menjalankan `stop` saat
  Ctrl-C.

Cek dari terminal lain:

```bash
ls -l /dev/ttyACM0
sudo drivers/linux/scripts/usb-gadget.sh status
```

---

## 4. Jalankan simulator

Terminal 2:

```bash
./target/release/embed-emu usb --socket /tmp/embed-emu-usb.sock
```

Biarkan proses ini berjalan. Hanya satu instance yang boleh aktif.

Jika muncul `connection refused`, bridge di terminal 1 belum siap atau
sudah berhenti. Jika ada dua instance, client kedua akan menunggu dan
membuat upload tampak gagal.

---

## 5. Upload sketch

Tutup Serial Monitor dan program lain yang memakai `/dev/ttyACM0` sebelum
upload. Hanya satu program boleh membuka port.

### Arduino IDE

1. Pilih board `Arduino Uno`.
2. Pilih port `/dev/ttyACM0`.
3. Klik Upload.

### avrdude

Contoh dengan toolchain Arduino:

```bash
AVRDUDE="$HOME/.arduino15/packages/arduino/tools/avrdude/8.0.0-arduino1/bin/avrdude"
CONF="$HOME/.arduino15/packages/arduino/tools/avrdude/8.0.0-arduino1/etc/avrdude.conf"

"$AVRDUDE" -C "$CONF" -p atmega328p -c arduino \
  -P /dev/ttyACM0 -b 115200 \
  -U flash:w:path/ke/sketch.hex:i
```

Upload sukses diakhiri dengan:

```text
bytes of flash verified
Avrdude done.  Thank you.
```

---

## 6. Serial Monitor

Sketch contoh memakai:

```cpp
Serial.begin(9200);
Serial.println("Hello World");
```

Set baud Serial Monitor ke `9200`, bukan `115200`. Baud `115200` hanya
dipakai untuk protokol upload avrdude.

Output yang benar:

```text
Hello World
Hello World
Hello World
```

Jika upload berikutnya gagal dengan `Device or resource busy`, tutup
Serial Monitor terlebih dahulu.

---

## 7. Berhenti dengan urutan aman

Urutannya penting:

1. Tutup Serial Monitor dan program yang memakai `/dev/ttyACM0`.
2. Tekan Ctrl-C di terminal 2, yaitu simulator Rust. Tunggu proses
   benar-benar berhenti.
3. Tekan Ctrl-C di terminal 1, yaitu bridge C++.
4. Tunggu sampai script menampilkan `stopped embed_emu`.
5. Verifikasi:

```bash
pgrep -af "embed-emu-usb"
pgrep -af "target/release/embed-emu usb"
ls -l /dev/ttyACM0 /dev/ttyGS0
```

Bridge harus berhenti sebelum `usb-gadget.sh stop` dijalankan. Script
`run-usb.sh` sekarang menangani urutan itu, tetapi Ctrl-C manual pada
bridge tidak boleh disusul `stop` sebelum proses bridge hilang.

---

## 8. Jika muncul "device is busy"

Diagnosis:

```bash
fuser -v /dev/ttyGS0 /dev/ttyACM0
pgrep -af "embed-emu-usb"
pgrep -af "target/release/embed-emu usb"
```

Penyebab umum:

| Gejala | Penyebab | Tindakan |
|---|---|---|
| `stop` menggantung | Bridge masih memegang `/dev/ttyGS0` | Hentikan bridge, ulangi `stop` |
| `stop` menolak | `ttyGS0` masih dibuka | Hentikan bridge lebih dulu |
| UDC busy | Gadget sudah terikat | Jalankan script baru; bind idempoten |
| Upload busy | Serial Monitor atau avrdude lain membuka port | Tutup program tersebut |
| Permission denied | Device dimiliki `root:dialout` dan user bukan anggota `dialout` | Chown device atau tambahkan user ke `dialout` |
| Port tidak ada | Gadget setengah terhapus | `stop`, lalu `run-usb.sh` |
| Dua client menunggu | Lebih dari satu `embed-emu usb` | Hentikan semuanya, jalankan satu |

### 8.1 Permission denied

Cek:

```bash
ls -l /dev/ttyACM0
id
```

Jika hasilnya `root dialout` dan user tidak ada di group `dialout`, lakukan
perbaikan sementara:

```bash
sudo chown "$USER" /dev/ttyACM0 /dev/ttyGS0
```

Lalu ulangi upload. Untuk perbaikan permanen, tambahkan user ke group
`dialout`:

```bash
sudo usermod -aG dialout "$USER"
```

Logout dan login kembali agar group baru berlaku. Script `run-usb.sh`
juga melakukan chown ke `SUDO_USER` setelah `udevadm settle`, tetapi
`uaccess` tidak selalu memberi ACL pada `dummy_hcd` karena device virtual
tersebut tidak punya `ID_SEAT`.

Urutan pemulihan:

```bash
kill -TERM <PID_BRIDGE>
sudo drivers/linux/scripts/usb-gadget.sh stop
drivers/linux/scripts/run-usb.sh
```

Jika `stop` tetap menolak, cari pemegang tty:

```bash
fuser -v /dev/ttyGS0
```

Jangan menghapus direktori ConfigFS dengan `rm -rf`.

---

## 9. Setup manual untuk debugging

Mulai gadget:

```bash
sudo drivers/linux/scripts/usb-gadget.sh start
```

Status:

```bash
sudo drivers/linux/scripts/usb-gadget.sh status
```

Jalankan bridge manual:

```bash
./build/drivers-linux/embed-emu-usb \
  --socket /tmp/embed-emu-usb.sock \
  --device /dev/ttyGS0
```

Jalankan simulator di terminal lain:

```bash
./target/release/embed-emu usb --socket /tmp/embed-emu-usb.sock
```

Saat berhenti, hentikan simulator lebih dulu, lalu bridge, lalu gadget.
Jangan menjalankan `usb-gadget.sh stop` selama bridge masih hidup.

---

## 10. Checklist

- [ ] Tidak ada bridge atau simulator lama.
- [ ] `run-usb.sh` menampilkan `USB gadget ready`.
- [ ] `/dev/ttyACM0` ada dan bisa diakses user.
- [ ] Simulator `embed-emu usb` berjalan satu instance.
- [ ] Serial Monitor ditutup sebelum upload.
- [ ] Upload selesai dengan `flash verified`.
- [ ] Baud Serial Monitor sama dengan `Serial.begin`.
- [ ] Simulator dihentikan sebelum bridge.
- [ ] Bridge berhenti sebelum gadget dihapus.

---

## 11. Batas

- Tutorial ini khusus Linux dan `dummy_hcd`.
- Arduino IDE GUI belum diuji ulang setelah perubahan script.
- Cleanup nyata pada gadget kernel perlu diverifikasi ulang dengan sudo.
- Windows dan macOS belum tersedia.
