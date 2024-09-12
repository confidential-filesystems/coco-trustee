FROM confidentialfilesystems/coco-trustee-builder:v1.0.0 AS builder

WORKDIR /usr/src
COPY . coco-trustee
RUN --mount=type=ssh cd coco-trustee && cargo install --locked --path src/kbs --no-default-features --features coco-as-builtin,resource,rustls,opa

RUN mkdir -p coco-trustee/cfs-kbs/lib && find coco-trustee/target/release -name libcfs.so -print0 | xargs -0 -I {} cp {} coco-trustee/cfs-kbs/lib/
RUN cp coco-trustee/target/release/kbs coco-trustee/cfs-kbs/

FROM confidentialfilesystems/coco-trustee-base:v1.0.0

COPY --from=builder /usr/src/coco-trustee/cfs-kbs /cfs-kbs
COPY --from=builder /usr/src/coco-trustee/cfs-kbs/default_resource_policy.rego /opa/confidential-containers/kbs/policy.rego

WORKDIR /cfs-kbs
ENTRYPOINT ["/cfs-kbs/run.sh"]