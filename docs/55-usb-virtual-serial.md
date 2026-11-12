# 55 - Virtual USB Serial Port

Dokumen ini merangkum riset untuk membuat virtual USB serial port.

**Status:** riset, bridge C++, ConfigFS ACM, dan upload STK500v1 berhasil; verifikasi GUI Arduino IDE dan cleanup gadget belum.

**Kesimpulan awal:**

- USB serial virtual yang benar-benar muncul sebagai `/dev/ttyACM0` bisa
  dibuat di Linux tanpa hardware USB.
- Jalur tercepat: `dummy_hcd` + ConfigFS ACM.
- Jalur lintas platform: server USB/IP di Rust + client USB/IP di OS host.
- PTY tetap berguna sebagai fallback, tetapi bukan USB device.
- Windows punya jalur USB/IP yang matang.
- macOS adalah platform paling sulit untuk USB virtual murni.

---

## 1. Istilah

| Istilah | Arti |
|---|---|
| USB host | Komputer yang memiliki host controller, misalnya PC Linux |
| USB device | Perangkat yang dicolok ke host, misalnya Arduino |
| Gadget | Implementasi USB device di Linux |
| UDC | USB Device Controller, hardware atau virtual controller untuk gadget |
| CDC ACM | Communication Device Class, Abstract Control Model |
| `ttyGS0` | Sisi gadget serial di Linux |
| `ttyACM0` | Sisi host untuk USB CDC ACM |
| USB/IP | Protokol berbagi USB device lewat TCP/IP |
| VHCI | Virtual Host Controller Interface di Linux |
| VUDC | Virtual USB Device Controller di Linux |

---

## 2. Cara kerja USB CDC ACM

CDC ACM adalah standar USB untuk modem dan virtual COM port. Host melihat
device ini sebagai serial port.

### 2.1 Enumeration

Saat device dicolok atau di-attach, host melakukan enumeration:

1. Host mengirim `GET_DESCRIPTOR` untuk device descriptor.
2. Host memilih alamat dengan `SET_ADDRESS`.
3. Host membaca configuration descriptor.
4. Host membaca string descriptor.
5. Host memilih konfigurasi dengan `SET_CONFIGURATION`.

Descriptor penting:

- Device descriptor.
- Configuration descriptor.
- Interface descriptor untuk Communication Class Interface.
- Interface descriptor untuk Data Class Interface.
- Endpoint descriptor.
- CDC functional descriptor.

### 2.2 Endpoint CDC ACM

CDC ACM biasanya memakai tiga endpoint:

| Endpoint | Arah | Fungsi |
|---|---|---|
| Control endpoint | Dua arah | Enumeration dan class request |
| Interrupt IN | Device ke host | Notifikasi `SERIAL_STATE` |
| Bulk IN | Device ke host | Data serial dari device |
| Bulk OUT | Host ke device | Data serial ke device |

Control endpoint selalu endpoint 0.

### 2.3 Class request CDC ACM

Request CDC ACM memakai `bmRequestType` kelas dan interface:

| Request | Kode | Arah | Isi |
|---|---|---|---|
| `SET_LINE_CODING` | `0x20` | Host ke device | Baud, stop bit, parity, data bits |
| `GET_LINE_CODING` | `0x21` | Device ke host | Konfigurasi line coding |
| `SET_CONTROL_LINE_STATE` | `0x22` | Host ke device | DTR dan RTS |
| `SEND_BREAK` | `0x23` | Host ke device | Break duration |

Struktur line coding berukuran 7 byte:

```text
dwDTERate    4 byte little-endian
bCharFormat  1 byte
bParityType  1 byte
bDataBits    1 byte
```

`SET_CONTROL_LINE_STATE` memakai `wValue`:

```text
bit 0: DTR
bit 1: RTS
```

Arduino IDE dan avrdude memakai DTR untuk memicu reset bootloader. Karena
itu, DTR harus diteruskan ke simulator atau diabaikan dengan aman.

### 2.4 Notifikasi serial state

Device dapat mengirim notifikasi 10 byte melalui interrupt IN:

```text
bmRequestType = 0xA1
bNotification = 0x20 (SERIAL_STATE)
wValue        = 0
wIndex        = interface
wLength       = 2
data          = bitmap state
```

Bit state mencakup overrun, parity error, framing error, ring, break,
carrier TX, dan carrier RX.

### 2.5 Driver host

| OS | Driver | Nama port |
|---|---|---|
| Linux | `cdc_acm` | `/dev/ttyACM0` |
| Windows 10+ | `usbser.sys` | `COMx` |
| macOS | `AppleUSBCDC` | `/dev/cu.usbmodem...` |

