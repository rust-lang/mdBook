# `install` phase: install stuff needed for the `script` phase

set -ex

case $TARGET in
  # Install standard libraries needed for cross compilation
  i686-apple-darwin | \
  i686-unknown-linux-gnu | \
  x86_64-unknown-linux-musl)
    case $TRAVIS_RUST_VERSION in
      stable)
        # e.g. 1.6.0
        version=$(rustc -V | cut -d' ' -f2)
        ;;
      *)
        version=$TRAVIS_RUST_VERSION
        ;;
    esac
    tarball=rust-std-${version}-${TARGET}

    curl -Os http://static.rust-lang.org/dist/${tarball}.tar.gz

    tar xzf ${tarball}.tar.gz

    ${tarball}/install.sh --prefix=$(rustc --print sysroot)

    rm -r ${tarball}
    rm ${tarball}.tar.gz
    ;;
  # Nothing to do for native builds
  *)
    ;;
esac
