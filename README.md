# sst-chaos
To temporary chaos test

## if you have ldb binary

```
# NOTE: use absolute path
usage: <cmd> <ldb_path> <rocksdb_manifest_path>
```

## if you don't have ldb binary

```
git submodule update --init --recursive
cd rocksdb
make ldb -j16
```

Now you got ldb binary in `./rocksdb`, then follow the step above.