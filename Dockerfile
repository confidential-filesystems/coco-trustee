FROM debian:stable-slim as builder

RUN apt-get update && \
    apt-get install -y \
    clang \
    curl \
    gnupg-agent \
    procps net-tools

# Install TDX Runtime Dependencies
RUN curl -fsSL https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | \
    gpg --dearmor --output /usr/share/keyrings/intel-sgx.gpg
RUN echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/intel-sgx.gpg] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | tee /etc/apt/sources.list.d/intel-sgx.list
RUN apt-get update
RUN apt-get install -y --no-install-recommends \
    libsgx-dcap-default-qpl \
    libsgx-dcap-quote-verify \
    libtdx-attest \
    tpm2-tools && apt-get clean && rm -rf /var/lib/apt/lists/*

# Intel PCCS URL Configurations
# If you want the AS in KBS to connect to your customized PCCS for Intel TDX/SGX evidence verification,
# please modify this parameter.
# Default using localhost PCCS (Run in Host which the container land on).
ENV INTEL_PCCS_URL "https://localhost:8081/sgx/certification/v4/"
ENV INTEL_PCCS_USE_SECURE_CERT false

# Setup Intel PCCS URL
RUN sed -i "s|\"pccs_url\":.*$|\"pccs_url\":$INTEL_PCCS_URL,|" /etc/sgx_default_qcnl.conf; \
    sed -i "s/\"use_secure_cert\":.*$/\"use_secure_cert\":$INTEL_PCCS_USE_SECURE_CERT,/" /etc/sgx_default_qcnl.conf


FROM debian:stable-slim

RUN apt-get update
RUN apt-get install -y ca-certificates  && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY ./cfs-kbs /cfs-kbs
COPY --from=builder /lib/x86_64-linux-gnu/libtdx_attest.so.1.21.100.3 /lib/x86_64-linux-gnu/libtdx_attest.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libsgx_dcap_quoteverify.so.1.13.101.3 /lib/x86_64-linux-gnu/libsgx_dcap_quoteverify.so.1
COPY --from=builder /lib/x86_64-linux-gnu/libtss2-esys.so.0.0.0 /lib/x86_64-linux-gnu/libtss2-esys.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libtss2-tctildr.so.0.0.0 /lib/x86_64-linux-gnu/libtss2-tctildr.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libtss2-mu.so.0.0.0 /lib/x86_64-linux-gnu/libtss2-mu.so.0
COPY --from=builder /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/x86_64-linux-gnu/libcrypto.so.3
COPY --from=builder /lib/x86_64-linux-gnu/libtss2-sys.so.1 /lib/x86_64-linux-gnu/libtss2-sys.so.1

WORKDIR /cfs-kbs
ENTRYPOINT ["/cfs-kbs/run.sh"]
