cmake_minimum_required(VERSION 3.10.2)

project(SteamOSNative)

add_library(
        SteamOSNative
        SHARED
        SteamOSNative.c)

target_link_libraries(
        SteamOSNative
        log)
