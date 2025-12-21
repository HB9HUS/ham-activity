FROM rust:1-slim AS builder

WORKDIR /app

# First copy only the files Cargo needs to resolve the dependency
# graph.  This layer is cached as long as Cargo.toml / Cargo.lock
# stay unchanged, so rebuilding after a source change is fast.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs   # dummy file to allow cargo fetch
RUN cargo fetch

# Now copy the actual source code.
COPY src/ ./src/
COPY static/ ./static/
# Build a *release* binary (optimised, stripped).  If the project
# produces a library instead of a binary, change `--bin <name>` to
# `--lib` or the appropriate target.
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

# Create a non‑root user for the final container.
ARG USER=appuser
ARG UID=10001
RUN groupadd -r "${USER}" && useradd -r -g "${USER}" -u "${UID}" "${USER}"

# Switch to that user.
WORKDIR /app

COPY --from=builder /app/target/release/ham-activity ./ham-activity
COPY --from=builder /app/static ./static
RUN chown ${USER}:${USER} ham-activity

USER ${USER}:${USER}
EXPOSE 8000

ENTRYPOINT ["/app/ham-activity"]
