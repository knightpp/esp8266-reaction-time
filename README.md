# esp8266_reaction_time

This program is intended to test your reaction time. The built-in LED will flash after some random
delay and you are supposed to press FLASH button as soon as possible.

## How it works

1. The program start a timer with 1 ms interval.
2. Interruption handler adds 1 to the interruption counter;
3. When a user presses the button
    1. if LED is disabled print _too soon_ message and exit interruption handler
    2. otherwise print `INTERRUPTION_COUNTER` variable and set the variable to 0

There's an additional variable called `START_NEW_GAME` which is used in the main loop to wait until
the currect game finishes.
