---
title: BPF traffic control filter for implementing a NTP covert channel.
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
  - \usepackage{supertabular}
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
bibliography: paper.bib
type: pdf\_document
geometry: margin=3cm
toc: false
monofont: DejaVuSansMono
---


\Begin{multicols}{2}

# Abstract {-}

Many papers focus on creating covert channels for the purpose of data
exfiltration. That is, they attempt to remove some information from a
protected network. Less common appears to be the concept of data
_infiltration_, where a covert channel is established to secretly move
information _inside_ a protected network. Many exfiltration oriented
channels make assumptions about extant arbitrary infiltration channels being
available for loading the tooling necessary to establish the outbound
channel. We are proposing studying and implementing an NTP-based covert
channel for infiltrating unauthorized information into a secure network.


# Introduction

Covert channels are traditionally classified as either storage or timing
channels.  These classifications came about from definitions outlined in the
Pink book. These definitions are listed below:

_Definition 1_:

A communication channel is covert if it was neither designed or intended to
transfer information at all. [Pink book section 2.1]

**TODO**: convert these to proper bibtex citations

_Definition 2_:

A communications channel is covert if it is based on "transmission by storage
into variables that describe resource states." [Pink book section 2.1]

_Definition 3_:

Covert channels "will be defined as those channels that are a result of resource
allocation policies and resource management implementation." [Pink book section
2.1]

_Definition 4_:

Covert channels are those that "use entities not normally viewed as data objects
to transfer information from one subject to another." [Pink book section 2.1]

_Definition 5_:

Given a non-discretionary security policy model M and its interpretation I(M) in
an operating system, any potential communication between two subjects I(${S_h}$)
and I(${S_i}$) of I(M) is covert if and only if any communication between the
corresponding subjects ${S_h}$ and ${S_i}$ of the model M is illegal in M [Pink
book section 2.1]

After understanding the criteria of what makes a covert channel our goal within
this paper is to utilize the network time protocol otherwise known as NTP to
create a covert channel  for data infiltration.

There are a number of network architectures that enterprises and device
manufacturers utilize in order to have network connected devices synchronize
their system time. We reason that most security controls within these
architectures focus on ensuring that access is only allowed to appropriate 
network time servers, over the protocol UDP and port 123, and that public
network time servers are themselves clients of higher tiered stratum layer
servers. Due to the default trust associated with NTP very few operators
validate the public NTP pool servers they connect to or the time data that is
retrieved and propagated throughout the network from servers to clients.

In order to address these security concerns as well as control NTP delivery,
enterprises have deployed dedicated vendor appliances while device manufacturers
have removed the ability to change time sources on certain devices. Furthermore,
adding to the lower scrutiny of NTP usage, CVEs and exploits are typically
directed at public NTP instances acting as time sources, with attacks in the
form of DDoS and reflective type attacks.  While it can be beneficial for an
adversary to attack an internal time source as these disruptions can affect
remote logins, authentication tokens, and DHCP leasing most service logging
indicates clearly when a time or date synchronizations are out of skew. For an
adversary attempting to keep a low profile, internal NTP attacks may create too
much noise for very little gain.  

We intend to show that, with the above factors in mind, ingress and egress NTP
communications that are not being analyzed for correctness leaves networks and
devices open to covert channel utilization.  This paper is broken down into the
following sections. In the background section we will will provide basic
terminology and workings of NTP and its communication structure. Related works
will review past and current analysis related to NTP covert channels. The design
section will explain our network architecture for implementation as well as
review our expected throughput, robustness, and detection of this channel.  The
implementation will demonstrate our covert channel in our lab environment.
Lastly, our conclusions and future work outlines further areas of expanding the
NTP covert channels based on current standards specifications as well as
observations during our implementation.


# Background

## NTP Modes

The network time protocol (NTP) operates in one of three modes. 

### Primary Server

