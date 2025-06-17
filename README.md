# rust-port-scanner

Just use 
```sh
cargo run
```
to run the file

## Syntax

Ranges are deffined with [FIRST]-[LAST] (ex: 192.160.0.1-255 (IP addresses) or 1-65535 (ports))

## Flags:

- "-i" or "--ip" - used to specify an ip address or range with a string
- "-p" or "--port" - used to specify the port of range with an unsigned 16 bit intiger
- "-d" or "--duration" - used to specify the duration it should wait for a respone on each ip and port combination in milliseconds
- "-y" and "-n" - used to automatically say yes or no to showing only the open ports

