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

This is what happens if there is no ongoing task. If there is already an ongoing task, the user is told about the ongoing task and prompted
to end it before running `timetrack pomo` again.

The user should be able to see the timer.

## Feature: todo list

run
```
timetrack todo list
```

to list one-off todos sorted (and numbered) by priority

run
```
timetrack todo add <name> <description?>
```

to add an item to the bottom of the todo list

run
```
timetrack todo swap <id1> <id2>
```

to swap the items with those priorities in the todo list

run
```
timetrack todo moveabove <id1> <id2>
timetrack todo ma <id1> <id2>
```
to move `<id2>` immediately above `<id1>` in the todo list

run
```
timetrack todo movebelow <id1> <id2>
timetrack todo mb <id1> <id2>
```
to move `<id2>` immediately above `<id1>` in the todo list
