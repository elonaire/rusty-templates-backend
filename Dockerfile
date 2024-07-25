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
# FROM ubuntu:latest
# ARG DEBIAN_FRONTEND=noninteractive
# ARG SERVICE_NAME
# ARG PORT

# ENV SERVICE_NAME=${SERVICE_NAME}
# ENV PORT=${PORT}

# # # Copy the binary from the builder stage
# COPY --from=0 /app/target/release/${SERVICE_NAME} .

# # # Expose the port
# EXPOSE ${PORT}
# # # Command to run
# CMD ./${SERVICE_NAME}

# Alpine
FROM alpine:latest
ARG DEBIAN_FRONTEND=noninteractive
ARG SERVICE_NAME
ARG PORT

ENV SERVICE_NAME=${SERVICE_NAME}
ENV PORT=${PORT}

# Install required dependencies on Alpine
RUN apk add --no-cache libc6-compat

# # Copy the binary from the builder stage
COPY --from=0 /app/target/release/${SERVICE_NAME} .

# # Expose the port
EXPOSE ${PORT}
# # Command to run
CMD ./${SERVICE_NAME}
