#!/usr/bin/env zsh

play -v 10 /usr/share/sounds/KDE-Sys-App-Message.ogg &
sleep 0.5
xdotool keydown Control_L sleep 0.5 keyup Control_L sleep 1 keydown Left sleep 0.2 keyup Left keydown Right sleep 0.07 keyup Right
