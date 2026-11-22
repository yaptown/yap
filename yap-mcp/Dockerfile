# Build with the repo root as context: `flyctl deploy --config yap-mcp/fly.toml`
# Pin the Rust version (keep in sync with rust-toolchain.toml) AND the distro —
# builder and runtime must be the same distro or the binary hits glibc
# mismatches at startup (see yap-ai-backend/Dockerfile for the war story).
FROM lukemathwalker/cargo-chef:latest-rust-1.96-trixie AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
# >>> cargo xtask dockerfiles: manifests
COPY yap-mcp/Cargo.toml yap-mcp/Cargo.toml
COPY language-utils/Cargo.toml language-utils/Cargo.toml
COPY libraries/audio-codec/Cargo.toml libraries/audio-codec/Cargo.toml
COPY libraries/autograde-core/Cargo.toml libraries/autograde-core/Cargo.toml
COPY libraries/bridgerton/Cargo.toml libraries/bridgerton/Cargo.toml
COPY libraries/bridgerton/macros/Cargo.toml libraries/bridgerton/macros/Cargo.toml
COPY libraries/eyedee/Cargo.toml libraries/eyedee/Cargo.toml
COPY libraries/imdex_map/Cargo.toml libraries/imdex_map/Cargo.toml
COPY libraries/knowledge-graph/Cargo.toml libraries/knowledge-graph/Cargo.toml
COPY libraries/movie-subtitles/Cargo.toml libraries/movie-subtitles/Cargo.toml
COPY libraries/omnigram/Cargo.toml libraries/omnigram/Cargo.toml
COPY libraries/pack-builder/Cargo.toml libraries/pack-builder/Cargo.toml
COPY libraries/weapon/Cargo.toml libraries/weapon/Cargo.toml
COPY libraries/yap-i18n/Cargo.toml libraries/yap-i18n/Cargo.toml
COPY yap-frontend-reducers/Cargo.toml yap-frontend-reducers/Cargo.toml
COPY yap-frontend-rs/Cargo.toml yap-frontend-rs/Cargo.toml
# <<< cargo xtask dockerfiles
# Trim the workspace to yap-mcp's path-dependency closure. The blocks marked
# `cargo xtask dockerfiles` are generated from cargo metadata: edit Cargo.toml
# and rerun `cargo xtask dockerfiles` rather than these lists.
# >>> cargo xtask dockerfiles: members
RUN sed -i '/^members = \[/,/^\]/c\members = [\n    "yap-mcp",\n    "language-utils",\n    "libraries/audio-codec",\n    "libraries/autograde-core",\n    "libraries/bridgerton",\n    "libraries/bridgerton/macros",\n    "libraries/eyedee",\n    "libraries/imdex_map",\n    "libraries/knowledge-graph",\n    "libraries/movie-subtitles",\n    "libraries/omnigram",\n    "libraries/pack-builder",\n    "libraries/weapon",\n    "libraries/yap-i18n",\n    "yap-frontend-reducers",\n    "yap-frontend-rs",\n]' Cargo.toml
# <<< cargo xtask dockerfiles
# cargo-chef needs stub files for every target declared in the manifests
# >>> cargo xtask dockerfiles: stubs
RUN mkdir -p yap-mcp/src \
    && touch yap-mcp/src/lib.rs \
    && echo "fn main() {}" > yap-mcp/src/main.rs
RUN mkdir -p language-utils/examples language-utils/src language-utils/tests \
    && touch language-utils/src/lib.rs \
    && echo "fn main() {}" > language-utils/examples/dump_sentence_phonemes.rs \
    && echo "fn main() {}" > language-utils/examples/pack_sizes.rs \
    && echo "fn main() {}" > language-utils/examples/verify_pack_hashes.rs \
    && echo "fn main() {}" > language-utils/tests/whitespace.rs
RUN mkdir -p libraries/audio-codec/src \
    && touch libraries/audio-codec/src/lib.rs
RUN mkdir -p libraries/autograde-core/src \
    && touch libraries/autograde-core/src/lib.rs
RUN mkdir -p libraries/bridgerton/src libraries/bridgerton/tests \
    && touch libraries/bridgerton/src/lib.rs \
    && echo "fn main() {}" > libraries/bridgerton/tests/default_generics.rs \
    && echo "fn main() {}" > libraries/bridgerton/tests/platform.rs \
    && echo "fn main() {}" > libraries/bridgerton/tests/tokio_context.rs
