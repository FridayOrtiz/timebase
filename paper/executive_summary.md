---
subtitle: NTP-based Data Infiltration Channels
author:
  - "Rafael Ortiz &lt;rortiz12@jhu.edu&gt;"
  - "Ramon Benitez-Pagan &lt;ramon.benitez@jhu.edu&gt;"
  - "Stephen Scally &lt;sscally@jhu.edu&gt;"
  - "Cyrus Bulsara &lt;cbulsar1@jhu.edu&gt;"
header-includes:
  - \usepackage[ruled,vlined,linesnumbered]{algorithm2e}
  - \usepackage{tikz}
  - \usepackage{gensymb}
  - \usepackage{fancyhdr}
  - \usepackage{amsmath}
  - \usepackage{stmaryrd}
  - \usepackage{lastpage}
  - \usepackage{multicol}
  - \newcommand{\hideFromPandoc}[1]{#1}
  - \hideFromPandoc{
      \let\Begin\begin
      \let\End\end
    }
  - \pagestyle{fancy}
  - \fancyfoot[LO,LE]{Ortiz R., Benitez R., Scally S., Bulsara C.}
  - \fancyfoot[C]{}
  - \fancyfoot[RE,RO]{\thepage\ of \pageref{LastPage}}
  - \renewcommand*{\thefootnote}{\normalfont\fnsymbol{footnote}} 
type: pdf\_document
geometry: margin=3cm
toc: false
---

\Begin{multicols}{2}

Many papers focus on creating covert channels for the purpose of data 
exfiltration. That is, they attempt to remove some information from a
protected network. Less common appears to be the concept of data
_infiltration_, where a covert channel is established to secretly move
information _inside_ a protected network. Many exfiltration oriented
channels make assumptions about extant arbitrary infiltration channels being
available for loading the tooling necessary to establish the outbound
channel. We are proposing studying and implementing an NTP-based covert
channel for infiltrating unauthorized information into a secure network.

# Problem Statement

In typical enterprise threat scenarios, data infiltration is often achieved by 
way of phishing emails, TLS tunnels, or shell access. From there, exfiltration
may be achieved by a number of covert channels. However, in certain locked-down
networks, it may not be possible to send inbound email, establish a TLS tunnel,
or directly access protected machines. On the other hand, administrators of
these networks typically _do_ want their machines clocks to be synchronized, since
Bad Things$^{TM}$ happen when clocks drift out of sync. This presents NTP as
a potential channel whereby information from the Internet can slowly leak into the inside
of a protected network. We will consider two scenarios: DMZ access and
direct access.

## DMZ Access

In DMZ access, devices in a secure network syncrhonize their clocks via an NTP
server that is located in a controlled DMZ. That NTP server syncrhonizes its 
clock by connecting to one or more trusted NTP pools. Any information that
can survive a stratum layer[^stratum] may reach devices through the DMZ.

```







```

```
   ┌───┐
   │NTP│ Stratum 1
   └─┬─┘
     │
┌────┼───────────────────┐
│    │   DMZ             │
│  ┌─▼─┐                 │
│  │NTP│ Stratum 2       │
│  └─┬─┘                 │
│    │  ┌──────────────┐ │
│    └──►Secure Network│ │
│       │Stratum 3     │ │
│       └──────────────┘ │
│                        │
└────────────────────────┘
```
**Figure 1.** The DMZ Access NTP scenario, where an NTP server in a DMZ 
serves replies to clients in a secure inner network.

\pagebreak
## Direct Access

In direct access, devices in a secure network synchronize their clocks directly
via NTP to a trusted, publicly available, pool. Any information that the NTP
server can pack into an NTP reply may reach devices inside this network.

```
 ┌───┐
 │NTP│ Stratum 1
 └─┬─┘
   │
┌──┼───────────────────┐
│  │   DMZ             │
│  │                   │
│  │  ┌──────────────┐ │
│  └──►Secure Network│ │
│     │Stratum 2     │ │
│     └──────────────┘ │
│                      │
└──────────────────────┘
```
**Figure 2.** The Direct Access NTP scenario, where clients in a secure inner
network are allowed direct access to NTP pool servers.


[^stratum]:  In NTP, a stratum is a layer of devices that are the same distance 
from a reference clock. Reference clocks, considered a source of truth, are
at stratum layer $0$. Servers that rely on reference clocks are at stratum layer
$1$, and so on. In our DMZ scenario, if public pools are at stratum $1$, then
the DMZ NTP server is at stratum $2$, and internal devices are at stratum $3$.


# Project Summary

We are proposing the creation of an extended Berkeley Packet Filter (BPF)
eXpress Data Path (XDP) filter that can be layered on top of existing NTP
servers, or on routers between an NTP server and the target client, that can
modify NTP server replies such that data may be infiltrated into secure 
networks. We will consider prior art on NTP covert channels and NTP as a
covert storage cache and device a method for data infiltration in one
or both of the two scenarios.

After identifying a potential covert channel for one or both scenarios, we
will fully define and document the channel. We will pay attention to factors
such as: channel bandwidth, channel resiliance, channel covertness, and so on.

After defining the channel, we will implement and test the channel in a
virtual environment. There will likely be two components to the channel,
an XDP filter and a receiver application that can decode received information.
Ideally, we will be able to build the receiver with nothing but tools
available on the target machine, requiring no outside applications. However,
for the purposes of this paper, we will consider the infiltration of data
into a secure network as "good enough," and leave data reconstruction as
a problem for the reader.

# Project Breakdown

This is a rough outline of our project plan. At this stage, steps may be
added or removed as necessary.

1. Study prior art on NTP storage channels and NTP covert caches
1. Identify potential NTP storage channels that satisfy the following criteria:
    *  Do not noticeably interfere with normal NTP functionality
    *  (Optionally) survive a stratum layer
1. Design and document one or more covert channels that meet these criteria
1. Implement the channel as a BPF XDP filter, with a receiver application
    *  Document a method to reconstitute data using nothing but OS native utilities
1. Demonstrate our implementation working in a virtual machine environment

Our final deliverables will be:

*  A working sender and receiver application
*  A report with the following sections:
    1.  Abstract
    1.  Introduction, covering background on NTP channels
    1.  Related works in NTP covert channels
    1.  Design of our channel
    1.  Implementation of our channel
    1.  Methods for discovering and defeating our channel
    1.  Conclusions and Future Work
    1.  Bibliography
*  A video reviewing the above paper and demonstrating our implementation

\End{multicols}
\End{document}
