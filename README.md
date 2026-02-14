# HarmonyStudio 🎵

> A Digital Audio Workstation (DAW) for Linux, built in Rust

**EPITA S4 Project - Academic Year 2024-2025**

---

## 📖 What is HarmonyStudio?

HarmonyStudio is a **music production software** designed specifically for Linux. It allows users to:

- 🎹 Create music using **5 virtual instruments** (synthesizers)
- 🎵 Arrange and edit notes on a **visual timeline**
- 🎚️ Mix multiple tracks together
- 💾 Import and export **MP3 audio files**
- 🎛️ Apply audio effects (filters, delay, reverb)

Think of it as a simplified version of professional DAWs like FL Studio or Ableton Live, but focused on:
- Native Linux support
- Lightweight and fast performance
- Synthesized (generated) sounds rather than audio samples
- Learning-friendly codebase

---

## 🎯 Project Goals

This project demonstrates:

1. **Real-time audio processing** - Generating and processing sound with minimal latency
2. **DSP algorithms** - Digital Signal Processing for synthesis and effects
3. **Complex software architecture** - Modular design with separated concerns
4. **Rust systems programming** - Using Rust for performance-critical applications
5. **Team collaboration** - 4 developers working on interconnected modules

**Target**: Build a fully functional DAW capable of creating, editing, and exporting music within 5 months.

---

## 👥 Team

| Member | Role | Responsibility |
|--------|------|----------------|
| **Alexandre d'AVEZAC de CASTERA** | Audio Engineer | DSP algorithms, synthesis, effects, polyphony |
| **Léa-Angélina KOLMERSCHLAG** | UI Developer | User interface, timeline editor, visualizations |
| **Hugo GUYENNET** | Sequencer Developer | Event management, timing, transport controls |
| **Quentin JOLY** | File Manager | MP3 encoding/decoding, project persistence |

---

## 🏗️ What We're Building

### The 5 Virtual Instruments

1. **Piano** - Multi-layered oscillators with velocity sensitivity
2. **Bass Synth** - Sub-oscillator with resonant lowpass filter
3. **Lead Synth** - PWM oscillator with filter sweeps and vibrato
4. **Pad Synth** - Detuned oscillators with slow attack for atmospheric sounds
5. **Drum Machine** - Synthesis-based percussion (kick, snare, hi-hat)

### Core Features

**Audio Engine:**
- Real-time sound synthesis using PolyBLEP anti-aliasing
- ADSR envelope control (Attack, Decay, Sustain, Release)
- Polyphonic playback (up to 64 simultaneous notes)
- Biquad filters for tone shaping
- Delay and reverb effects

**Sequencer:**
- Timeline-based note arrangement
- Sample-accurate event scheduling
- Tempo control and synchronization
- Loop markers and regions

**User Interface:**
- Piano roll editor for note input
- Waveform visualization
- Mixer with volume and pan controls
- Instrument parameter panels

**File Management:**
- Import existing MP3 files
- Export projects to MP3 format
- Save/load project files (JSON-based)

---

## 🛠️ Technology Stack

| Component | Technology | Why? |
|-----------|-----------|------|
| Language | **Rust** | Memory safety, performance, no garbage collector |
| Audio I/O | **CPAL** | Cross-platform, low-latency audio |
| GUI | **egui** | Immediate mode, simple to use, performant |
| MP3 | **minimp3** / **mp3lame** | Industry-standard encoding/decoding |
| Build | **Cargo** | Rust's package manager |

---

## 🚀 Getting Started

### Prerequisites

**System Requirements:**
- Linux (Ubuntu 20.04+ recommended)
- Rust 1.70 or later
- ALSA audio libraries
- Working audio hardware

**Install Dependencies:**

```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev pkg-config

# Arch Linux
sudo pacman -S alsa-lib pkgconf

# Fedora
sudo dnf install alsa-lib-devel pkgconfig
```

### Quick Setup

