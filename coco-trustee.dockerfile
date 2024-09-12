FROM confidentialfilesystems/coco-trustee-builder:v1.0.0 AS builder

WORKDIR /usr/src
COPY . coco-trustee
RUN --mount=type=ssh cd coco-trustee && cargo install --locked --path src/kbs --no-default-features --features coco-as-builtin,resource,rustls,opa

RUN mkdir -p coco-trustee/cfs-kbs/lib && find coco-trustee/target/release -name libcfs.so -print0 | xargs -0 -I {} cp {} coco-trustee/cfs-kbs/lib/
RUN cp coco-trustee/target/release/kbs coco-trustee/cfs-kbs/

FROM debian:stable-slim

ENV LD_LIBRARY_PATH /cfs-kbs/lib

COPY --from=builder /usr/src/coco-trustee/cfs-kbs /cfs-kbs
COPY --from=builder /usr/src/coco-trustee/cfs-kbs/default_resource_policy.rego /opa/confidential-containers/kbs/policy.rego

COPY --from=builder /usr/share/ca-certificates /usr/share/ca-certificates
COPY --from=builder /usr/bin/update-alternatives /usr/bin/update-alternatives

COPY --from=builder /lib/x86_64-linux-gnu/libtdx_attest.so.1.21.100.3 /lib/x86_64-linux-gnu/libtdx_attest.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libsgx_dcap_quoteverify.so.1.13.101.3 /lib/x86_64-linux-gnu/libsgx_dcap_quoteverify.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libtss2-esys.so.0.0.0 /lib/x86_64-linux-gnu/libtss2-esys.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libtss2-tctildr.so.0.0.0 /lib/x86_64-linux-gnu/libtss2-tctildr.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libtss2-mu.so.0.0.0 /lib/x86_64-linux-gnu/libtss2-mu.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/x86_64-linux-gnu/libcrypto.so.3
COPY --from=builder /lib/x86_64-linux-gnu/libtss2-sys.so.1 /lib/x86_64-linux-gnu/libtss2-sys.so.1

WORKDIR /cfs-kbs
RUN mkdir -p /opt/confidential-containers/kbs/repository
ENTRYPOINT ["/cfs-kbs/kbs", "--config-file", "/cfs-kbs/kbs-config.toml"]