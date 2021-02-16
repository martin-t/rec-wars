#!/usr/bin/env zsh

# Script for comparing how quickly guided missiles in RW stop turning.

play -v 10 /usr/share/sounds/KDE-Sys-App-Message.ogg &
sleep 0.5  # needed, otherwise RW doesn't detect CTRL

# xdotool sleep is much more precise than normal sleep
xdotool keydown Control_L sleep 0.5 keyup Control_L sleep 1 keydown Left sleep 0.2 keyup Left
