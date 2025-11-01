#!/usr/bin/env bash
set -e

HOST="127.0.0.1"
PORT="8078"
SEND=$(printf '\x13\xFE\xFF\xFF\x98\xAA\x5\x1\x1\x1D\x71\xB\x31\x37\x38\x33\x31\x38\x36\x30\x39\x30')
EXPECT="0dfeffff02"

RESPONSE=$(printf "$SEND" | nc -w 2 -q 1 "$HOST" "$PORT" | xxd -p -c 256)

if [[ "$RESPONSE" == "$EXPECT"* ]]; then
  exit 0
else
  echo "Unexpected handshake response: $RESPONSE"
  exit 1
fi