#!/bin/bash

CurrDir=$(cd "$(dirname "$0")"; pwd)

# kbs kata-runtime add: kata-qemu-security
vi /etc/containerd/config.toml

vi /home/cfs/work/kevin.chen/k8s/cc/runtimeclass-cfs.yaml

vi /home/cfs/work/kevin.chen/k8s/cc/install.sh

./install.sh

systemctl restart containerd


# kbs service
# https://github.com/kata-containers/kata-containers/blob/main/docs/Developer-Guide.md#build-a-custom-kata-agent---optional
export seccomp_install_path=/xxx/install/
export LIBSECCOMP_LIB_PATH="${seccomp_install_path}/lib"

make background-check-kbs POLICY_ENGINE=opa

export AA_EMULATE_ATTESTER=yes
export CFS_EMULATED_MODE=true
cp ./target/release/build/attestation-service-7a54c39712a09156/out/libcfs.so ./cfs-kbs/lib/
export LD_LIBRARY_PATH=./cfs-kbs/lib
./target/release/kbs --config-file ./cfs-kbs/kbs-config.toml

# kbs client

# set
export LD_LIBRARY_PATH=/home/cfs/work/herve.pang/cc/coco-trustee/cfs-kbs/lib
#export ServiceUrl=http://127.0.0.1:11111
export ServiceUrl=http://10.11.35.45:31111
export ResPath=default/test/file1

echo "haha-whf8934ht8y4f9h~83hrhhe~2hfh3tr-123" > ./cfs-kbs/file1

target/release/kbs-client \
  --url ${ServiceUrl} \
  config --auth-private-key ./cfs-kbs/private.key \
  set-resource --challenge 123456 --resource-file ./cfs-kbs/file1 \
  --path ${ResPath}

# get

target/release/kbs-client \
  --url ${ServiceUrl}  \
  get-resource --extra-credential-file ./cfs-kbs/extra_credential_file \
  --path ${ResPath}

target/release/kbs-client \
  --url ${ServiceUrl}  \
  get-kbs-evidence --challenge 123456

# ownership
# cfs/filesystems
curl -H "Content-Type:application/json" \
  -X POST \
  --data \
    '{"meta_tx_request":{"from":"err","to":"err","value":"err","gas":"err","nonce":"err","deadline":100,"data":"err"},"meta_tx_signature":"err"}' \
  ${ServiceUrl}/kbs/v0/cfs/filesystems

# cfs/filesystems/{name}
curl \
  -X GET \
  ${ServiceUrl}/kbs/v0/cfs/filesystems/fs1

# cfs/filesystems
curl -H "Content-Type:application/json" \
  -X DELETE \
  --data \
    '{"meta_tx_request":{"from":"err","to":"err","value":"err","gas":"err","nonce":"err","deadline":100,"data":"err"},"meta_tx_signature":"err"}' \
  ${ServiceUrl}/kbs/v0/cfs/filesystems

# cfs/accounts/{addr}/metatx
curl \
  -X GET \
  ${ServiceUrl}/kbs/v0/cfs/accounts/0xabc/metatx

# cfs/configure/.well-known
curl \
  -X GET \
  ${ServiceUrl}/kbs/v0/cfs/configure/.well-known


#end.
