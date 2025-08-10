# Use the official Rust image as a base image
FROM rust:latest

# Set the working directory in the container
WORKDIR /usr/src/is_subsequence

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

# Copy the src directory to the container
COPY src ./src

# Run the tests
CMD ["cargo", "test"]