Windows memuat `usbser.sys` otomatis untuk device dengan class `0x02` dan
subclass `0x02` pada device descriptor. Tidak perlu INF khusus pada
Windows 10 dan newer.

---

## 3. Opsi Linux

### 3.1 `dummy_hcd` + ConfigFS ACM

`dummy_hcd` adalah driver kernel yang menyediakan host controller virtual
dan UDC virtual di mesin yang sama. Gadget yang di-bind ke `dummy_udc.0`
akan terlihat oleh host lokal sebagai USB device.

Alur:

1. Muat `dummy_hcd`, `libcomposite`, dan `usb_f_acm`.
2. Buat gadget melalui ConfigFS.
3. Tambahkan function `acm`.
4. Bind gadget ke `dummy_udc.0`.
5. Host lokal melihat `/dev/ttyACM0`.
6. Sisi gadget muncul sebagai `/dev/ttyGS0`.

Rust dapat:

- Menulis file ConfigFS memakai `std::fs`.
- Membuka `/dev/ttyGS0` memakai `std::fs::File`.
- Membaca dan menulis byte serial.

Tidak perlu `unsafe` untuk alur utama.

Kebutuhan:

- Root untuk `modprobe`, mount ConfigFS, dan bind UDC.
- Kernel dengan `CONFIG_USB_DUMMY_HCD`, `CONFIG_USB_CONFIGFS`, dan
  `CONFIG_USB_CONFIGFS_ACM`.
- Hak akses ke `/dev/ttyGS0`, biasanya lewat group atau udev rule.

Kelebihan:

- Host melihat USB device asli, bukan PTY.
- Arduino IDE dapat mendeteksi `/dev/ttyACM0`.
- Tidak perlu menulis protokol USB di Rust.
- Tidak perlu USB/IP atau jaringan.

Kekurangan:

- Butuh root saat setup.
- `dummy_hcd` adalah driver testing, bukan driver produksi.
- DTR/RTS ke `ttyGS0` perlu diuji.
- Hanya Linux.

### 3.2 `usbip-vudc` + ConfigFS ACM + VHCI

`usbip-vudc` membuat UDC virtual yang diekspor lewat USB/IP. Di mesin yang
sama, `vhci-hcd` dapat meng-attach device tersebut.

Alur:

1. Muat `usbip-vudc`, `libcomposite`, dan `vhci-hcd`.
2. Buat gadget ACM di ConfigFS.
3. Bind gadget ke `usbip-vudc.0`.
4. Jalankan `usbipd --device`.
5. Attach dari client lokal:

   ```bash
   usbip attach --remote=127.0.0.1 --device=usbip-vudc.0
   ```

6. Host melihat `/dev/ttyACM0`.

Kelebihan:

- Memakai protokol USB/IP yang juga bisa dipakai Windows.
- Tidak butuh hardware UDC.

Kekurangan:

- Lebih banyak komponen: `usbipd`, VHCI, VUDC.
- Butuh root dan module kernel.
- Lebih sulit di-debug daripada `dummy_hcd`.

### 3.3 FunctionFS

FunctionFS memungkinkan user space menyediakan descriptor dan menangani
setup request USB. Kernel tetap menangani endpoint dan UDC.

Kelebihan:

- Kontrol penuh atas descriptor CDC ACM.
- Bisa menangani DTR, RTS, line coding, dan break sendiri.
- Bisa dipakai dengan `dummy_hcd`, UDC nyata, atau `usbip-vudc`.

Kekurangan:

- Butuh ioctl dan event loop endpoint.
- Rust `std` tidak menyediakan ioctl; perlu FFI atau crate eksternal.
- Implementasi lebih besar daripada ConfigFS ACM.

### 3.4 Raw Gadget

Raw Gadget memberikan akses langsung ke USB request dari user space.

Kelebihan:

- Paling fleksibel.
- Bisa mengemulasikan device apa pun.

Kekurangan:

- Interface ioctl.
- Dokumentasi kernel menyebutnya fitur debugging, bukan produksi.
- Butuh `CONFIG_USB_RAW_GADGET`.
- Butuh `unsafe` atau FFI.
- Implementasi protokol USB menjadi tanggung jawab penuh kita.

### 3.5 PTY

PTY adalah tty asli, tetapi bukan USB device.

Kelebihan:

- Tanpa root.
- Tanpa kernel module.
- Sudah berjalan di proyek ini melalui `socat`.

Kekurangan:

- Tidak punya VID/PID.
- Tidak muncul sebagai `/dev/ttyACM0`.
- Arduino IDE tidak mendeteksi dengan discovery normal.

---

## 4. Opsi Windows

Windows tidak menyediakan cara user-space murni untuk membuat USB device.
Diperlukan driver kernel atau virtual bus driver.

### 4.1 `usbser.sys`

