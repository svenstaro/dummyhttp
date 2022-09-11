FROM docker.io/ubuntu
COPY --chmod=755 dummyhttp /app/
ENTRYPOINT ["/app/dummyhttp"]
