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

This project's architecture is fully decoupled, making it perfectly suited for modern "best-of-breed" free-tier hosting platforms. 

### 1. Deploying the Database to Neon.tech

We recommend using [Neon.tech](https://neon.tech/) for a Serverless PostgreSQL database.

1. Create a new free tier project on Neon.
2. Retrieve your **Connection String** (it will look like `postgres://username:password@.../neondb`).
3. Run your migrations against this database locally:
   ```bash
   DATABASE_URL="<your-neon-connection-string>" diesel migration run
   ```

*(Alternatively, you can also use [Supabase](https://supabase.com/) following a similar process).*

### 2. Deploying the Backend to Render

[Render.com](https://render.com) provides native Rust build environments without needing a Dockerfile.

1. Create a new **Web Service** on Render and connect your GitHub repository.
2. Configure the build settings:
   - **Language/Environment**: Rust
   - **Build Command**: `cargo build --release -p uchat_server`
   - **Start Command**: `./target/release/api`
3. Under **Environment Variables**, add the following:
   - `API_DATABASE_URL`: *(Your Neon Connection String)*
   - `API_BIND`: `0.0.0.0:10000`
   - `PORT`: `10000`
   - `FRONTEND_URL`: `https://your-frontend-app.netlify.app` *(Must match your Netlify domain exactly, without a trailing slash)*
4. Click **Deploy**.

### 3. Deploying the Frontend to Netlify

[Netlify](https://www.netlify.com/) will automatically build the Trunk WebAssembly frontend and serve it over a global CDN.

1. Connect your repository to Netlify. Netlify will automatically detect the `netlify.toml` file and its build configuration.
2. Go to **Site Settings > Environment Variables** and add:
   - `API_URL`: `https://your-backend-app.onrender.com/` *(Make sure to include the trailing slash if required by your API client)*
3. Trigger a manual deploy in the **Deploys** tab to ensure the environment variable is injected during the build process.

*(Note: The `netlify.toml` file is configured to create an empty `.env` file during the build process to satisfy the `load_dotenv!()` macro requirements without needing to commit your secrets).*
