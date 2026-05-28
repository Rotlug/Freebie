# Freebie
![Screenshot of Freebie](./freebie.png)

GTK4 Frontend for downloading games from FitGirl

*for educational purposes only*

## Command line usage
You can also use `freebie` as a command line interface for downloading, installing and launching games.

### Example usage:
The command to obtain and launch a game would look like this
```bash
freebie -o elden-ring -l elden-ring -c [igdb-client-id],[igdb-client-secret]
```
The ordering of the arguments does not matter. You can also omit the `-c` argument if you already
configured your credentials through the GUI.
