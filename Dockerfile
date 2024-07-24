FROM ubuntu:latest

ARG DEBIAN_FRONTEND=noninteractive

# Set the working directory to the service specified in the build argument
ARG SERVICE_NAME

ENV SERVICE_NAME=${SERVICE_NAME}

RUN apt update
RUN apt install -y build-essential \
    curl \
    pkg-config \
    libssl-dev

RUN apt update
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default nightly-2024-03-09

WORKDIR /app

# Copy the entire workspace
COPY . .

# Build for release
RUN cargo build --release --package ${SERVICE_NAME}

# Final stage: use a lightweight image
FROM ubuntu:latest
ARG DEBIAN_FRONTEND=noninteractive
ARG SERVICE_NAME
ARG PORT

ENV SERVICE_NAME=${SERVICE_NAME}
ENV PORT=${PORT}

# Install CA certificates in the final stage
RUN apt update && apt install -y ca-certificates && update-ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy CA certificates to both common locations
RUN ln -s /etc/ssl/certs /usr/lib/ssl/certs || true

# Set environment variables to specify the correct certificate locations
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/usr/lib/ssl/certs

# # Copy the binary from the builder stage
COPY --from=0 /app/target/release/${SERVICE_NAME} .

# # Expose the port
EXPOSE ${PORT}
# # Command to run
CMD ./${SERVICE_NAME}
