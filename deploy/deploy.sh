#!/bin/bash

set -e

if [ -z "$DEVICE_PASSWORD" ]; then
    echo "please set DEVICE_PASSWORD env first"
    exit 1
fi
if [ -z "$DB_NAME" ]; then
    echo "please set DB_NAME env first"
    exit 1
fi
if [ -z "$DB_PASSWORD" ]; then
    echo "please set DB_PASSWORD env first"
    exit 1
fi
if [ -z "$DB_USER" ]; then
    echo "please set DB_USER env first"
    exit 1
fi

export K8S_SERVICE=${1:-"cfs-kbs"}
export K8S_SC=${2:-"ceph-rbd"}
export K8S_STORAGE_SIZE=${3:-"2Gi"}
export K8S_NAMESPACE=${4:-"confidential-filesystems"}
export K8S_TLS_SECRET="cfs-kbs-tls-secret"
export K8S_KBS_SECRET="cfs-kbs-secret"

kubectl create ns $K8S_NAMESPACE 2>/dev/null || true

CERT_DIR=certs

echo "Creating $K8S_TLS_SECRET secret:"
kubectl delete secret $K8S_TLS_SECRET --ignore-not-found -n $K8S_NAMESPACE
kubectl create secret generic $K8S_TLS_SECRET \
  --from-file=tls.crt=$CERT_DIR/tls.crt \
  --from-file=tls.key=$CERT_DIR/tls.key -n $K8S_NAMESPACE

echo "Creating $K8S_KBS_SECRET secret:"
kubectl delete secret $K8S_KBS_SECRET --ignore-not-found -n $K8S_NAMESPACE
kubectl create secret generic $K8S_KBS_SECRET --from-literal=device_pwd=${DEVICE_PASSWORD} \
  --from-literal=db_name=${DB_NAME} \
  --from-literal=db_pwd=${DB_PASSWORD} \
  --from-literal=db_user=${DB_USER} -n $K8S_NAMESPACE

echo "Creating cfs-kbs-pvc pvc:"
sed -e "s/K8S_STORAGE_SIZE/$K8S_STORAGE_SIZE/" -e "s/K8S_SC/$K8S_SC/" ./artifact/cfs-kbs-pvc.yaml  > ./artifact/tmp-cfs-kbs-pvc.yaml
kubectl apply -f ./artifact/tmp-cfs-kbs-pvc.yaml -n $K8S_NAMESPACE

echo "Applying kbs deployment:"
kubectl delete -f ./artifact/cfs-kbs.yaml --ignore-not-found -n $K8S_NAMESPACE
kubectl apply -f ./artifact/cfs-kbs.yaml -n $K8S_NAMESPACE


rm -rf ./artifact/tmp-cfs-kbs-pvc.yaml