# timebase
A BPF based tool for establishing NTP-based covert channels, for JHU 695.722 Covert Channels.

## Project Goals

*  Create a BPF XDP filter that can be attached onto a NTP server, such that 
the NTP server can infiltrate data into a network via client requests.
*  Create a malicious client that can read and reconstruct data slowly from 
NTP responses.
*  Measure the performance impact of our implementation on a Linux virtual 
machine.
*  Measure the bandwidth of the channel.
*  Enumerate some ways that the channel might be defeated.
*  Deliver a working userspace executable along with a paper and presentation summarizing
our work and results.
   
## Build Requirements

### timebase

*  Docker and docker-compose
*  Vagrant

### paper

*  pandoc
*  texlive-full (Linux) or mactex (macOS)

You can render the paper with the included `paper/build_paper.sh` script.

## Building

First, you must build the XDP filter program.

```
$ cd bpf/
$ docker-compose build
$ docker-compose run --rm filter-builder
$ cd ..
```
This will create the `filter_program_x86_64` program object file in the `bpf/` directory.
Then, you can build the program itself in a Linux VM.

```
$ vagrant up && vagrant ssh
$ cd timebase/
$ cargo build
$ cd target/debug
$ ./timebase
```
