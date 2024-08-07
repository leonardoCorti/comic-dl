#!/bin/sh

cd "$(dirname "$0")"

# GitHub username and repository
USER="leonardoCorti"
REPO="comic-dl"
FILE_PATTERN="comic-dl-armv7-linux"

# Get the latest release download URL
DOWNLOAD_URL=$(wget -qO- https://api.github.com/repos/$USER/$REPO/releases/latest | grep "browser_download_url.*$FILE_PATTERN" | cut -d '"' -f 4)

# Check if the download URL was found
if [[ -z "$DOWNLOAD_URL" ]]; then
    echo "Error: Could not find the download URL for $FILE_PATTERN in the latest release."
    exit 1
fi

# Download the file
wget -O $FILE_PATTERN $DOWNLOAD_URL

# Verify the download was successful
if [[ $? -eq 0 ]]; then
    echo "Download completed successfully: $FILE_PATTERN"
else
    echo "Error: Download failed."
    exit 1
fi

