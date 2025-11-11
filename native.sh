cd $(dirname $0)
cmake -GNinja -Bout-native -S.
ninja -C out-native