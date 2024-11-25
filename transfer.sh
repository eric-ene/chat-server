#!/usr/bin/env bash
 
MAIN_FILE_NAME="chat_server"
DEST_FILE_NAME="chat-server"
MAIN_FILE_DIR="target/x86_64-unknown-linux-musl/release"

DATA_DIRS=()
ADDITIONAL_FILES=()

REMOTE="home_server"
REMOTE_DIR="/home/eric/containers/chat-server"

scp "./$MAIN_FILE_DIR/$MAIN_FILE_NAME" "$REMOTE:$REMOTE_DIR/$DEST_FILE_NAME"

# shellcheck disable=SC2068
for DIR in ${DATA_DIRS[@]}; do
  scp -r "./$DIR" "$REMOTE:$REMOTE_DIR"
done

# shellcheck disable=SC2068
for FILE in ${ADDITIONAL_FILES[@]}; do
  scp -r "./$FILE" "$REMOTE:$REMOTE_DIR/$FILE"
done
