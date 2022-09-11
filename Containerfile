FROM scratch
COPY --chmod=755 dummyhttp /app/
ENTRYPOINT ["/app/dummyhttp"]
