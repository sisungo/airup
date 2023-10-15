# Airup C/C++ SDK
This directory contains the Airup SDK for `C` and `C++`.

## Requirements
 - A C/C++ compiler with `C23` and `C++20` support.
 - `cmake`, the build system used by the SDK.

## Build
To build Airup C/C++ SDK, first we should create a separated `build/` directory:
```bash
$ mkdir build/
$ cd build/
```
Then, run `cmake` to generate build files, and then start building, for example:
```bash
$ cmake ..
$ make -j12
```
Have a good time! :\)