The first mode is primary server which is directly synchronized from a reference
clock. Reference clocks can come from multiple sources however the most common
are satillite based from GPS[1], GLONASS[2], and Galileo[3] as well as regional
radio based time signals provided by MSF[4] in the UK, DCF77[5] in Germany, and
WWVB[6] within the United States. A primary server is utilized by secondary
servers and clients. Since the primary server derives its time from a reference
clock it is also categorized as a stratum 1 server. The stratum designation
signifies two items, the first is the distance that the server is from a
reference clock(stratum 0), in this case one hop, and that this server provides
a high level of time accuracy. 

### Secondary Server

The second mode is being a secondary server. This
mode operates as a client to upstream primary servers and as a server to
downstream clients. There is a defined maximum of 16 stratum levels and each
secondary server will reflect their level depending on how far they are from a
primary server. Each increased stratum level indicates a decrease time accuracy
from the the higher level time source. Any system reporting a stratum level of
16 is understood to be unsynchronized. 

### Client

Lastly the third mode is client to which
most devices fall under.  A client references time from multiple available time
sources to synchronize its system time.

### NTP Uses

// TODO: this section can be cut down if we need space

While accurate time is useful the question remains about why it is needed for
devices such as computers, phones and within local area networks. Most if not
all computing type devices contain some local clock or time keeping mechanisms. 
Typically they are adjusted during the setup or initialization phase to the
current date and time. While the local system clock is able to keep track of
time, through frequency ticks, it is subject to the same interrupt process
delays as other system hardware and software. Over time the missed or delayed
interrupts can cause the system to fall behind real-time causing the system time
to become skewed[7]. Putting this into perspective it can be seen how each
device, on the network, if left to its own time keeping, would eventually fall
out of sync with other devices. For a single device this may not be concerning
as the time can be adjusted, however with logging, security, and within the
network this can have cascading problems.  

The first of these issues is the ability to schedule tasks.  Within current
operating systems tasks can be scheduled to run at specific dates and times in
order to perform maintenance, updates, or monitoring. These tasks can have
dependencies on other scheduled tasks running or checking dates and times for
file access and updates. Externally to the system other devices may depend on
the scheduled jobs placing needed files or updates within a specific time frame,
as time drifts between the systems this can lead to failed job processing and
race conditions. 

This leads to the second issue, auditing and logging. When creating log events,
troubleshooting issues or auditing security logins one of the many sources
referenced is the system or application logs. When reviewing the timeline of
events and seeing that the system clocks are not synchronized it places extra
effort in determining when systems were accessed and at what time relative to
each local system clock. Instead, if the systems were synchronized, dates and
times would line up accordingly and administrators could efficiently determine
root causes of errors. Other network devices such as firewalls can depend on
rulesets being configured to allow or deny access based on the current date and
time. Since these devices are regularly deployed in pairs security access
controls would show inconsistencies as devices would allow access early or
delayed depending on the firewalls system time. When combining multiple device
logs such as firewalls, systems and applications to audit compliance, assurance
and integrity can be brought into question as numerous time and date
inconsistencies have to be re-evaluated for proper alignment. With centralized
log aggregation of these entries can be dropped from processing as a previous
time period has been marked as processed or because the current time is
determined to be in the future.

The final item is security. Systems and services that require users to
authenticate need accurate dates and times across numerous systems. With the
Windows operating system, which utilizes the W32Time manager, any system looking
to join the Active Directory domain or obtain needed authorization Kerberos
tickets needs to have highly accurate and stable system time[8]. Accurate time
reduces the ability of attackers to perform replay attacks with expired Kerberos
tickets which utilize time stamps to limit the life of the ticket. Additionally,
the timestamps inform applications when to request a token refresh[9]. For other
web protocols, such as HTTPS, time and date settings are important for
validating and generating certificates. For example each website that is secured
using HTTPS presents a certificate which contains a _not valid before_ and _not
valid after_ entries which contain a date, time and timezone as seen below:

```
Signature Algorithm: sha256WithRSAEncryption
  Issuer: 
    C=US, O=DigiCert Inc, 
    OU=www.digicert.com, 
    CN=GeoTrust RSA CA 2018
  Validity
    Not Before: Jul  1 00:00:00 2021 GMT
    Not After : Mar  7 23:59:59 2022 GMT
```
**Figure 1.**: https://www.jhu.edu website certificate information

