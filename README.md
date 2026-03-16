# Keptr

Keptr is a modern, zero-knowledge, high-performance secure vault designed to store and manage highly sensitive user data including passwords, documents, and auto-type records. 

## Features
- **End-to-End Encrypted Storage**: Fully offline-first local SQLite databases, augmented with XChaCha20-Poly1305 and Argon2id.
- **Secure File Backups**: Encrypted file attachments (`.kaps`) capable of automatic syncing.
- **Auto-Type**: Seamlessly type your passwords globally bypassing the clipboard via a Rust-level secure typing hook.
- **Continuous Session Access**: Automatic background JWT token refreshing to ensure uninterrupted connectivity to cloud backups.
- **Multi-Vault Support**: Seamlessly isolate distinct contexts.

## Tech Stack
- Frontend: React + TypeScript + Vite, Tailwind CSS + Radix UI
- Backend: Tauri + Rust
- Database: Sqlite / Encrypted Data Payload
- Sync Platform: Supabase

## Setup Instructions
Download the latest pre-compiled installer from our [Releases](#) tab, or clone and build from source:

1. Copy `.env.example` to `.env` and fill in your keys.
2. Run `npm install`
3. Run `npm run tauri dev`
