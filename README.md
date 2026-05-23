<a name="top"></a>
<div align="center">

# 🖥️ grainx

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey?style=for-the-badge)](#platform-uyumluluğu)

**Modern, cross-platform, terminal tabanlı sistem izleme aracı**

*Rust ile yazılmış, gerçek zamanlı sistem analizi ve güzel Unicode grafikleri*

[Özellikler](#-özellikler) • [Kurulum](#-kurulum) • [Kullanım](#-kullanım) • [Konfigürasyon](#-konfigürasyon)

![grainx Demo](https://via.placeholder.com/800x400/1a1a1a/00ff00?text=grainx+Sistem+Monitörü)

</div>

---

## ✨ Özellikler

<table>
<tr>
<td width="50%">

### 🎨 **Yüksek Çözünürlük Grafikleri**
- Unicode Braille karakter grafikleri (8x yüksek çözünürlük)
- Gerçek zamanlı CPU ve bellek grafikleri
- Dinamik renk kodlaması
- Akıcı kaydırmalı zaman çizelgesi

### 🧠 **Akıllı Analitik**
- İstatistiksel anomali tespiti
- Metrikler arası korelasyon analizi
- Tahmine dayalı CPU kullanım tahmini
- Özel metrik formülleri

</td>
<td width="50%">

### ⚡ **Performans Optimizasyonu**
- Adaptif yenileme hızları
- Yük altında frame atlama
- Bellek verimli tasarım
- Engelleyici olmayan girdi işleme

### 🖱️ **Etkileşimli Arayüz**
- Process seçimi ve yönetimi
- Gerçek zamanlı process sonlandırma
- Türkçe yardım sistemi
- Duraklat/Devam et işlevi

</td>
</tr>
</table>

### 📊 **Kapsamlı İzleme**

| Bileşen | Özellikler |
|---------|------------|
| **CPU** | Genel kullanım + bireysel çekirdek izleme (8 çekirdeğe kadar) |
| **Bellek** | Kullanım yüzdesi ve mutlak değerler, trend analizi |
| **Disk** | Çoklu sürücü kullanım istatistikleri ve kapasite bilgisi |
| **Ağ** | Gerçek zamanlı I/O istatistikleri ve throughput izleme |
| **Sistem** | OS bilgisi, kernel versiyonu, uptime ve sistem detayları |
| **Processler** | En çok CPU kullanan processler ve detaylı bilgiler |

---

## 🚀 Kurulum

### Ön Gereksinimler
- **Rust 1.70+** ([Rust Kurulumu](https://rustup.rs/))
- **Unicode destekli terminal**

### Hızlı Başlangıç

```bash
# Repository'yi klonla
git clone https://github.com/rustfuture/grainx.git
cd grainx

# Direkt çalıştır
cargo run

# Veya optimize edilmiş sürümü derle
cargo build --release
./target/release/grainx
```

### Platform Bazlı Kurulum

<details>
<summary><b>🐧 Linux</b></summary>

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Fedora/RHEL
sudo dnf groupinstall "Development Tools"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Arch Linux
sudo pacman -S base-devel
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

</details>

<details>
<summary><b>🍎 macOS</b></summary>

```bash
# Rust kurulumu
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Veya Homebrew ile
brew install rustup
rustup-init
```

</details>

<details>
<summary><b>🪟 Windows</b></summary>

```powershell
# winget kullanarak
winget install Rustlang.Rust

# Veya https://rustup.rs/ adresinden indirin
# Sonra PowerShell veya Windows Terminal'de çalıştırın
```

</details>

---

## 🎮 Kullanım

### Temel Komutlar

```bash
cargo run                      # Yerel TUI (monitor, varsayılan)
cargo run -- monitor             # Açıkça monitor modu
cargo run -- agent -p 9090       # HTTP metrics agent
cargo run -- monitor --remote http://127.0.0.1:9090  # Remote TUI
cargo run -- version             # Sürüm bilgisi
cargo run -- export              # Headless JSON/CSV export (TUI gerekmez)
cargo run -- export --remote http://127.0.0.1:9090  # Remote export
grainx completions bash > /etc/bash_completion.d/grainx  # Shell tamamlama
cargo test                       # Test paketini çalıştır
cargo clippy -- -D warnings      # Lint kontrolü
cargo bench                      # Benchmark'ları çalıştır
cargo build --release            # Optimize edilmiş binary derle
```

### Agent API

```bash
# Terminal 1
grainx agent --bind 127.0.0.1 -p 9090

# Terminal 2
curl http://127.0.0.1:9090/health
curl http://127.0.0.1:9090/metrics
```

### Klavye Kontrolleri

<div align="center">

| Tuş | İşlev | Tuş | İşlev |
|-----|-------|-----|-------|
| `q` / `ESC` | Programdan çık | `h` / `?` | Yardım menüsü |
| `↑` / `↓` | Process'lerde gezin | `p` | Duraklat/Devam |
| `k` | Seçili process'i öldür | `r` | Ekranı yenile |
| `a` | Adaptif yenilemeyi aç/kapat | `s` | İstatistikleri kaydet |

</div>

### Ekran Görüntüsü

<details>
<summary><b>📸 Terminal Görünümü</b></summary>

```
┌─ grainx Sistem Monitörü ─ İterasyon: 42 ─┐
│ CPU Kullanımı:  45.2%                     │
│ ████████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  │
│                                           │
│ Bellek: 68.5% (5.5GB/8.0GB)              │
│ ████████████████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  │
│                                           │
│ Sistem: Linux | Kernel: 5.15.0 | Up: 2s  │
│ Ağ: RX:125.3MB TX:89.7MB                  │
│ CPU Çekirdekleri: C0:42% C1:38% C2:51%    │
│ Diskler: sda:75%(465GB) nvme:45%(1TB)     │
│                                           │
│ En Çok CPU Kullanan (↑↓=seç, k=öldür)    │
│ ► 1234  firefox         25.3%    512MB    │
│   5678  code            15.8%    256MB    │
│   9012  grainx           2.1%     8MB     │
└───────────────────────────────────────────┘
```

</details>

---

## ⚙️ Konfigürasyon

grainx özelleştirme için JSON konfigürasyon dosyası kullanır:

<details>
<summary><b>📝 dashboard_config.json</b></summary>

```json
{
  "name": "grainx_gelişmiş",
  "layout": [
    "cpu_graph",
    "memory_usage", 
    "network_stats",
    "process_list",
    "analytics"
  ],
  "refresh_interval_ms": 500,
  "cpu_warning_threshold": 80.0,
  "memory_warning_threshold": 85.0,
  "show_predictions": true,
  "show_correlations": true,
  "max_processes": 10,
  "graph_history_size": 100
}
```

</details>

### Konfigürasyon Seçenekleri

| Seçenek | Tip | Varsayılan | Açıklama |
|---------|-----|------------|----------|
| `cpu_warning_threshold` | `f32` | `80.0` | Uyarı renkleri için CPU kullanım % |
| `memory_warning_threshold` | `f32` | `85.0` | Uyarılar için bellek kullanım % |
| `show_predictions` | `bool` | `true` | CPU kullanım tahminlerini etkinleştir |
| `show_correlations` | `bool` | `true` | Korelasyon analizini etkinleştir |
| `max_processes` | `usize` | `10` | Gösterilecek maksimum process sayısı |
| `graph_history_size` | `usize` | `100` | Grafiklerdeki veri noktası sayısı |
| `color_theme` | `string` | `default` | Renk teması: default, dark, light, high_contrast |
| `log_enabled` | `bool` | `true` | Metrik log dosyasına yazma |
| `log_path` | `string` | `grainx_metrics.log` | Metrik log dosya yolu |

### Ortam Değişkenleri (config dosyasını geçersiz kılar)

| Değişken | Açıklama |
|----------|----------|
| `GRAINX_REFRESH_INTERVAL_MS` | Yenileme aralığı (ms) |
| `GRAINX_CPU_WARNING_THRESHOLD` | CPU uyarı eşiği (%) |
| `GRAINX_MEMORY_WARNING_THRESHOLD` | Bellek uyarı eşiği (%) |
| `GRAINX_COLOR_THEME` | Renk teması adı |

CLI bayrakları config dosyası ve ortam değişkenlerinin üzerinde uygulanır.

---

## 🧪 Testler

# Tüm testleri çalıştır
cargo test

# Belirli modülleri test et
cargo test analytics
cargo test performance  
cargo test config

# Integration testlerini çalıştır
# Benchmark'ları çalıştır
cargo bench
```

### Test Kategorileri

- **Unit Testler**: Bireysel modül işlevselliği
- **Integration Testler**: Uçtan uca sistem davranışı  
- **Performance Testler**: Kritik yolların benchmark'ı
- **Platform Testler**: Cross-platform uyumluluk

---

## 🌍 Platform Uyumluluğu

<div align="center">

| Platform | Destek | Notlar |
|----------|:------:|--------|
| **🐧 Linux** | ✅ **Tam** | Native geliştirme platformu |
| **🍎 macOS 11+** | ✅ **Tam** | Tam özellik seti |
| **🪟 Windows 10+** | ✅ **Tam** | Windows Terminal ile en iyi deneyim |

</div>

### Performans Metrikleri

| Metrik | Değer | Notlar |
|--------|-------|--------|
| **Bellek Kullanımı** | ~5-10MB | Tipik çalışma zamanı kullanımı |
| **CPU Etkisi** | <1% | Modern sistemlerde |
| **Yenileme Hızı** | 250ms-2s | Yüke göre adaptif |
| **Başlama Süresi** | ~100-200ms | Platforma bağlı |

---

## 🐛 Sorun Giderme

<details>
<summary><b>Yaygın Sorunlar ve Çözümler</b></summary>

### Unicode Görüntü Sorunları
```bash
# Linux/macOS
export LANG=tr_TR.UTF-8
export LC_ALL=tr_TR.UTF-8

# Windows
chcp 65001
```

### İzin Hataları
```bash
# Linux/macOS - tam sistem erişimi için
sudo ./target/release/grainx

# Windows - Yönetici olarak çalıştır
```

### Yüksek CPU Kullanımı
- Adaptif yenilemeyi açmak için `a` tuşuna basın
- Config'de yenileme aralığını artırın
- Sistem darboğazlarını kontrol edin

</details>

### Debug Modu

```bash
RUST_LOG=debug cargo run
RUST_LOG=grainx=trace cargo run  # Ayrıntılı loglama
```

---

## 🤝 Katkıda Bulunma

Katkılarınızı memnuniyetle karşılıyoruz! Başlamak için:

### Geliştirme Kurulumu

```bash
# Fork ve clone
git clone https://github.com/rustfuture/grainx.git
cd grainx

# Özellik branch'i oluştur
git checkout -b feature/harika-ozellik

# Değişiklik yap ve test et
cargo test
cargo clippy
cargo fmt

# Commit ve push
git commit -m "Harika özellik ekle"
git push origin feature/harika-ozellik
```

### Katkı Rehberi

- 🧪 **Yeni işlevsellik için test ekle**
- 📝 **API değişiklikleri için dokümantasyonu güncelle**  
- 🎨 **Rust konvansiyonlarını takip et** (rustfmt, clippy)
- ✅ **Göndermeden önce tüm testlerin geçtiğinden emin ol**
- 📋 **Açık commit mesajları yaz**

---

## 📄 Lisans

Bu proje **MIT Lisansı** altında lisanslanmıştır - detaylar için [LICENSE](LICENSE) dosyasına bakın.

---

## 🙏 Teşekkürler

- 🦀 **[sysinfo](https://github.com/GuillaumeGomez/sysinfo)** - Cross-platform sistem bilgisi
- 🖥️ **[crossterm](https://github.com/crossterm-rs/crossterm)** - Terminal manipülasyonu
- 📊 **[criterion](https://github.com/bheisler/criterion.rs)** - Performance benchmark'ı
- 🎨 **Unicode Consortium** - Braille karakter standartları

---

<div align="center">

### ⭐ Bu projeyi faydalı buluyorsanız yıldızlamayı unutmayın!

Made with ❤️ and Rust 🦀

[⬆ Başa Dön](#top)

</div>