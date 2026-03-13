# xBible 📖

A high-performance, native Bible study application for the **GNOME Desktop**. Built with **Rust**, **GTK4**, and **Libadwaita**, powered by the **SWORD Project** engine.

xBible provides a modern interface for scripture engagement, featuring original language tools, fast searching, and a responsive design that fits perfectly into the Linux ecosystem.

---

## ✨ Features

- **Native GNOME Experience**: Full Libadwaita integration with support for Dark Mode and adaptive layouts.
- **Original Language Support**: Interactive Strong's Greek and Hebrew lookups directly within the text.
- **Flexible Library**: Support for hundreds of Bible versions, commentaries, and lexicons via the [SWORD Project](https://www.crosswire.org/sword/).
- **Fast Navigation**: Quick verse selection and history tracking.
- **Privacy First**: No tracking, no accounts, and works entirely offline once modules are downloaded.

---

## 🛠️ Build & Development

### The GNOME Builder Way (Recommended)
This project is configured to work seamlessly with **GNOME Builder** and **Flatpak**.

1. Install [GNOME Builder](https://apps.gnome.org/Builder/).
2. Select **Clone Repository** and use this project's URL.
3. Once opened, Builder will detect the `org.flame.xbible.json` manifest.
4. Click the **Build** (Hammer) or **Run** (Play) button. Builder will automatically download the Rust SDK and compile the SWORD C++ library within the sandbox.

### Manual Build (Host System)
Ensure you have the following dependencies installed:
- `rustc` & `cargo`
- `meson` & `ninja`
- `libsword-dev` (>= 1.9.0)
- `libadwaita-1-dev` (>= 1.4)
- `gtk4-dev`

```bash
# Setup the build directory
meson setup builddir --prefix=/usr

# Compile the project
meson compile -C builddir

# Install to system
sudo meson install -C builddir