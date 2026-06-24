# RoleTect

RoleTect is a local-first, privacy-focused desktop application and companion browser extension designed to organize the job application process, parse job descriptions, and tailor LaTeX-based resumes and cover letters using on-device or API-driven AI models.

---

## 🏗️ System Architecture

The following diagram illustrates how the components of RoleTect interact, from web scraping to secure key retrieval, database storage, and local compilation.

```mermaid
flowchart TD
    subgraph Browser ["Web Browser"]
        A[Job Posting Webpage] -->|1. Scrape & Compress DOM| B[Extension Content Script]
        B -->|2. POST JSON + Secret Key| C[Extension Background Worker]
    end

    subgraph DesktopApp ["RoleTect (Tauri Desktop App)"]
        C -->|3. Send Job Data| D[Local Axum Server]
        D -->|4. Authenticate Secret| E[Tauri AppState]
        E -->|5. Store Pending Job| F[(Local SQLite DB)]
        
        G[Vue 3 Frontend UI] -->|6. Trigger Parsing / Tailoring| H[Tauri Command Handlers]
        
        H -->|7. Request Decryption| I[IOTA Stronghold Vault]
        I -->|8. Return Decrypted Key to Memory| H
        
        H -->|9. Run LLM Ingestion / Tailoring| J[Rig AI Library]
        J -->|10. Execute API Call| K[AI Providers]
        
        H -->|11. Compile LaTeX Document| L[Tectonic Engine]
        L -->|12. Output PDF Output| M[Local File System]
    end

    classDef component fill:#2a2f3d,stroke:#4e5a75,stroke-width:1px,color:#fff;
    classDef storage fill:#1e2430,stroke:#3b475c,stroke-width:2px,color:#fff;
    class Browser,DesktopApp component;
    class F,I storage;
```

---

## 🎯 Core Features & Technical Implementation

### 1. Zero-Trust API Key Storage (IOTA Stronghold Enclave)
*   **The Issue:** Storing API keys in plaintext files or standard databases leaves them vulnerable to extraction by local malware.
*   **The Solution:** RoleTect utilizes `tauri-plugin-stronghold` to create an encrypted database enclave (`secrets.stronghold`) utilizing Argon2 key derivation.
*   **Runtime Security:** A cryptographically random 256-bit passkey is generated at first run and stored locally. API credentials are decrypted on-the-fly inside Rust command memory only when executing AI calls, and are never saved to the SQLite database.

### 2. Secure Local Ingest Server (Axum Core)
*   **Dynamic Port Bindings:** At startup, the Tauri backend spins up a local Axum server. It tests a port range (from `14207` to `14280`) to avoid port conflicts and registers the active port in the local configuration.
*   **Authentication:** The browser extension communicates with the Axum server using a dynamic 32-character secret key generated with `nanoid`. Requests lacking this secret in the JSON payload are rejected with a `401 Unauthorized` status.

### 3. Token-Squashing Scraper Pipeline
*   **Efficiency:** Raw HTML bloats token counts and incurs unnecessary API costs.
*   **DOM Sanitation:** The extension cleans the webpage DOM before transmission, stripping scripts, CSS styles, images, inline SVGs, iframes, navbars, and buttons.
*   **Text Processing:** The content script runs a regex pipeline to collapse vertical spacing, convert returns into periods, eliminate horizontal gaps, and squash duplicate periods.

### 4. On-Device Tectonic LaTeX Compilation
*   **Architecture:** To bypass the requirement of a system-wide LaTeX installation (such as TeX Live or MiKTeX), RoleTect embeds the **Tectonic compiler** directly into a background thread.
*   **Compilation Isolation:** PDF compilation runs in a dedicated thread stack sized to 10MB to handle complex LaTeX structures, utilizing a temporary directory to manage intermediate files and cache output dynamically.

### 5. AI-Assisted Technical Diagramming Workspace
*   **Interactive Visual Canvas:** Incorporates a dedicated workspace for creating, editing, and rendering Mermaid.js flowcharts, sequence diagrams, and ER diagrams.
*   **Interactive Panning & Zooming:** Uses `svg-pan-zoom` to allow users to interact with large, complex layouts.
*   **AI Synthesis & Repair:** Features specialized commands (`refine_diagram_with_ai`, `fix_diagram_with_ai`) that use AI to dynamically build, refine, and debug diagrams from conversational natural language instructions.

### 6. Multi-Provider LLM Orchestration
*   **Implementation:** Using the Rust-based `rig` AI library, the application integrates with Gemini, OpenAI, Anthropic, Groq, and Bedrock.
*   **Local Inference Support:** Supports Ollama, enabling users to keep both their data and LLM operations entirely offline.

---

## 📂 Project Structure

```text
├── application/             # Standalone assets & icons
├── src-tauri/               # Tauri Rust Backend
│   ├── src/
│   │   ├── commands/        # Tauri command handlers (settings, compiler, database)
│   │   ├── ai.rs            # Rig LLM integration & prompt templates
│   │   ├── db.rs            # rusqlite database initialization and migrations
│   │   ├── lib.rs           # Tauri app setup & plugin registration
│   │   └── server.rs        # Axum local server
│   └── Cargo.toml           # Rust backend dependencies
├── src/                     # Vue 3 Frontend
│   ├── components/          # Views & UI Components (Editor, Job Tracker, Settings)
│   ├── store/               # Pinia State Management (Settings, Jobs, Resumes)
│   └── App.vue              # Main App wrapper
├── extentions/              # Browser Extensions
│   ├── chrome/              # Manifest V3 extension code
│   └── firefox/             # Firefox compatible manifest extension code
└── README.md
```