Jika device CDC ACM memiliki class `0x02` dan subclass `0x02`, Windows 10
memuat `usbser.sys` otomatis dan membuat `COMx`. Tidak perlu INF.

### 4.2 USB/IP client `usbip-win2`

`usbip-win2` menyediakan UDE (USB Device Emulation) driver dan client
USB/IP untuk Windows.

- Client terhubung ke server USB/IP.
- Windows melihat device sebagai USB device nyata.
- `usbser.sys` membuat COM port.
- Driver ditandatangani dan membutuhkan administrator saat instalasi.
- Protokol yang didukung: USB/IP 1.1.1.

Jalur ini cocok jika simulator menyediakan server USB/IP di Rust.

### 4.3 Driver virtual sendiri

Alternatifnya adalah menulis UDE-based driver atau kernel driver sendiri.
Ini membutuhkan WDK, signing, dan instalasi administrator. Tidak cocok
untuk tahap awal.

---

## 5. Opsi macOS

macOS adalah platform paling sulit untuk virtual USB murni.

### 5.1 DriverKit dan USBSerialDriverKit

DriverKit dapat membuat driver untuk USB device nyata. Untuk membuat
device virtual, diperlukan virtual USB host controller atau driver dext
khusus. Ini membutuhkan entitlement, signing, dan instalasi pengguna.

### 5.2 USB/IP di macOS

Tidak ada client USB/IP bawaan di macOS. Ada proyek pihak ketiga, tetapi
bukan jalur `std`-only Rust.

### 5.3 Fallback macOS

Fallback yang realistis:

- PTY + custom Arduino discovery plugin.
- Symlink `/dev/cu.*` dengan hak akses yang sesuai.
- DriverKit hanya jika benar-benar membutuhkan USB virtual native.

---

## 6. Arduino IDE dan discovery

Dengan USB device nyata seperti `/dev/ttyACM0`, Arduino IDE memakai
`serial-discovery` bawaan dan port muncul otomatis.

Dengan PTY, port tidak muncul karena bukan USB device. Solusinya adalah
pluggable discovery plugin. Protokolnya berbasis stdin/stdout dan JSON:

```json
{"eventType":"add","port":{"address":"/tmp/embed-emu-tty","label":"embed-emu","protocol":"serial","protocolLabel":"Serial Port"}}
```

Plugin dapat ditulis di Rust tanpa `unsafe`. Namun upload tetap memakai
avrdude dan perilaku DTR perlu diuji.

---

## 7. Rekomendasi arsitektur Rust

Pisahkan protokol dari transport.

```text
+---------------------+
| STK500 / AVR flash  |
+----------+----------+
           |
+----------v----------+
| SerialTransport     |
| read / write / DTR  |
+----------+----------+
           |
   +-------+--------+----------------+
   |                |                |
+--v------+   +-----v------+   +-----v------+
| ttyGS0  |   | USB/IP     |   | PTY        |
| Linux   |   | server     |   | fallback   |
+---------+   +------------+   +------------+
```

Trait yang disarankan:

```rust
trait SerialTransport {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError>;
    fn write(&mut self, bytes: &[u8]) -> Result<(), TransportError>;
    fn control_lines(&self) -> ControlLines;
}
```

Backend:

| Backend | Platform | Root | Unsafe | Prioritas |
|---|---|---|---|---|
| `dummy_hcd` + ConfigFS ACM | Linux | Ya, setup | Tidak | 1 |
| USB/IP server + VHCI | Linux | Ya, attach | Tidak | 2 |
| USB/IP server + usbip-win2 | Windows | Ya, install | Tidak | 2 |
| FunctionFS | Linux | Ya, setup | Ya/FFI | 3 |
| PTY + discovery plugin | Semua | Tidak | Tidak | Fallback |
| DriverKit virtual USB | macOS | Ya | Tidak di Rust | Nanti |

---

## 8. Rencana bertahap

### Fase 1 - Linux nyata

- Setup script dengan sudo untuk `dummy_hcd` dan ConfigFS ACM.
- CLI Rust mendeteksi `/dev/ttyGS0`.
- Protokol STK500 yang sudah ada dipindahkan ke trait transport.
- Host melihat `/dev/ttyACM0`.
- Uji Arduino IDE upload dan Serial Monitor.

### Fase 2 - USB/IP

- Implementasikan USB/IP server di Rust memakai `std::net::TcpStream`.
- Emulasikan CDC ACM dan endpoint.
- Linux memakai `vhci-hcd`.
- Windows memakai `usbip-win2`.
- Tidak perlu driver khusus di sisi simulator.

### Fase 3 - macOS

- PTY + discovery plugin sebagai fallback.
- Evaluasi DriverKit jika USB native wajib.

