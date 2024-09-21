# Setup the Endless Online client

## Ubuntu

For Ubuntu users, we provide a script to simplify the client setup. Just follow these steps:

```sh
cd eo-client
chmod +x setup.sh
./setup.sh
```

This script will download the client, configure settings, and apply necessary fixes to get the client running smoothly.

## Manual setup (Windows and other platforms)

If you're not on Ubuntu or prefer a manual approach, follow these steps:

1. Download and unzip the client from [here](https://cache.tehsausage.com/EndlessOnline/EOzipped028.zip).

2. Copy the custom `eo-client/config/setup.ini` file from this repository into the `EOzipped028/config` directory. _(Do not replace the `setup.ini` in the root of the `EOzipped028` directory.)_

3. If using Wine, you may want to apply these optional fixes if the client doesn't run smoothly:
   ```sh
   cat /usr/lib/wine/wined3d.dll.so | sed 's/EnterCrit/LeaveCrit/' > $HOME/.wine/drive_c/windows/system32/wined3d.dll 
   wine reg add 'HKEY_CURRENT_USER\Software\Wine\DllOverrides' /v 'wined3d' /d 'native,builtin' /f
   ```

## Running the Client

1. Ensure you have the server up and running in a different terminal window.

2. Once you've completed the setup, you can run the Endless Online client:
   ```sh
   cd eo-client/EOzipped028 # or the directory where you unzipped the client
   wine endless.exe
   ```

3. Check the server console for a message confirming that the client has connected.