---

## 🚀 Setup & Local Execution

### Prerequisites
*   [Node.js](https://nodejs.org/) (v18+)
*   [Rust Compiler & Cargo](https://www.rust-lang.org/)
*   [Bun Package Manager](https://bun.sh/) (Recommended, or npm)

### Backend & Frontend Setup
1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/AhmedTrooper/RoleTect.git
    cd RoleTect
    ```
2.  **Install Dependencies:**
    ```bash
    bun install
    # or
    npm install
    ```
3.  **Run Development Environment:**
    ```bash
    bun run tauri dev
    # or
    npm run tauri dev
    ```

### Extension Installation
1.  Open your browser and navigate to the extensions page (e.g., `chrome://extensions` in Chrome).
2.  Enable **Developer Mode** (toggle in the top-right corner).
3.  Click **Load unpacked** and select the `extentions/chrome` folder (or `extentions/firefox` for Firefox).
4.  Copy the **Secret Key** and **Active Server Port** from the *Inbox* or *Settings* tab in the RoleTect desktop app and paste them into the extension settings popup.

---

## 📊 Model & Data Card

### 1. Model Summary
*   **Recommended Models:** `gemini-1.5-pro` / `gemini-1.5-flash` (via Google AI Studio), `gpt-4o` / `gpt-4o-mini` (via OpenAI), or `llama3-70b-8192` (via Groq).
*   **Local Inference:** Compatible with any GGUF running via `ollama` locally (e.g., `llama3:8b`).
*   **AI Roles:**
    *   **Parser:** Evaluates raw text/URL crawled data and maps it to a JSON schema defining validation properties (job details, responsibilities, requirements).
    *   **Tailoring Engine:** Injects matching skills and structural details directly into LaTeX resume structures while conserving compilability.
    *   **Debugger:** Ingests Tectonic error logs and broken code strings to resolve LaTeX syntax errors.

### 2. Data Processing Policy
*   **Zero-Cloud Storage:** RoleTect operates no remote application servers. Job details, resumes, templates, and application progress remain in a local SQLite file.
*   **Inference Path:** Data sent to LLMs is routed directly from the desktop client to the selected API provider endpoint via HTTPS. No third-party relays are used.

### 3. Limitations & Considerations
*   **Initial Compiler Latency:** On its first run, the Tectonic engine downloads compiler assets to compile document structures. Subsequent runs utilize the local Tectonic cache.
*   **Offline Compilation:** While LaTeX PDF generation is 100% offline, AI-based parsing and tailoring require a network connection unless a local model is running via Ollama.

---

## 🔓 Open-Source Attribution & Licensing Directory

In compliance with **Section 10.2** of the SciBlitz AI Challenge 2026 Rulebook, below is the comprehensive attribution list of third-party open-source libraries, engines, and AI models utilized in the development of RoleTect.

### 1. Backend Core & Plugins (Rust)
*   **Tauri v2 Framework** (MIT / Apache-2.0) | Cross-platform runtime orchestration.
*   **Tectonic Engine** (MIT) | Self-contained LaTeX-to-PDF compiler.
*   **Rig AI Framework** (MIT) | Declarative agent structures & LLM connector.
*   **Axum & Tokio** (MIT) | Non-blocking web server and async execution runtime.
*   **IOTA Stronghold** (Apache-2.0 / MIT) | Argon2 secure local database enclave.
*   **rusqlite** (MIT) | Native SQLite wrapper for local relational tracking.
*   **tower-http** (MIT) | CORS and network request middleware layers.

### 2. Frontend Application (Vue 3 / TypeScript)
*   **Vue 3** (MIT) | Declarative user interface runtime.
*   **Vite** (MIT) | Frontend bundler and build system.
*   **Pinia** (MIT) | Global state stores for configurations and workspace files.
*   **CodeMirror** (MIT) | In-browser LaTeX editor canvas.
*   **DOMPurify** (Apache-2.0) | HTML sanitization preventing XSS during ingestion.
*   **Mermaid.js** (MIT) | Client-side markup diagram renderer.
*   **Motion-V** (MIT) | UI transition and micro-animation engine.
*   **svg-pan-zoom** (MIT) | Interactive PDF preview panning tools.

### 3. Pre-trained AI Models (Third-party)
*   **Gemini 1.5 Pro & Flash** (Google AI Studio Terms of Service) | Used for structured job schema parsing and resume template tailoring.
*   **GPT-4o & GPT-4o-mini** (OpenAI APIs Terms of Service) | Supported alternative routing models.
*   **Claude 3.5 Sonnet** (Anthropic API Terms of Service) | High-fidelity LaTeX syntax debugging.
*   **Llama 3 (8B/70B)** (Meta LLaMA 3 License Agreement) | Local inference compatible model via Ollama.
