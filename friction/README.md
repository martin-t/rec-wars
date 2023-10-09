# RecWars vs Quake friction

This is a small tool for comparing RecWars and Quake friction code.

I tried to make friction framerate independent in RecWars.
If you run 30 fps, you get the same resulting velocity (ignoring float rounding errors)
as 60 fps because the time delta (dt) changes too and using powf assures friction applies proportionally.
Unfortunately I didn't realize that when integrating it to get distance, the step size still matters
and therefore the final position is affected by framerate.

Quake friction uses a different formula which takes dt into account in such a way
that even velocity differs based on framerate so there are two places where differences accumulate.
Quake code: https://github.com/id-Software/Quake-III-Arena/blob/master/code/game/bg_pmove.c

This example shows that the effect, however, is much smaller than I expected.
The upper table shows that I have chosen parameters for which custom and Quake give almost the same results at 60 FPS.
Then compare it to the lower table - it shows what happens at 15 FPS.
With my algorithm, the speed at a given time (compare rows with the same `i`) is the same regardless of FPS.
However, the total distance trvavelled at the end is different.
The error in Quake physics is only about twice bigger than in mine though.

Note that i will likely change RecWars to use either Quake's or rapier's friction in the future so this example might become obsolete.
