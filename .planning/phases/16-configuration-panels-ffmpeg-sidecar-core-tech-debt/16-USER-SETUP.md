# Phase 16: User Setup Required

**Generated:** 2026-05-14
**Phase:** 16-configuration-panels-ffmpeg-sidecar-core-tech-debt
**Status:** Incomplete

Complete these items for FFmpeg sidecar packaging across all target platforms. The Windows x64 sidecar binaries were copied locally during Plan 16-01 and tracked through Git LFS.

## Environment Variables

None.

## Dashboard Configuration

None.

## Local Binary Setup

- [x] **Windows x64 sidecar binaries**
  - Location: `src-tauri/binaries/`
  - Files:
    - `ffmpeg-x86_64-pc-windows-msvc.exe`
    - `ffprobe-x86_64-pc-windows-msvc.exe`
  - Source used during execution: `C:\ProgramData\chocolatey\lib\ffmpeg\tools\ffmpeg\bin\`

- [ ] **Add non-Windows target binaries before packaging those platforms**
  - Required naming pattern:
    - `ffmpeg-x86_64-apple-darwin`
    - `ffprobe-x86_64-apple-darwin`
    - `ffmpeg-aarch64-apple-darwin`
    - `ffprobe-aarch64-apple-darwin`
    - `ffmpeg-x86_64-unknown-linux-gnu`
    - `ffprobe-x86_64-unknown-linux-gnu`
  - Place all files in `src-tauri/binaries/`.

- [ ] **Ensure Git LFS is configured on the remote**
  - Local rule added: `src-tauri/binaries/** filter=lfs diff=lfs merge=lfs -text`
  - Run `git lfs install` on machines that clone or package the app.

## Verification

After completing setup, verify with:

```powershell
git lfs status
cd src-tauri
cargo check
```

Expected results:
- Sidecar binaries are shown as LFS-managed files.
- `cargo check` passes for the target platform.

---

**Once all target binaries and remote LFS setup are complete:** Mark status as "Complete" at top of file.
