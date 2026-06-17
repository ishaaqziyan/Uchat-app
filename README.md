# uChat

Uchat is a revolutionary full-stack chat app built completely on [Rust](https://www.rust-lang.org/), while leveraging the capabilities of [WASM](https://webassembly.org/).

<br>

![Uchat App Screenshot](https://github.com/ishaaqziyan/Uchat-app/assets/98882071/0c0621f0-cb36-4634-9790-e6a0a240f235)

<br>

Demo video is available [here](https://youtu.be/lsbjFcgJdTo)

<br>

## Design

The `design/` directory contains some design-related files and assets used throughout the project.

## Initial Setup

If you are on Windows, using [WSL](https://learn.microsoft.com/en-us/windows/wsl/install) is highly recommended to manage build dependencies and tooling.

### Rust

If you haven't installed Rust yet, you can do so using [rustup](https://rustup.rs/) and then install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Compiling Rust for the browser requires adding the `wasm32` compilation target:

```bash
rustup target add wasm32-unknown-unknown
```

### Database

This project uses [PostgreSQL](https://www.postgresql.org/) for the database.
Please follow the [official instructions](https://www.postgresql.org/download/) for how to install PostgreSQL on your system. Alternatively, you can use a managed database like Supabase (see the Deployment section below).

### Trunk

[Trunk](https://trunkrs.dev/) is a tool to build and bundle Rust WASM applications. Install it with:

```bash
cargo install --locked trunk

# Apple M1/M2/M3 users may also need to install this:
cargo install --locked wasm-bindgen-cli
```

### Diesel CLI

[Diesel](https://diesel.rs/) is a Rust SQL query builder and ORM for working with the database.

Ensure you have PostgreSQL installed locally (or its development libraries) before proceeding.

Install the Diesel CLI with:

```bash
cargo install diesel_cli --no-default-features --features postgres
```

> **Note:** If you receive build or linker errors, make sure you install `libpq`. This may be packaged separately depending on your operating system and package manager (e.g., `libpq-dev` on Ubuntu/WSL).

### Environment Variables & Database Setup

Create a `.env` file in the workspace directory containing:

```env
API_DATABASE_URL="postgres://DATABASE_USER:PASSWORD@localhost/uchatfinal"
API_PRIVATE_KEY="API_PRIVATE_KEY_HERE"
API_URL="http://127.0.0.1:8070/"
API_BIND="127.0.0.1:8070"

# Development only
DATABASE_URL="postgres://DATABASE_USER:PASSWORD@localhost/uchatfinal"
TEST_DATABASE_URL="postgres://DATABASE_USER:PASSWORD@localhost/uchatfinal_test"
FRONTEND_URL="http://127.0.0.1:8080"
```

Substitute the following:
- `DATABASE_USER`: Role created to access PostgreSQL
- `PASSWORD`: Your password to login to the database (omit `:PASSWORD` if not using a password)

After the `.env` is ready, run this command to create the database:

```bash
diesel setup
```

### npm & Tailwind CSS

This project uses [Tailwind CSS](https://tailwindcss.com/) for utility classes.
You will need [Node.js and npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) installed to use the `npx` command for Tailwind.

### (Optional) `just`

[just](https://github.com/casey/just) is a handy command runner which simplifies running different development commands. Install it with:

```bash
cargo install just
```

## Commands

### Project Initialization

This will check for the dependencies listed above and attempt to install the required Rust dependencies. It will provide installation links for dependencies that require manual installation.

```bash
cargo run -p project-init
```

### Documentation

To build the documentation:

```bash
cargo doc -F docbuild
```

*(Note: Enabling the `docbuild` feature is a temporary workaround for a minor bug in a transitive dependency.)*

### Development Server

To run a development server for the frontend and open a new browser window:

```bash
trunk serve --open
```

To run the backend server:

```bash
cargo run -p uchat_server
```

### Check / Clippy

Check the frontend and backend targets:

```bash
cargo check -p frontend --target wasm32-unknown-unknown
cargo check --workspace --exclude frontend
```

Run clippy for the frontend and backend targets:

```bash
cargo clippy -p frontend --target wasm32-unknown-unknown
cargo clippy --workspace --exclude frontend
```

### Build for Production

To build the project for distribution:

```bash
trunk --config Trunk-release.toml build
cargo build --release --workspace --exclude frontend
```

To run the production build locally:

**Frontend:**
```bash
trunk serve --release --config Trunk-release.toml
```

**Backend:**
Navigate to `target/release/` and run the executable:
```bash
cd target/release/
./api
```

### Migrations

To create new database migrations, run:

```bash
diesel migration generate MIGRATION_NAME
```

This creates a new folder in `data/migrations/timestamp_MIGRATION_NAME/`. Add your SQL for applying the migration to `up.sql` and the SQL for reverting it to `down.sql`.

Apply the migration with:

```bash
diesel migration run
```

To verify your `down.sql` works, run this command to revert and then reapply the migration:

```bash
diesel migration redo
```

After creating a new migration, you may want to delete the testing database:

```bash
psql -d postgres -c 'DROP DATABASE IF EXISTS uchat_test;'
```

## Deployment

### Deploying the Database to Supabase

[Supabase](https://supabase.com/) is an excellent open-source Firebase alternative that provides a managed PostgreSQL database.

1. Create a new project on [Supabase](https://supabase.com/).
2. Navigate to your Project Settings -> Database.
3. Under **Connection string**, select **URI**.
4. Copy the connection string. It should look something like:
   `postgresql://postgres.[YOUR_PROJECT_REF]:[YOUR_PASSWORD]@aws-0-[REGION].pooler.supabase.com:6543/postgres`
5. Replace `[YOUR_PASSWORD]` with your actual database password.
6. Use this connection string for your `API_DATABASE_URL` and `DATABASE_URL` in your production environment variables.
7. Run your migrations against the Supabase database:
   ```bash
   DATABASE_URL="<your-supabase-connection-string>" diesel migration run
   ```

### Deploying the Frontend to Netlify

[Netlify](https://www.netlify.com/) is a popular platform for hosting static sites and frontend frameworks. Since the frontend is compiled to WebAssembly via Trunk, it can be hosted as a static site.

1. Push your repository to GitHub, GitLab, or Bitbucket.
2. Log into [Netlify](https://app.netlify.com/) and click **Add new site** -> **Import an existing project**.
3. Connect your Git provider and select the `Uchat-app` repository.
4. Configure the build settings:
   - **Base directory:** Leave blank or set to the root directory where `Trunk-release.toml` is located.
   - **Build command:** Netlify needs to install the Rust toolchain, the `wasm32` target, and Trunk before building. Use the following build command:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && source "$HOME/.cargo/env" && rustup target add wasm32-unknown-unknown && cargo install --locked trunk && trunk --config Trunk-release.toml build --release
     ```
   - **Publish directory:** `dist` (or the output directory configured in your `Trunk-release.toml`).
5. Set your environment variables in Netlify (e.g., `FRONTEND_URL`, `API_URL` pointing to your deployed backend).
6. Click **Deploy site**.

*(Note: The backend (`uchat_server` or `api`) needs to be hosted on a platform that supports Rust binaries or Docker containers, such as Render, Railway, Fly.io, or AWS EC2, as Netlify only hosts the frontend static files.)*
