// Native entry point for launching SteamOS/Deck ISO environment
// This is a stub for the C core, to be expanded with logic inspired by Pluvia/GameNative

#include <jni.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <string.h>

// Android NDK does not support fork/chroot/waitpid directly. Use shell commands as a workaround.
// This version uses system() to launch a shell script that does the work.

JNIEXPORT jint JNICALL
Java_com_piston_SteamOSNative_launchSteamOS(JNIEnv *env, jobject thiz, jstring isoPath) {
    const char *path = (*env)->GetStringUTFChars(env, isoPath, 0);
    printf("[SteamOSNative] Launching SteamOS from: %s\n", path);

    // 1. Prepare shell script to mount/extract and launch environment
    FILE *script = fopen("/data/local/tmp/launch_steamos.sh", "w");
    if (!script) {
        printf("[SteamOSNative] Failed to create shell script.\n");
        (*env)->ReleaseStringUTFChars(env, isoPath, path);
        return -10;
    }
    fprintf(script,
        "#!/system/bin/sh\n"
        "mkdir -p /data/local/tmp/steamos_root\n"
        "busybox mount -o loop '%s' /data/local/tmp/steamos_root 2>/dev/null || busybox tar -xf '%s' -C /data/local/tmp/steamos_root\n"
        "if [ -x /data/local/tmp/steamos_root/bin/bash ]; then\n"
        "  if [ $(id -u) -eq 0 ]; then\n"
        "    chroot /data/local/tmp/steamos_root /bin/bash\n"
        "  else\n"
        "    proot -S /data/local/tmp/steamos_root /bin/bash\n"
        "  fi\n"
        "else\n"
        "  echo 'No bash found in rootfs.'\n"
        "  exit 127\n"
        "fi\n",
        path, path);
    fclose(script);
    system("chmod 755 /data/local/tmp/launch_steamos.sh");

    // 2. Run the script
    int ret = system("sh /data/local/tmp/launch_steamos.sh");
    (*env)->ReleaseStringUTFChars(env, isoPath, path);
    return ret;
}