```bash
# 1. Clone the repository
git clone <repository-url>
cd harmonystudio

# 2. Run the setup script
./setup.sh

# 3. Build the project
cargo build --release

# 4. Run HarmonyStudio
cargo run --release
```

For detailed instructions, see [`docs/QUICKSTART.md`](docs/QUICKSTART.md)

---

## 📁 Project Structure

```
harmonystudio/
├── src/
│   ├── audio/          # 🎵 Sound generation and processing
│   ├── sequencer/      # ⏱️ Timeline and event management
│   ├── ui/             # 🖼️ User interface
│   ├── files/          # 💾 File I/O (MP3, projects)
│   └── utils/          # 🔧 Shared utilities
├── docs/               # 📚 Documentation
│   ├── ARCHITECTURE.md # System design
│   ├── ROADMAP.md      # Development plan
│   └── QUICKSTART.md   # Setup guide
├── tests/              # ✅ Tests
└── Cargo.toml          # 📦 Dependencies
```

**Key Files:**
- `src/main.rs` - Application entry point
- `src/audio/synthesis.rs` - Oscillators and waveform generation
- `src/audio/voice.rs` - Polyphony and envelope management
- `src/sequencer/timeline.rs` - Event storage and retrieval
- `src/ui/app.rs` - Main GUI application

---

## 📊 Development Status

### ✅ Completed (Foundation)
- Project structure and architecture
- Basic audio engine framework
- Oscillator implementations
- Timeline data structures
- GUI skeleton with transport controls
- Project file format design

### 🚧 In Progress (Current Sprint)
- Connecting audio engine to hardware
- Implementing PolyBLEP anti-aliasing
- Building timeline editor UI
- Integrating MP3 libraries

### ⏳ Planned (Next Phases)
- All 5 instruments fully functional
- Complete effects chain
- Full timeline editing capabilities
- MP3 import/export working
- Performance optimization
- User documentation

**Current Progress:** ~30% complete

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for detailed timeline.

---

## 🎓 Learning Resources

New to audio programming? Start here:

- [Digital Signal Processing Guide](http://www.dspguide.com/) - Free comprehensive guide
- [PolyBLEP Explained](http://www.martin-finke.de/blog/articles/audio-plugins-018-polyblep-oscillator/) - Anti-aliasing technique
- [Audio EQ Cookbook](https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html) - Filter design formulas
- [Synth Secrets Series](https://www.soundonsound.com/series/synth-secrets) - Sound synthesis tutorials
- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust programming

---

## 🤝 Contributing

This is an academic project with a closed team, but we appreciate:
- 🐛 Bug reports
- 💡 Suggestions
- 📖 Documentation improvements

For team members, see [`CONTRIBUTING.md`](CONTRIBUTING.md) for development guidelines.

---

## 📝 Documentation

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design and module interactions
- **[ROADMAP.md](docs/ROADMAP.md)** - Development phases and milestones
- **[QUICKSTART.md](docs/QUICKSTART.md)** - Detailed setup instructions
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development workflow and standards
- **[PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)** - Comprehensive project overview

---

## 📞 Contact

**EPITA - S4 Project**  
Academic Year 2024-2025

**Team Members:**
- Alexandre d'AVEZAC de CASTERA
- Léa-Angélina KOLMERSCHLAG
- Hugo GUYENNET
- Quentin JOLY

For questions or collaboration: [Contact via EPITA]

---

## 📄 License

This project uses a **dual licensing model**:

### During Academic Period (January 2025 - June 2025)
**All Rights Reserved - Academic Use Only**

This is a student project for EPITA S4 curriculum. During the academic evaluation period:
- ✅ Permitted: Faculty review, team development, portfolio inclusion
- ❌ Prohibited: Copying for other academic submissions (plagiarism), commercial use, public redistribution

### After Academic Period (After June 30, 2025)
**MIT License**

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

---

**See [LICENSE](LICENSE) for complete terms**

---

<p align="center">
  <strong>Built with 🦀 Rust & ❤️ for Linux</strong><br>
  <sub>EPITA S4 Project 2024-2025</sub>
</p>
