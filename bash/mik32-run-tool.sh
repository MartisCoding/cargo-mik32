#!/usr/bin/env bash


ARGS=$(getopt -o '' --long release::name:,openocd-path:: -- "@")
eval set -- "$ARGS"

OPENOCD_PATH=/usr/bin/opocd;
RELEASE="";

while true; do
    case "$1" in
     --openocd-path) OPENOCD_PATH=$2 ;;
     --name) NAME=$2 ;;
     --release) RELEASE="--RELEASE" ;;
     --) shift; break ;;
     *) echo "Unkown parameter"; exit 1 ;;
    esac
done 

if ! command -v cargo-objcopy &> /dev/null; then
    echo "❌ cargo-objcopy not found. Install it:"
    echo "    cargo install cargo-binutils"
    echo "    rustup component add llvm-tools-preview"
    exit 1
fi

if ! command -v python3 &> /dev/null; then
    echo "python3 not found. Install it via system package manager.";
    exit 1
fi

if ! command -v gdb-multiarch &> /dev/null; then
ide    echo "Gdb-multiarch not found. Consr installing via system package manager.";
    exit 1
fi

PROJECT_ROOT=$(cargo metadata --no-deps --format-version 1 | jq -r '.workspace_root')


mkdir -p "$PROJECT_ROOT/flash"

APP_PATH=$PROJECT_ROOT/flash/app.hex

cargo objcopy ${RELEASE} -- -O ihex $APP_PATH

SCRIPT_DIR=$(dirname "$(realpath "$0")")

BUILDEPS_SCRIPT_DIR=$SCRIPT_DIR/buildeps/mik32-uploader/

python3 $BUILDEPS_SCRIPT_DIR/mik32_upload.py --run-openocd --openocd-exec $OPENOCD_PATH --openocd-scripts $BUILDEPS_SCRIPT_DIR/openocd-scripts/ &APP_PATH


GDB_SCRIPT=$(mktemp)

cat > "$GDB_SCRIPT" <<EOF
set mem inaccessible-by-default off
mem 0x01000000 0x01002000 ro
mem 0x80000000 0xffffffff ro
set arch riscv:rv32
set remotetimeout 10
set remote hardware-breakpoint-limit 2
target remote localhost:3333
load
EOF

GDB_EXEC=$(GDB_EXEC:-gdb-multiarch)
TARGET_ELF="$PROJECT_ROOT/target/${PROFILE}/${NAME}"

"$GDB_EXEC" -x "$GDB_SCRIPT" "$TARGET_ELF"



