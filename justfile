TRUNK_CONFIG_FILE := if os() == "windows" { "Trunk.win.toml" } else { "Trunk.toml" }
TRUNK_RELEASE_CONFIG_FILE := if os() == "windows" { "Trunk-release.win.toml" } else { "Trunk.toml" }

# build in release mode
build:
    # build frontend
    trunk --config {{TRUNK_RELEASE_CONFIG_FILE}} build
    # build backend
    cargo build --release --workspace --exclude frontend

# run cargo check
check:
    cargo check -p frontend --target wasm32-unknown-unknown
    cargo check --workspace --exclude frontend

# run cargo clippy
clippy:
    cargo clippy -p frontend --target wasm32-unknown-unknown
    cargo clippy --workspace --exclude frontend

# run clippy fix
fix:
    cargo clippy -p frontend --fix --target wasm32-unknown-unknown
    cargo clippy --workspace --fix --exclude frontend

# build docs. use --open to open in browser
doc *ARGS:
    cargo doc -F docbuild {{ ARGS }}

# run frontend devserver. use --open to open a new browser
serve-frontend *ARGS:
    trunk --config {{TRUNK_CONFIG_FILE}} serve {{ ARGS }}

# run API server
serve-api *ARGS:
    watchexec -r -i "frontend/**" -i "target/**" --exts rs,sql,toml cargo run -p uchat_server {{ ARGS }}

# run both frontend and API server
serve-all:
    just serve-frontend &
    just serve-api
        
# set up project dependencies
init:
    cargo run -p project-init
    cd frontend && npm install

# migration related commands
# apply migrations
db-migrate:
    diesel migration run
    # test migration
    diesel migration redo
    psql -d postgres -c 'DROP DATABASE uchat_test;'

# reset the database
db-reset:
    diesel database reset
    psql -d postgres -c 'DROP DATABASE uchat_test;' || true

# create a new database migration
db-new-migration NAME:
    diesel migration generate {{ NAME }}

# run tests with cargo nextest
test:
    cargo nextest run

# run production frontend server
serve-pro-frontend:
    trunk serve --release --config {{TRUNK_RELEASE_CONFIG_FILE}}

# run production API server
serve-pro-api:
    cd target/release && ./api