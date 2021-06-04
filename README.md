# Velocity Calculator for Trello

## Setup

### Get an API key to access your boards
TODO

```toml
[trello.api]
key = ""
token = ""
```

### Get the identifier for the trello board you use
Before you can start calculating the velocity for a sprint, you have to determine the id of the board on which the
stories are managed. By opening the board and looking at the URL you can figure it out. If the URL would be
`https://trello.com/b/deadbeef/some-additional-board-title`, the id of your bord is `deadbeef`. To let the
program know which board you want to use, simply but the following lines into your `velocity.toml` file:

```toml
[trello.board]
id = "deadbeef"
```

### Get the identifier for the lists to be used for the calculation
By running `trello-velocity-calculator show-lists-of-board` you can query the lists available on a trello board. This
is required to specify which lists contain which kind of stories. After running the command stated above, the result
should look similar to the following listing:

```
+--------------------------+--------------------------------+
| **ID**                   | **List name**                  |
+--------------------------+--------------------------------+
| deadbeefdeadbeefd8e4efff | Inbox                          |
+--------------------------+--------------------------------+
| deadbeefdeadbeef9272efff | Canvas backlog                 |
+--------------------------+--------------------------------+
| deadbeefdeadbeef91466fff | Sprint backlog                 |
+--------------------------+--------------------------------+
| deadbeefdeadbeefb385afff | Doing                          |
+--------------------------+--------------------------------+
| deadbeefdeadbeef4b85dfff | Done (this sprint) 🎉          |
+--------------------------+--------------------------------+
| deadbeefdeadbeef3a11bfff | Done (older sprints)           |
+--------------------------+--------------------------------+
```

Use these information to setup your `velocity.toml` file by filling out the following lines:

```toml
[trello.lists]
backlog_id = "deadbeefdeadbeef91466fff"
doing_id = "deadbeefdeadbeefb385afff"
done_id = "deadbeefdeadbeef4b85dfff"
```