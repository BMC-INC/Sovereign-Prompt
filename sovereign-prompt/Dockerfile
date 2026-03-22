FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/sovereign-prompt /usr/local/bin/sovereign-prompt

ENV SOVEREIGN_MCP_TRANSPORT=sse
ENV SOVEREIGN_MCP_SSE_ADDR=0.0.0.0:8790
ENV SOVEREIGN_DASHBOARD_ADDR=0.0.0.0:8787

EXPOSE 8790 8787

ENTRYPOINT ["sovereign-prompt"]
