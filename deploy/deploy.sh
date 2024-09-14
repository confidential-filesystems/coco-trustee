#!/bin/bash

set -e

if [ -z "$DEVICE_PASSWORD" ]; then
    echo "please set DEVICE_PASSWORD env first"
    exit 1
fi

export K8S_SERVICE=${1:-"cfs-security-engine"}
export K8S_SC=${2:-"ceph-rbd"}
export K8S_STORAGE_SIZE=${3:-"2Gi"}
export K8S_NAMESPACE=${4:-"confidential-filesystems"}
export K8S_TLS_SECRET="cfs-security-engine-tls-secret"

kubectl create ns $K8S_NAMESPACE 2>/dev/null || true

CERT_DIR=certs

echo "Creating $K8S_TLS_SECRET secret:"
kubectl delete secret $K8S_TLS_SECRET --ignore-not-found -n $K8S_NAMESPACE
kubectl create secret generic $K8S_TLS_SECRET \
  --from-file=tls.crt=$CERT_DIR/tls.crt \
  --from-file=tls.key=$CERT_DIR/tls.key -n $K8S_NAMESPACE

echo "Creating cfs-security-engine-pvc pvc:"
sed -e "s/K8S_STORAGE_SIZE/$K8S_STORAGE_SIZE/" -e "s/K8S_SC/$K8S_SC/" ./artifact/cfs-security-engine-pvc.yaml  > ./artifact/tmp-cfs-security-engine-pvc.yaml
kubectl apply -f ./artifact/tmp-cfs-security-engine-pvc.yaml -n $K8S_NAMESPACE

echo "Applying security engine deployment:"
kubectl delete -f ./artifact/cfs-security-engine.yaml --ignore-not-found -n $K8S_NAMESPACE
kubectl apply -f ./artifact/cfs-security-engine.yaml -n $K8S_NAMESPACE


rm -rf ./artifact/tmp-cfs-security-engine-pvc.yaml