RUN mkdir -p libraries/bridgerton/macros/src \
    && touch libraries/bridgerton/macros/src/lib.rs
RUN mkdir -p libraries/eyedee/src \
    && touch libraries/eyedee/src/lib.rs
RUN mkdir -p libraries/imdex_map/src \
    && touch libraries/imdex_map/src/lib.rs
RUN mkdir -p libraries/knowledge-graph/src \
    && touch libraries/knowledge-graph/src/lib.rs
RUN mkdir -p libraries/movie-subtitles libraries/movie-subtitles/src \
    && touch libraries/movie-subtitles/src/lib.rs \
    && echo "fn main() {}" > libraries/movie-subtitles/build.rs
RUN mkdir -p libraries/omnigram/src \
    && touch libraries/omnigram/src/lib.rs
RUN mkdir -p libraries/pack-builder/src \
    && touch libraries/pack-builder/src/lib.rs
RUN mkdir -p libraries/weapon/src libraries/weapon/src/bin \
    && touch libraries/weapon/src/lib.rs \
    && echo "fn main() {}" > libraries/weapon/src/bin/weapon-scope.rs
RUN mkdir -p libraries/yap-i18n/src libraries/yap-i18n/tests \
    && touch libraries/yap-i18n/src/lib.rs \
    && echo "fn main() {}" > libraries/yap-i18n/tests/catalogs.rs
RUN mkdir -p yap-frontend-reducers yap-frontend-reducers/src \
    && touch yap-frontend-reducers/src/lib.rs \
    && echo "fn main() {}" > yap-frontend-reducers/build.rs
RUN mkdir -p yap-frontend-rs/benches yap-frontend-rs/examples yap-frontend-rs/src yap-frontend-rs/tests \
    && touch yap-frontend-rs/src/lib.rs \
    && echo "fn main() {}" > yap-frontend-rs/examples/profile_simulation.rs \
    && echo "fn main() {}" > yap-frontend-rs/tests/web.rs \
    && echo "fn main() {}" > yap-frontend-rs/benches/profile_next_cards.rs \
    && echo "fn main() {}" > yap-frontend-rs/benches/simulation.rs
# <<< cargo xtask dockerfiles
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/Cargo.toml Cargo.toml
COPY Cargo.lock .
# Build dependencies - this is the caching Docker layer! Filter to yap-mcp so
# the trimmed workspace's other members don't enable features the final
# build never uses (audio-codec's default `full` needs libopus/cmake).
RUN cargo chef cook --release --recipe-path recipe.json --package yap-mcp
# out/ before the sources: yap-frontend-rs include_str!s the showcase.json and
# language_data.hash files, and this layer (rkyv packs included) is huge and
# rarely changes — keep it above the every-commit source layers.
COPY out out
# >>> cargo xtask dockerfiles: sources
COPY yap-mcp yap-mcp
COPY language-utils language-utils
COPY libraries/audio-codec libraries/audio-codec
COPY libraries/autograde-core libraries/autograde-core
COPY libraries/bridgerton libraries/bridgerton
COPY libraries/eyedee libraries/eyedee
COPY libraries/imdex_map libraries/imdex_map
COPY libraries/knowledge-graph libraries/knowledge-graph
COPY libraries/movie-subtitles libraries/movie-subtitles
COPY libraries/omnigram libraries/omnigram
COPY libraries/pack-builder libraries/pack-builder
COPY libraries/weapon libraries/weapon
COPY libraries/yap-i18n libraries/yap-i18n
COPY yap-frontend-reducers yap-frontend-reducers
COPY yap-frontend-rs yap-frontend-rs
# <<< cargo xtask dockerfiles
# Drop target/ within the same layer: it's ~hundreds of MB of incremental
# artifacts that change every commit, and mode=max cache export would ship
# all of it to the registry each deploy.
RUN cargo build --release -p yap-mcp \
    && cp target/release/yap-mcp /app/yap-mcp-bin \
    && rm -rf target

FROM debian:trixie-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y \
    libssl3t64 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/yap-mcp-bin /usr/local/bin/yap-mcp
# Language packs are read from disk at runtime (lazily, per course).
COPY out /app/out
ENV YAP_OUT_DIR=/app/out
# The review widget is built by CI (pnpm, outside Docker) and copied in;
# the server serves it as the MCP Apps ui:// resource.
COPY yap-mcp/widget/dist/index.html /app/widget/index.html
ENV YAP_WIDGET_HTML=/app/widget/index.html
ENTRYPOINT ["/usr/local/bin/yap-mcp", "serve"]
