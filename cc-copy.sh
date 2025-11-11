set -eu
cd $(dirname $0)
rm -rf $1/cc-copy || echo "initializing"
mkdir -p $1/cc-copy
cp -r ./include ./src $1/cc-copy