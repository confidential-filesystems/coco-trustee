#!/bin/bash

# kbs service
# https://github.com/kata-containers/kata-containers/blob/main/docs/Developer-Guide.md#build-a-custom-kata-agent---optional
export seccomp_install_path=/xxx/install/
export LIBSECCOMP_LIB_PATH="${seccomp_install_path}/lib"

make background-check-kbs POLICY_ENGINE=opa

export AA_EMULATE_ATTESTER=yes
export CFS_EMULATED_MODE=true
cp ./target/release/build/attestation-service-7a54c39712a09156/out/libcfs.so ./cfs-test/lib/
export LD_LIBRARY_PATH=./cfs-test/lib
./target/release/kbs --config-file ./cfs-test/kbs-config.toml

# kbs client

# set
target/release/kbs-client \
  --url http://127.0.0.1:11111 \
  config --auth-private-key ./cfs-test/private.key \
  set-resource --resource-file ./cfs-test/file1 --path default/test/file1

# get
target/release/kbs-client \
  --url http://127.0.0.1:11111  \
  get-resource \
  --path default/test/file1 \
  --extra-credential-file ./cfs-test/extra_credential_file

#end.