This allows systems to validate the security of the presented certificate from
the webserver.

### NTP Public Pool

Synchronized time utilizing NTP is depended upon by numerous services and
security mechanisms within typical network architectures. To support this need
for accurate time, public time sources are provided by the NTP pool project[10].
Servers are allocated by DNS round robin, where each client requests a forward
lookup of one of the following:

- 0.pool.ntp.org
- 1.pool.ntp.org
- 2.pool.ntp.org
- 3.pool.ntp.org

The DNS based pool set of returned NTP servers IP addresses rotates every hour.
Additionally, further continent zones and sub-zones exist to provide time
service asa close as possible to the intended client as seen in table 1 below.

\tablefirsthead{%
   \hline
       Area & Hostname \\\hline}
\bottomcaption{Continet Zones}
\begin{supertabular}{|c|c|}
\hline
Asia & asia.pool.ntp.org \\
Europe & europe.pool.ntp.org \\
North America & north-america.pool.ntp.org \\
Oceania & oceania.pool.ntp.org \\
South America & south-america.pool.ntp.org \\
\hline
\end{supertabular}

The available servers in the pool are volunteered by research organizations,
companies, and individuals helping to support the project. In order to add a
server to the available pool you must have a static IP address and a stable
Internet connection. If that criteria is met, you can configure your NTP servers
with a known good stratum 1 or 2 servers and make the appropriate firewall
allowances. To add the severs to the available pool, account registration is
needed at ntppool.org. Once you have added your server(s) to the pool they are 
monitored for connectivity and timing accuracy. The ntp project scores systems
as part of their monitoring and once a system reaches 20 points it can be added
to the public pool. If a server drops below a score of 10 it is removed from the
public pool availability.

# Related Work

While the intial specification for NTP was published in 1985[12] there has been
minimal  public analysis into using the NTP protocol as a covert channel. The
first implementation by Halvemaan and Lahaye utilizes a NTP covert channel
through tunneling, called NTPTunnel. This type of channel is also observed with
other common protocols such as DNS with Iodine as well as TCP and ICMP with
PTunnel. NTPTunnel utilizes the Field Type value within the NTP header to build
the initial client / server connection.  A modified Pytun implementation listens
for a specific client NTP packet with the extension field, _field type_ value of
FF(hex) 00(hex) to which it will respond to the client with a _field type_ value
of 00(hex) FF(hex).  This exchange allows the client and server to discover each
other without any pre-shared details.  After the tunnel is established, crafted
NTP packets are sent between the server and client over the tunnel utilizing the
_value_ field of the NTP extension field. The _extension field_ has a described
use as supporting the autokey security protocol which makes it difficult to
restrict or flag with IPS or IDS signature rules. However given the open nature
of NTP at this time, the use of this field or values within it can provide
credible detection of this covert channel. In order to obscure the payload from
analysis the data is encrypted with AES and utilizes a shared key between server
and client. 

A second NTP covert channel utilizes the NTP Timestamp Format, specifically the
32 bits representing the fraction of seconds within the timestamp format. The
establishment of this covert channel does require shared information between the
sender and receiver, however the receiver does not have to be the NTP server and
can be any host capable of listening to NTP traffic between the server and its
clients. The receiver listens on the network for the predetermined Initiation
pattern ($0x00 00 00 01$), Sequence pattern ($0xe9$, $0xab$, $0xcb$) of three 32
bit segments, and an end of message pattern ($0xeb$). In order to track the
messages between the client and receiver the _Peer Clock Precision_ field is
used. The detection within this channel is difficult without knowing the
sequences to look for. Additionally the data flows within the existing NTP
communication flow only using a small amount of storage within the existing
channel. However, based on the limitation of data that can be sent per message
analysis of NTP message volume may expose this channel.

