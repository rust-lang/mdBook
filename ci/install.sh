
set -ex

case $TARGET in
    x86_64-*)
        ;;
    i686-unknown-linux-gnu)
        rustup target add $TARGET
        sudo apt-get -qq update
        sudo apt-get -y install gcc-multilib
        ;;
    i686-*)
        rustup target add $TARGET
        ;;
esac
