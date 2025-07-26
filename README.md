# TimeTrack

This is a terminal application for managing todo lists. It can track one-off tasks, and tasks that need to be worked on for a specific amount of time every day. It also allows you to start pomodoro sessions,
and picks a task for you to do in those pomodoro sessions so that you can't procrastinate one task by only ever working on other tasks.

This application currently only works on MacOS because I couldn't find a cross-platform notification library.

# TODO

## Feature: Change state path

It should be possible to use an environment variable to change where the state is loaded from and saved to.

## Refactoring: Constant Updates

The state should be rarely saved to disk: only when the user quits the application, and when they start a pomodoro session.

## Bug Fix: Notifications Feature Flag

Notifications should be hidden behind a feature flag so that people on other operating systems can still use the app.

## Feature: Good Start

When run for the first time, the program should set up the required configuration files 

## Feature: Help Page / Manual

A paragraph explaining how to use this app

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

# Contributing

If you want to implement something in the TODO list above, then open a pull request!

If you think something should be added to the TODO list, open an issue!
