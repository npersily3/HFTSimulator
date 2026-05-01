# syntax=docker/dockerfile:1

# Use a small Node.js image based on Alpine Linux.
# Alpine is much smaller than the default Debian-based image (~50 MB vs ~300 MB).
FROM node:20-alpine

WORKDIR /app

# Copy package.json first and install dependencies before copying the rest.
# Docker builds each instruction as a separate layer and caches them.
# Because package.json changes less often than source code, this layer is
# only re-run when dependencies actually change — making rebuilds much faster.
COPY package.json .
RUN npm install --omit=dev

# Copy the server source and the static HTML page
COPY server.js .
COPY public/ public/

# The port server.js listens on (must match compose.yaml)
EXPOSE 8000

CMD ["node", "server.js"]
