FROM scratch
ADD target/x86_64-unknown-linux-musl/release/checkout /checkout
CMD ["/checkout"]
