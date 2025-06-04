FROM node:24-slim AS pnpm

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

FROM pnpm AS ui-builder

WORKDIR /app

COPY html .
RUN pnpm i
RUN pnpm build


WORKDIR /app
COPY html .
RUN pnpm i
RUN pnpm build

FROM rust AS rs-builder
RUN rustup default nightly

WORKDIR /usr/src/app

COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=rs-builder /usr/local/cargo/bin/ifconfig-rs /usr/local/bin/ifconfig-rs
COPY --from=ui-builder /app/dist/_astro/*.css /srv/styles.min.css

EXPOSE 8080
CMD [ "ifconfig-rs", "--host" ]