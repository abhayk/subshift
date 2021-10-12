# subshift
A tool to add or subtract offsets to the timestamps in a .srt subtitle file. After offsets are
applied the original file will be backed up to <file>.orig

```
USAGE:
    subshift.exe --file <FILE> --offset <OFFSET>

FLAGS:
    -h, --help       Print help information
    -V, --version    Print version information

OPTIONS:
    -f, --file <FILE>        The path to the subtitle file
    -o, --offset <OFFSET>    The shift offset. To increment by half a second provide +500, To
                             decrement -500.
```
