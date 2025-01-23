#!/bin/bash
if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <argument>"
  exit 1
fi

ARG=$1

JSON_PAYLOAD=$(cat <<EOF
{ "payment_request": "$ARG" }
EOF
)

URL="http://localhost:3000/invoice/settled"

echo "$JSON_PAYLOAD" | http POST "$URL" Content-Type:application/json
