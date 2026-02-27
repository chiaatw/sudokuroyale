FROM mcr.microsoft.com/devcontainers/rust:1

RUN apt-get update && apt-get install -y \
    postgresql-client \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*