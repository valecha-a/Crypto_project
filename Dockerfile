# # Start from a base image that includes Rust and Cargo
# FROM rust:latest

# # Set the working directory inside the container
# WORKDIR /app

# # Install OpenSSL development package
# RUN apt-get update \
#     && apt-get install -y libssl-dev

# # Copy the Cargo.toml and Cargo.lock files to the working directory
# COPY Cargo.toml Cargo.lock ./

# # Copy the entire source code to the working directory
# COPY . .

# # Build your Rust application
# RUN cargo build --release

# # Expose the port on which your Rust application will listen
# EXPOSE 8080

# # Command to run your Rust application inside the container
# CMD ["cargo", "run", "--release"]

#working

# Start from a base image that includes Rust and Cargo
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Install OpenSSL development package (needed for tokio-postgres)
RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config

# Copy the Cargo.toml and Cargo.lock files to the working directory
COPY Cargo.toml Cargo.lock ./

# Copy the entire source code to the working directory
COPY . .

# Build your Rust application
RUN cargo build --release

# Expose the port on which your Rust application will listen
EXPOSE 8080

# Command to run your Rust application inside the container
CMD ["target/release/bitcoin_explorer_backend"]


# # Start from a base image that includes Rust and Cargo
# FROM rust:latest

# # Set the working directory inside the container
# WORKDIR /app

# # Install OpenSSL development package (needed for tokio-postgres)
# RUN apt-get update \
#     && apt-get install -y libssl-dev pkg-config

# # Copy the Cargo.toml and Cargo.lock files to the working directory
# COPY Cargo.toml Cargo.lock ./

# # Copy the entire source code to the working directory
# COPY . .

# # Build your Rust application
# RUN cargo build --release

# # Copy the wait-for-it.sh script
# COPY wait-for-it.sh /usr/local/bin/wait-for-it.sh
# RUN chmod +x /usr/local/bin/wait-for-it.sh

# # Expose the port on which your Rust application will listen
# EXPOSE 8080



