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
export ResPath=default/test/file1

echo "haha-whf8934ht8y4f9h~83hrhhe~2hfh3tr-123" > ./cfs-test/file1

target/release/kbs-client \
  --url http://127.0.0.1:11111 \
  config --auth-private-key ./cfs-test/private.key \
  set-resource --challenge 123456 --resource-file ./cfs-test/file1 \
  --path ${ResPath}

# get

target/release/kbs-client \
  --url http://127.0.0.1:11111  \
  get-resource --extra-credential-file ./cfs-test/extra_credential_file \
  --path ${ResPath}

target/release/kbs-client \
  --url http://127.0.0.1:11111  \
  get-kbs-evidence --challenge 123456

#end.
