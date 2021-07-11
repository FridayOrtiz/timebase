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
Then, you can run the program itself in the Linux lab environment. `scargo` is included
as an alias to run `cargo` as root for convenience.

First, start the lab environment.

```
$ cd lab/
$ vagrant up 

```

Then launch the client listener.

```
$ vagrant ssh client
$ cd timebase/
$ scargo run -- client --interface eth1
```

From a second terminal, launch the server.

```
$ cd lab/
$ vagrant ssh server
$ cd timebase/
$ scargo run -- server --interface eth1
```

# Licenses

All Rust code here is distributed under the MIT license. 

The BPF filter program source (`bpf/filter.c`) and subsequent artifacts are distributed under dual MIT/GPL.
