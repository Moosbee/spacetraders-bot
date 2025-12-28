FROM rust:1.88.0 AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

COPY ./ ./

# Build your application
RUN cargo build --release

# Start a new stage to create a smaller image without unnecessary build dependencies
FROM rust:1.88.0

# Set the working directory
WORKDIR /usr/src/app

# Copy the built binary from the previous stage
COPY --from=builder /usr/src/app/target/release/spacetraders /usr/src/app/

# Command to run the application
CMD ["./spacetraders"]
