#!/bin/bash
set -eo pipefail

files=$(find $(pwd) -wholename "**/solutions/*.txt")
for file in $files; do
    num=$(echo $file | grep -Eow "([0-9]+).txt" | grep -Eow "[0-9]+")
    echo "submitting solution for problem $num in $file"
    curl -S -s \
        -X POST "https://robovinci.xyz/api/submissions/$num/create" \
        -H "Authorization: Bearer $ICFPC_TOKEN" \
        --form file=@"$file"
done
