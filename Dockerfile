# Stage 1: Build the WASM app with Trunk
FROM rust:1.86 AS builder

# Install wasm target and trunk (trunk downloads wasm-bindgen-cli automatically)
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked

WORKDIR /app
COPY . .

# Build the app
RUN cd crates/app && trunk build --release

# Stage 2: Serve with nginx
FROM nginx:alpine

# Copy the built static files
COPY --from=builder /app/crates/app/dist /usr/share/nginx/html

# SPA fallback: serve index.html for all routes
RUN echo 'server { \
    listen 8080; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
}' > /etc/nginx/conf.d/default.conf

EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