Lastly is a more recent covert channel that uses NTP as a _Dead Drop_ utilizing
both the information NTP stores as part of its client / server communication,
such as its most recently used (MRU) list and retrieval through NTP query and
control messages. NTP has a number of service query and monitoring commands that
be used to acquire information about the status of the service. It was noted by
the authors that these requests were typically disabled by default however
testing with the public NTP pool revealed that queries were answered leading to
the likelihood that other environments would also respond to these queries. To
implement the first possible covert channel the peer list, which is used to keep
track of data about upstream time sources, such as poll, offset, jitter, delay,
refid, etc, sets it stratum level equal to 14. This indicates to any querying 
client that there is a covert message stored within the reference ID(_refid_)
field of the peer list. In the authors implementation the message is within the
IP address, stored in the _refid_ field, of which each octet represents an ASCII
value. A covert client can obtain this information by querying the peer status
of the covert server. The second covert channel utilizes the MRU list available
on every NTP instance. A crafted NTP packet is sent to a listening NTP instance,
which if not configured with restrictions any instance can respond to NTP
queries, where data can be set / updated within that instances MRU table. Once
these values are set a covert client can query for the instances MRU data which
it can then decode the covert message from. In each instance of these covert
channels any monitoring or warden device would observer this to be expected NTP
communications.

# Standard Implemenation 

- Based on the modes outlined in the background section we outline below the 
standard implementations that are suggested by pool.ntp.org and generally
accepted as BCP. [11]

## DMZ Access

In DMZ access, devices in a secure network synchronize their clocks via an NTP
server that is located in a controlled DMZ. That NTP server synchronizes its
clock by connecting to one or more trusted NTP pools. Any information that can
survive a stratum layer[^stratum] may reach devices through the DMZ.


```
Placeholder for direct 
access image / diagram.
```
**Figure 2.** The DMZ Access NTP scenario, where an NTP server in a DMZ
serves replies to clients in a secure inner network.

## Direct Access

In direct access, devices in a secure network synchronize their clocks directly
via NTP to a trusted, publicly available, pool. Any information that the NTP
server can pack into an NTP reply may reach devices inside this network.

```
Placeholder for direct 
access image / diagram.
```
**Figure 3.** The Direct Access NTP scenario, where clients in a secure inner
network are allowed direct access to NTP pool servers.


[^stratum]:  In NTP, a stratum is a layer of devices that are the same distance
from a reference clock. Reference clocks, considered a source of truth, are at
stratum layer $0$. Servers that rely on reference clocks are at stratum layer
$1$, and so on. In our DMZ scenario, if public pools are at stratum $1$, then
the DMZ NTP server is at stratum $2$, and internal devices are at stratum $3$.

# Covert Implementation
 
// TODO: I'll rewrite this section with my implementation details
 - How we integrated our BPF filter into common deisgn 

## Throughput

 - The throughput of our implementation

## Robustness

 - The robustness of our implemenation

## Detection 

 - How someone would detect our implemenation


# Conclusions and Future Work

Deploying content to one service and exploiting automated idempotence to further
the propagation of the exploit.  You could use puppet or ansible which is
configured to return a directory or file to a previously well-known hashedstate.
The replacing of our covert file 


\pagebreak
# References

\nocite{*}
\bibliography{paper}

**TODO**: format this with bibtex

[1] https://www.gps.gov/applications/timing/
[2] https://gssc.esa.int/navipedia/index.php/GLONASS_General_Introduction 
[3] https://gssc.esa.int/navipedia/index.php/Galileo_General_Introduction
[4] https://www.npl.co.uk/msf-signal 
[5] https://www.ptb.de/cms/en/ptb/fachabteilungen/abt4/fb-44/ag-442/dissemination-of-legal-time/dcf77.html
[6] https://www.nist.gov/pml/time-and-frequency-division/time-distribution/radio-station-wwvb 
[7] https://en.wikipedia.org/wiki/Clock_skew
[8] https://docs.microsoft.com/en-us/windows-server/networking/windows-time-service/support-boundary
[9] https://docs.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2012-r2-and-2012/jj852172(v=ws.11)
[10] https://www.ntppool.org/en/
[11] https://www.rfc-editor.org/rfc/rfc8633.txt
[12] https://datatracker.ietf.org/doc/html/rfc958

\End{multicols}
