FROM confidentialfilesystems/cc/coco-trustee-builder:v1.0.0-amd64 AS builder

ARG TAG

WORKDIR /usr/src/kbs
RUN --mount=type=ssh git clone --depth 1 --branch ${TAG} git@github.com:confidential-filesystems/coco-trustee.git
RUN --mount=type=ssh cd coco-trustee && cargo install --locked --path src/kbs --no-default-features --features coco-as-builtin,resource,rustls,opa

RUN mkdir -p coco-trustee/cfs-kbs/lib && find coco-trustee/target/release -name libcfs.so -print0 | xargs -0 -I {} cp {} coco-trustee/cfs-kbs/lib/
RUN cp coco-trustee/target/release/kbs coco-trustee/cfs-kbs/

FROM confidentialfilesystems/cc/coco-trustee-base:v1.0.0-amd64

COPY --from=builder /usr/src/kbs/coco-trustee/cfs-kbs /cfs-kbs
COPY --from=builder /usr/src/kbs/coco-trustee/cfs-kbs/default_resource_policy.rego /opa/confidential-containers/kbs/policy.rego

WORKDIR /cfs-kbs
ENTRYPOINT ["/cfs-kbs/run.sh"]