# RoleTect

[![Releases](https://img.shields.io/github/v/release/AhmedTrooper/RoleTect?style=flat-square&color=blue)](https://github.com/AhmedTrooper/RoleTect/releases/latest)
[![Firefox Add-on](https://img.shields.io/badge/Firefox-Add--on-FF7139?style=flat-square&logo=firefox-browser&logoColor=white)](https://addons.mozilla.org/en-US/firefox/addon/roletect-ingest/)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-brightgreen?style=flat-square)](https://github.com/AhmedTrooper/RoleTect/releases/latest)
[![Bun](https://img.shields.io/badge/Bun-1.3+-black?style=flat-square&logo=bun)](https://bun.sh/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-24C8D8?style=flat-square&logo=tauri)](https://tauri.app/)

**RoleTect** is a local-first, privacy-focused desktop application and companion browser extension engineered to centralize your job application pipeline, parse job descriptions into structured requirements, and tailor professional LaTeX resumes and cover letters using sovereign AI models.

---

## 📸 Interface Preview

<table>
  <tr>
    <td width="50%"><img src="./assets/RoleTect/home_page.png" alt="RoleTect Home Dashboard" /></td>
    <td width="50%"><img src="./assets/RoleTect/extention_job_vault.png" alt="Extension Job Vault" /></td>
  </tr>
  <tr>
    <td width="50%"><img src="./assets/RoleTect/job_details_compile.png" alt="LaTeX Compilation & Preview" /></td>
    <td width="50%"><img src="./assets/RoleTect/job_details_comparison_base_vs_tailored.png" alt="Base vs Tailored Comparison" /></td>
  </tr>
  <tr>
    <td width="50%"><img src="./assets/RoleTect/latex_workspace.png" alt="LaTeX Workspace & CodeMirror Editor" /></td>
    <td width="50%"><img src="./assets/RoleTect/diagram_mermaid_markdown.png" alt="Mermaid Technical Diagramming Canvas" /></td>
  </tr>
</table>

---

## ⚡ What is RoleTect?

RoleTect integrates sovereign LLM orchestration with professional TeX typesetting. Built for engineers and professionals who treat their career narrative as a precision specification:

* 🔒 **Local-First & Privacy Sovereign**: Your resume source files, parsed applications, and personal data remain stored in an encrypted local SQLite vault on your device.
* 🤖 **Multi-Provider AI Tailoring**: Tailor your resume content to any target job description using Gemini, OpenAI, Claude, Groq, AWS Bedrock, or completely offline with local **Ollama** models.
* 📄 **Built-in Tectonic LaTeX Compiler**: Compiles LaTeX documents directly into PDFs on-device in milliseconds without requiring heavy external TeX Live or MiKTeX distributions.
* 🧩 **Companion Browser Ingestion**: Capture job postings from LinkedIn, Indeed, Glassdoor, and career portals in a single click with the companion extension.
* 📊 **Self-Healing LaTeX & Side-by-Side Comparison**: Automatic syntax fixing and instant visual diffing between your baseline master resume and tailored documents.
* ☁️ **S3-Compatible Cloud Synchronization**: Automated, encrypted database backup and restore to any S3-compatible storage provider (AWS, Cloudflare R2, MinIO).
* 🎨 **Technical Diagramming Canvas**: Built-in interactive Mermaid.js editor with AI-assisted diagram generation and SVG export.

---

## 🏗️Complete System Architecture

![RoleTect Architecture Diagram](./assets/RoleTect_Diagram.jpg)

---

## 📥 Download & Installation

Pre-built standalone packages for Linux, macOS, and Windows are available directly on the [**Releases**](https://github.com/AhmedTrooper/RoleTect/releases/latest) page.

### Quick Installers

#### Linux & macOS (Terminal)
```bash
curl -fsSL https://raw.githubusercontent.com/AhmedTrooper/RoleTect/main/desktop/install.sh | bash
```

#### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/AhmedTrooper/RoleTect/main/desktop/install.ps1 | iex
```

#### Linux (Flatpak)
```bash
flatpak install --user roletect-0.1.5.flatpak
```

### Companion Browser Extension

* **Firefox**: [Install from Mozilla Add-ons (Available Now)](https://addons.mozilla.org/en-US/firefox/addon/roletect-ingest/)
* **Chrome / Chromium / Brave / Edge**: Download `roletect-chrome-extension.zip` from [Releases](https://github.com/AhmedTrooper/RoleTect/releases/latest), unpack it, open `chrome://extensions`, enable **Developer Mode**, and click **Load unpacked**.

---

## 🛠️ Development & Building from Source

### Prerequisites

* [Bun](https://bun.sh/) (v1.2+)
* [Rust & Cargo](https://www.rust-lang.org/) (Stable)
* C compiler and CMake (for Tectonic build dependencies)

### Running the Desktop App Locally

```bash
# Clone the official repository
git clone https://github.com/AhmedTrooper/RoleTect.git
cd RoleTect/desktop

# Install frontend dependencies with Bun
bun install

# Run desktop app in development mode (Tauri v2 + Vue 3)
bun run tauri dev
```

### Production Build

```bash
cd RoleTect/desktop

# Build frontend and compile release binary
bun run build
bun run tauri build
```

---

## 📄 License & Authors

© 2025–2026 **MD. RAMJAN MIAH (AHMEDTROOPER)**. All Rights Reserved.  
Commercial Proprietary software with open ecosystem companion tools.
