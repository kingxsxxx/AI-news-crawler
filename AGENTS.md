# Repository Guidelines

## Project Structure & Module Organization
- Frontend app lives in `src/`:
  - `src/main.tsx` boots React.
  - `src/App.tsx` is the root UI entry.
  - `src/styles/` stores global styles.
- Desktop backend lives in `src-tauri/`:
  - `src-tauri/src/main.rs` starts the Tauri runtime.
  - `src-tauri/src/lib.rs` contains Tauri command handlers.
  - `src-tauri/tauri.conf.json` defines app/build/window settings.
- Static assets go in `public/` (for example `public/images/`).
- Product docs are currently in root (`product design.md`, `Technical stack.md`).

## Build, Test, and Development Commands
- `npm install`: install frontend dependencies.
- `npm run dev`: run Vite dev server for frontend iteration.
- `npm run build`: type-check and build frontend assets into `dist/`.
- `cargo check --manifest-path src-tauri/Cargo.toml`: validate Rust/Tauri backend compiles.
- `cargo run --manifest-path src-tauri/Cargo.toml`: run backend directly (useful for command-layer debugging).

## Coding Style & Naming Conventions
- Use TypeScript with strict mode; avoid `any` unless documented.
- Indentation: 2 spaces for TS/JSON/CSS, 4 spaces for Rust.
- React components: `PascalCase` (e.g., `ArticleCard.tsx`).
- Hooks: `useXxx` naming in `src/hooks/`.
- Rust modules/functions: `snake_case`; types/enums: `PascalCase`.
- Prefer small, composable modules over large files.

## Testing Guidelines
- Frontend E2E: Playwright (planned baseline).
- Rust: unit tests next to modules (`mod tests`) and integration tests under `src-tauri/tests/`.
- Test file naming:
  - Frontend: `*.spec.ts` or `*.e2e.ts`.
  - Rust: descriptive `*_tests.rs` for integration suites.
- Minimum expectation for PRs: cover new logic paths and at least one failure/edge case.

## Commit & Pull Request Guidelines
- Use Conventional Commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`.
- Keep commits focused and atomic (one concern per commit).
- PRs should include:
  - Clear summary and scope.
  - Linked issue/task ID.
  - Screenshots/GIFs for UI changes.
  - Notes on config/schema changes and manual verification steps.

## Security & Configuration Tips
- Never commit secrets (API keys, tokens, local DB dumps).
- Keep local overrides in untracked env files.
- Validate all external content before persistence (crawler and manual URL ingestion paths).
