FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY backend ./backend
RUN cd backend \
    && cargo build --release -p app \
    && rm -rf target/release/{build,deps,incremental} \
    && rm -rf /root/.cargo/registry /root/.cargo/git

FROM alpine:latest AS runtime
RUN apk add --no-cache ca-certificates tzdata chromium \
    && rm -rf /var/cache/apk/*
ENV CHROMIUM_PATH=/usr/bin/chromium-browser
WORKDIR /app
COPY --from=builder /build/backend/target/release/app ./app
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser
EXPOSE 8080
ENTRYPOINT ["./app"]

