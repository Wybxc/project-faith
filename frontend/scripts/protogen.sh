#!/usr/bin/env bash

mkdir -p ./src/generated/proto
protoc \
  --plugin=./node_modules/.bin/protoc-gen-ts_proto \
  --ts_proto_out=./src/generated/proto \
  --ts_proto_opt=esModuleInterop=true \
  --ts_proto_opt=outputClientImpl=grpc-web \
  --ts_proto_opt=oneof=unions-value \
  --ts_proto_opt=useReadonlyTypes=true \
  --ts_proto_opt=forceLong=long \
  --ts_proto_opt=outputJsonMethods=false \
  -I ../proto \
  ../proto/auth.v1.proto \
  ../proto/game.v1.proto
