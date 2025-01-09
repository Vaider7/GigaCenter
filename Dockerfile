FROM centos/devtoolset-7-toolchain-centos7
 
ENV RUSTUP_VER="1.27.1" \
    RUST_ARCH="x86_64-unknown-linux-gnu" \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

RUN curl "https://static.rust-lang.org/rustup/archive/${RUSTUP_VER}/${RUST_ARCH}/rustup-init" -o rustup-init && \
    chmod +x rustup-init && \
    ./rustup-init -y --default-toolchain stable --profile minimal --no-modify-path && \
    rm rustup-init

ENV PATH=/opt/app-root/src/.cargo/bin:$PATH \
    RUSTUP_HOME=/opt/app-root/src/.rustup \
    CARGO_BUILD_TARGET=x86_64-unknown-linux-gnu
 
USER root
WORKDIR /app

ADD Cargo.toml .
ADD Cargo.lock .
ADD build.rs .
ADD src src
ADD assets assets
RUN mkdir target && chmod -R 777 target

CMD cargo build --release && cp ./target/x86_64-unknown-linux-gnu/release/gigacenter /volume
