# TODO list

## Feature: Pomodoro Session

run
```
timetrack pomo [minutes]
```
and it will pick the best activity to start, tell you how long to work for (up to `[minutes]`min) and start a timer.

if `[minutes]` is not provided, it will default to 30 minutes.

Once the minutes have passed, it will record the work as done, end the session and give you a notification.

If it receives a Ctrl+C before the timer is up, it will record the amount of work done and end the session, but also let you know how many
more minutes you were supposed to work.
