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

## Feature: Todo List Buckets

It should be possible to categorise todo list items in buckets, and to switch between different buckets

## Feature: Todo item descriptions

It should be possible to give multi-line descriptions to todo items, which can be expanded and viewed.

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

None so far.