---

## 9. Risiko

- `dummy_hcd` adalah driver testing dan bisa berkonflik dengan gadget lain.
- Setup root dan module kernel menyulitkan distribusi.
- DTR/RTS pada `ttyGS0` belum tentu tersedia tanpa ioctl.
- USB/IP membutuhkan port TCP dan binding localhost yang aman.
- Windows membutuhkan driver signed dan administrator.
- macOS membutuhkan signing/entitlement untuk dext.
- Perbedaan perilaku DTR antar avrdude dan Arduino IDE.
- Kecepatan USB bukan bottleneck utama; emulasi CPU dan simulator tetap
  bottleneck.

---

## 10. Rekomendasi akhir

Untuk Linux sekarang:

1. Gunakan `dummy_hcd` + ConfigFS ACM.
2. Host melihat `/dev/ttyACM0`.
3. CLI membaca dan menulis `/dev/ttyGS0`.
4. Setup root dipisahkan ke script.
5. Protokol STK500 tetap sama.

Untuk lintas platform nanti:

1. Implementasikan USB/IP server di Rust.
2. Gunakan `vhci-hcd` di Linux.
3. Gunakan `usbip-win2` di Windows.
4. Gunakan PTY + discovery plugin di macOS sampai driver virtual tersedia.

Jangan mulai dari FunctionFS atau Raw Gadget kecuali kita membutuhkan
kontrol descriptor dan DTR penuh. Untuk kebutuhan Arduino IDE, ConfigFS
ACM sudah cukup.

Todolist implementasi driver C++ ada di [`TODO.md`](../TODO.md), bagian
P18 dan P19.

Rencana struktur dan batas komponen ada di
[`drivers/README.md`](../drivers/README.md).

---

## 11. Status implementasi

Sudah ada:

- Bridge C++ `drivers/linux` antara `/dev/ttyGS0` dan Unix socket.
- Script setup dan teardown ConfigFS ACM.
- Perintah Rust `embed-emu usb --socket PATH`.
- Parser STK500v1 untuk avrdude dan AVR109 untuk kompatibilitas.
- Test integrasi bridge memakai PTY.

Sudah diverifikasi:

- `/dev/ttyACM0` muncul dari `dummy_hcd` dan ConfigFS ACM.
- Akses `/dev/ttyACM0` tanpa sudo.
- `avrdude 8.0` sync, signature, upload, dan verify.
- Firmware blink berjalan dan LED berubah.
- Firmware hasil `avr-gcc` berjalan, termasuk `ADIW` dan `SBIW`.
- DTR dari avrdude tidak memutus bridge.

Belum diverifikasi:

- Upload dari GUI Arduino IDE.
- Cleanup gadget setelah Ctrl-C.

Perintah:

```bash
./scripts/build-drivers.sh
python3 scripts/test-usb-bridge.py
drivers/linux/scripts/run-usb.sh
./target/release/embed-emu usb --socket /tmp/embed-emu-usb.sock
```

Detail ada di [`drivers/linux/README.md`](../drivers/linux/README.md).

---

## 12. Sumber

- Linux USB gadget ConfigFS:
  https://docs.kernel.org/usb/gadget_configfs.html
- Linux FunctionFS:
  https://docs.kernel.org/usb/functionfs.html
- Linux Raw Gadget:
  https://www.kernel.org/doc/html/v5.9/usb/raw-gadget.html
- Linux USB/IP protocol:
  https://docs.kernel.org/usb/usbip_protocol.html
- Linux USB/IP vUDC example:
  https://github.com/torvalds/linux/blob/master/tools/usb/usbip/vudc/vudc_server_example.sh
- Collabora dummy_hcd:
  https://www.collabora.com/news-and-blog/blog/2019/06/24/using-dummy-hcd/
- USB CDC class definitions:
  https://www.usb.org/document-library/class-definitions-communication-devices-12
- USB CDC PSTN subclass:
  https://community.nxp.com/pwmxy87654/attachments/pwmxy87654/mcuxpresso-sdk/2095/1/PSTN120.pdf
- Silicon Labs CDC ACM overview:
  https://docs.silabs.com/protocol-usb/1.7.2/protocol-usb-cdc/
- Microsoft usbser.sys:
  https://learn.microsoft.com/en-us/windows-hardware/drivers/usbcon/usb-driver-installation-based-on-compatible-ids
- usbip-win2:
  https://github.com/vadimgrn/usbip-win2
- Arduino pluggable discovery:
  https://arduino.github.io/arduino-cli/0.20/pluggable-discovery-specification
- Apple DriverKit:
  https://developer.apple.com/documentation/driverkit
- Apple USBSerialDriverKit:
  https://developer.apple.com/documentation/usbserialdriverkit
