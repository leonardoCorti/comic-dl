#!/bin/sh

cd "$(dirname "$0")"

# GitHub username and repository
USER="leonardoCorti"
REPO="comic-dl"
FILE_PATTERN="comic-dl-armv7-linux"

# Download the file
wget -L "https://github.com/$USER/$REPO/releases/latest/download/$FILE_PATTERN"

# Verify the download was successful
if [[ $? -eq 0 ]]; then
    echo "Download completed successfully: $FILE_PATTERN"
else
    echo "Error: Download failed."
    exit 1
fi

