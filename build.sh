#!/bin/bash

CurrDir=$(cd "$(dirname "$0")"; pwd)

echo "" && echo "" && echo ""
Op=none
if [ $1 ]; then
  Op=$1
  shift
fi
echo Op=${Op}

#export SourceDir=xxx
echo "" && echo "" && echo ""
echo SourceDir=${SourceDir}

# update code
if [ ${Op} = "update" ]; then
  echo "" && echo "" && echo ""
  # rm -rf ../coco-attestation-service/attestation-service/*
  scp -r ${SourceDir}/coco-attestation-service/attestation-service/* \
    ../coco-attestation-service/attestation-service/

  echo "" && echo "" && echo ""
  # rm -rf ./src/*
  scp -r ${SourceDir}/coco-trustee/src/* \
    ./src/

  echo "" && echo "" && echo ""
    # rm -rf ./tools/*
    scp -r ${SourceDir}/coco-trustee/tools/* \
      ./tools/

fi

# build kbs service
echo "" && echo "" && echo ""
# https://github.com/kata-containers/kata-containers/blob/main/docs/Developer-Guide.md#build-a-custom-kata-agent---optional
export seccomp_install_path=/xxx/install/
export LIBSECCOMP_LIB_PATH="${seccomp_install_path}/lib"

rm -f ./target/release/build/attestation-service-7a54c39712a09156/out/libcfs.so
rm -f ./target/release/kbs
rm -f ./target/release/kbs-client
make background-check-kbs POLICY_ENGINE=opa

echo "" && echo "" && echo ""
if [ -s ./target/release/build/attestation-service-7a54c39712a09156/out/libcfs.so ]; then
	echo "compile libcfs.so succ ."
else
    echo "ERROR: compile libcfs.so fail !"
    exit 1;
fi

echo "" && echo "" && echo ""
if [ -s ./target/release/kbs ]; then
	echo "compile kbs succ ."
else
    echo "ERROR: compile kbs fail !"
    exit 2;
fi

echo "" && echo "" && echo ""
if [ -s ./target/release/kbs-client ]; then
	echo "compile kbs-client succ ."
else
    echo "ERROR: compile kbs-client fail !"
    exit 2;
fi

# run kbs service
echo "" && echo "" && echo ""
mkdir -p ./cfs-kbs/lib
rm -f ./cfs-kbs/lib/libcfs.so
rm -f ./cfs-kbs/kbs
rm -f ./cfs-kbs/kbs-client
cp ./target/release/build/attestation-service-7a54c39712a09156/out/libcfs.so ./cfs-kbs/lib/
cp ./target/release/kbs ./cfs-kbs/
cp ./target/release/kbs-client ./cfs-kbs/
chmod 755 ./cfs-kbs/kbs
chmod 755 ./cfs-kbs/kbs-client
chmod 755 ./cfs-kbs/run.sh

if [ ${Op} = "update" ]; then
  echo "" && echo "" && echo ""
  export AA_EMULATE_ATTESTER=yes
  export CFS_EMULATED_MODE=true
  export LD_LIBRARY_PATH=${CurrDir}/cfs-kbs/lib
  ./target/release/kbs --config-file ./cfs-kbs/kbs-config.toml

else
  echo "" && echo "" && echo ""
  KBSImage=coco-trustee:v0.8.0-filesystem-d4
  docker rmi -f ${KBSImage}
  docker build -f Dockerfile -t ${KBSImage} .

  docker tag ${KBSImage} hub.confidentialfilesystems.com:30443/cc/${KBSImage}
  docker push hub.confidentialfilesystems.com:30443/cc/${KBSImage}

  echo "" && echo "" && echo ""
  echo Op=${Op}
  if [ ${Op} = "run-docker" ]; then
    KBSContainer=coco-trustee-kbs
    docker rm -f ${KBSContainer}

    docker run -itd --privileged \
      --name=${KBSContainer} \
      --restart=always \
      -p 8443:8443 \
      ${KBSImage} \
      /bin/bash

    docker ps -a | grep -i coco-trustee-kbs

    docker exec -it coco-trustee-kbs bash
  fi

fi

#
echo "" && echo "" && echo ""
exit 0
#end.
