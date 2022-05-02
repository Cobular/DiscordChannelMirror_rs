FROM rust:slim-buster AS builder

RUN update-ca-certificates

# Create appuser
ENV USER=discord_channel_mirror_rs
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /discord_channel_mirror_rs

# Build once with dummy data to cache build artifacts
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs
RUN cargo build --release
RUN rm -rf ./src

# Now copy our stuff and build again
COPY ./ .

RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM debian:buster-slim

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /discord_channel_mirror_rs

# Copy our build
COPY --from=builder /discord_channel_mirror_rs/target/release/discord_channel_mirror_rs ./

# Use an unprivileged user.
USER discord_channel_mirror_rs:discord_channel_mirror_rs

CMD ["/discord_channel_mirror_rs/discord_channel_mirror_rs"]