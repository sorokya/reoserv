#!/bin/bash

# Variables
CLIENT_URL="https://cache.tehsausage.com/EndlessOnline/EOzipped028.zip"
SETUP_INI_PATH="./config/setup.ini"
WINE_INSTALL_PREFIX="/usr/lib/wine"  # Default Wine installation path
WINEPREFIX="$HOME/.wine"             # Default Wine prefix

# Download and unzip the client
wget $CLIENT_URL -O EOzipped028.zip
unzip EOzipped028.zip -d EOzipped028

# Replace setup.ini
cp $SETUP_INI_PATH ./EOzipped028/config/setup.ini

# Install Wine if not installed
if ! command -v wine &> /dev/null
then
    echo "Wine could not be found, installing..."
    sudo apt update
    sudo apt install -y wine
fi

# Apply Wine fixes
cat $WINE_INSTALL_PREFIX/wined3d.dll.so | sed 's/EnterCrit/LeaveCrit/' > $WINEPREFIX/drive_c/windows/system32/wined3d.dll 
wine reg add 'HKEY_CURRENT_USER\Software\Wine\DllOverrides' /v 'wined3d' /d 'native,builtin' /f

echo "Client setup complete. You can now run the client using Wine."
