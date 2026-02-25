# Dev workflow commands for ryguessr

set dotenv-load := true

# List available recipes
default:
    @just --list

# Run frontend and backend concurrently with auto-reload
dev:
    just web/install && just web/build && just web/watch & just server/watch & wait

# Run continuous integration suite
ci:
    just server/build
    just server/test
    just server/clippy
    just server/fmt-check

    just web/build
    just web/lint

    @echo "Done!"
