# TimeTrack

This is a terminal application for managing todo lists. It can track one-off tasks, and tasks that need to be worked on for a specific amount of time every day. It also allows you to start pomodoro sessions,
and picks a task for you to do in those pomodoro sessions so that you can't procrastinate one task by only ever working on other tasks.

To enable notifications when pomodoro sessions are completed, use the following features:  
- MacOS: feature `mac-notifications`  
- Linux: not implemented yet  
- Windows: not implemented yet  

This application stores state in a json file (defaults to `$HOME/.timetrack/state.json`). You can change the file path by setting the `TIMETRACK_STATE_FILE_PATH` to the path to the file (ending with the file name).
If the file does not exist, or directories in the file path do not exist, this program will create them when the program starts.

# TODO

## Feature: Todo item descriptions

It should be possible to give multi-line descriptions to todo items, which can be expanded and viewed.

## Feature: Todo Item Status

It should be possible to mark todo items as done or not done, without deleting them.

It should also be possible to delete every completed item in the current bucket

## Bug Fix: Todo List Input

Fix bug (0)

## Feature: Control Widget

The bottom of the screen should be reserved for showing every command which can be executed.

## Feature: Todo bucket descriptions

It should be possible to give buckets multi-line descriptions, which can be expanded and viewed.

## Feature: Edit bucket

It should be possible to edit bucket names and descriptions and to edit todo item names and descriptions.

## Feature: Cross-Platform Notifications

Notifications should work on Windows, Linux and MacOS.

## Feature: Error Info

When something goes wrong (e.g. invalid user input), the user should be notified in an appropriate way.

# Contributing

If you want to implement something in the TODO list above, then open a pull request!

If you think something should be added to the TODO list, open an issue!

# Known Bugs

## (0) Todo List Input

Pressing the 1 key on the Todo section's input box does not do anything (a 1 should be seen in the input box).

## (1) Inputs don't recognise capital letters

It looks like there's something wrong with inputting stuff using modifier keys such as Shift.
