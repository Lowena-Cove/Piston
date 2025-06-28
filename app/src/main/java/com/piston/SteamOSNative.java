package com.piston;

public class SteamOSNative {
    static {
        System.loadLibrary("SteamOSNative");
    }

    // Native method to launch SteamOS/Deck ISO
    public static native int launchSteamOS(String isoPath);
}
