FROM scratch

# Copy the static binary from build artifacts
# The TARGETARCH arg is automatically set by Docker buildx (amd64 or arm64)
ARG TARGETARCH
COPY artifacts/${TARGETARCH}/kustomizer /usr/local/bin/kustomizer

ENTRYPOINT ["/usr/local/bin/kustomizer"]
