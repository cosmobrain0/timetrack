# TODO list

## Feature: information provided

Make all commands show the state of the activities they manipulate before and after. (or maybe just after)

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

first the code needs to be neatly refactored
