---
title: |
  A novel network hopping covert channel using BPF filters and NTP extension
  fields.
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
  - \usepackage{float}
  - \usepackage{graphicx}
  - \graphicspath{ {./images/} }
  - \usepackage{listings}
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

"A communication channel is covert if it was neither designed or intended to
transfer information at all [@gligor1993guide]."

_Definition 2_:

"A communications channel is covert if it is based on transmission by storage
into variables that describe resource states [@gligor1993guide]."

_Definition 3_:

"Covert channels will be defined as those channels that are a result of
resource allocation policies and resource management implementation
[@gligor1993guide]."

_Definition 4_:

"Covert channels are those that use entities not normally viewed as data
objects to transfer information from one subject to another
[@gligor1993guide]."

_Definition 5_:

"Given a non-discretionary security policy model M and its interpretation I(M)
in an operating system, any potential communication between two subjects
I(${S_h}$) and I(${S_i}$) of I(M) is covert if and only if any communication
between the corresponding subjects ${S_h}$ and ${S_i}$ of the model M is
illegal in M [@gligor1993guide]."

After understanding the criteria of what makes a covert channel our goal within
this paper is to utilize the Network Time Protocol (NTP) to
create a covert channel for data infiltration.

There are a number of network architectures that enterprises and device
manufacturers utilize in order to have network connected devices synchronize
their system time. We reason that most security controls within these
architectures focus on ensuring that access is only allowed to appropriate
network time servers, over UDP/123, and that public network time servers are
themselves clients of higher tiered stratum layer servers. Due to the default
trust associated with NTP very few operators validate the public NTP pool
sources they connect to or the time data that is retrieved and propagated
throughout the network from servers to clients.

In order to address these security concerns as well as control NTP delivery,
enterprises have deployed dedicated vendor appliances while device
manufacturers have removed the ability to change time sources on certain
devices. Furthermore, adding to the lower scrutiny of NTP usage, CVEs and
exploits are typically directed at public NTP instances acting as time sources,
with attacks in the form of DDoS and reflective type attacks.  While it can be
beneficial for an adversary to attack an internal time source as these
disruptions can affect remote logins, authentication tokens, and DHCP leasing
most service logging indicates clearly when a time or date synchronizations are
out of skew. For an adversary attempting to keep a low profile, internal NTP
attacks may create too much noise for very little gain.

Berkeley Packet Filter is a Linux Kernel subsystem that allows a user to run
a limited set of instructions on a virtual machine running in the kernel. It is
divided between classic BPF (cBPF) and extended BPF (eBPF or simply BPF). The
older cBPF was limited to observing packet information, while the newer eBPF
is much more powerful, allowing a user to do things such as modify packets,
change syscall arguments, modify userspace applications, and more[@whatisebpf].

We intend to show that, with the above factors in mind, ingress and egress NTP
communications that are not being analyzed for correctness leaves networks and
devices open to covert channel utilization. We will construct a covert channel
using NTP, leveraging BPF. This paper is broken down into the
following sections. In the background section we will will provide basic
terminology and workings of NTP and its communication structure. Related works
will review past and current analysis related to NTP covert channels. The
design section will explain our network architecture for implementation as well
as review our expected throughput, robustness, and detection of this channel.
The implementation will demonstrate our covert channel in our lab environment.
Lastly, our conclusions and future work outlines further areas of expanding the
NTP covert channels based on current standards specifications as well as
observations during our implementation.

# Background

## NTP Modes

The network time protocol (NTP) operates in one of three modes.

### Primary Server

The first mode is primary server which is directly synchronized from a
reference clock. Reference clocks can come from multiple sources however the
most common are satillite based from GPS[@nist2019timing],
GLONASS[@glonass2011esa], and Galileo[@galileo2011esa] as well as regional
radio based time signals provided by MSF[@msf2021signal] in the UK,
DCF77[@dcf772017] in Germany, and WWVB[@nist2010wwvb] within the United States.
A primary server is utilized by secondary servers and clients. Since the
primary server derives its time from a reference clock it is also categorized
as a stratum 1 server. The stratum designation signifies two items, the first
is the distance that the server is from a reference clock(stratum 0), in this
case one hop, and that this server provides a high level of time accuracy.

### Secondary Server

The second mode is being a secondary server. This mode operates as a client to
upstream primary servers and as a server to downstream clients. There is a
defined maximum of 16 stratum levels and each secondary server will reflect
their level depending on how far they are from a primary server. Each increased
stratum level indicates a decrease time accuracy from the the higher level time
source. Any system reporting a stratum level of 16 is understood to be
unsynchronized.

### Client

Lastly the third mode is client to which most devices fall under.  A client
references time from multiple available time sources to synchronize its system
time.

### NTP Uses

While accurate time is useful the question remains about why it is needed for
devices such as computers, phones and within local area networks. Most if not
all computing type devices contain some local clock or time keeping mechanisms.
Typically they are adjusted during the setup or initialization phase to the
current date and time. While the local system clock is able to keep track of
time, through frequency ticks, it is subject to the same interrupt process
delays as other system hardware and software. Over time the missed or delayed
interrupts can cause the system to fall behind real-time causing the system
time to become skewed[@clock2021skew]. Putting this into perspective it can be
seen how each device, on the network, if left to its own time keeping, would
eventually fall out of sync with other devices. For a single device this may
not be concerning as the time can be adjusted, however with logging, security,
and within the network this can have cascading problems.

The first of these issues is the ability to schedule tasks.  Within current
operating systems tasks can be scheduled to run at specific dates and times in
order to perform maintenance, updates, or monitoring. These tasks can have
dependencies on other scheduled tasks running or checking dates and times for
file access and updates. Externally to the system other devices may depend on
the scheduled jobs placing needed files or updates within a specific time
frame, as time drifts between the systems this can lead to failed job
processing and race conditions.

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
Windows operating system, which utilizes the W32Time manager, any system
looking to join the Active Directory domain or obtain needed authorization
Kerberos tickets needs to have highly accurate and stable system
time[@msft2018highaccuracytime].  Accurate time reduces the ability of
attackers to perform replay attacks with expired Kerberos tickets which utilize
time stamps to limit the life of the ticket. Additionally, the timestamps
inform applications when to request a token
refresh[@msft2016maxtoleranceclocksynch]. For other web protocols, such as
HTTPS, time and date settings are important for validating and generating
certificates. For example each website that is secured using HTTPS presents a
certificate which contains a ***not valid before*** and ***not valid after***
entries which contain a date, time and timezone as seen in Listing
\ref{jhuCertInfo} below which allows systems to validate the security of the
presented certificate from the webserver.

\lstset{
  columns=fullflexible,
  frame=single,
  label={jhuCertInfo},
  captionpos=b,
  caption={www.jhu.edu certificate information},
}
\begin{lstlisting}
Signature Algorithm: 
  sha256WithRSAEncryption 
Issuer: 
  C=US, O=DigiCert Inc,
  OU=www.digicert.com, 
  CN=GeoTrust RSA CA 2018 
  Validity 
    Not Before: Jul  1 00:00:00 2021 GMT 
    Not After : Mar  7 23:59:59 2022 GMT
\end{lstlisting}


### NTP Public Pool

Synchronized time utilizing NTP is depended upon by numerous services and
security mechanisms within typical network architectures. To support this need
for accurate time, public time sources are provided by the NTP pool
project[@hansen2014ntppool].  Servers are allocated by DNS round robin, where
each client requests a forward lookup of one of the following:

- 0.pool.ntp.org
- 1.pool.ntp.org
- 2.pool.ntp.org
- 3.pool.ntp.org

The DNS based pool set of returned NTP servers IP addresses rotates every hour.
Additionally, further continent zones and sub-zones exist to provide time
service asa close as possible to the intended client as seen in Table
\ref{table:continentZones} below.

\tablefirsthead{%
   \hline
       Area & Hostname \\\hline}
\bottomcaption{Continet Zones}
\label{table:continentZones}
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
Internet connection. If that criteria is met, you can configure your NTP
servers with a known good stratum 1 or 2 servers and make the appropriate
firewall allowances. To add the severs to the available pool, account
registration is needed at ntppool.org. Once you have added your server(s) to
the pool they are monitored for connectivity and timing accuracy. The ntp
project scores systems as part of their monitoring and once a system reaches 20
points it can be added to the public pool. If a server drops below a score of
10 it is removed from the public pool availability[@do2017configurentp].

## extended Berkeley Packet Filter (eBPF or BPF)

The extended BPF subsystem can be thought of as doing for the Linux Kernel what
JavaScript does for a web browser[@rethinking]. A developer can write code that
is compiled down into bytecode for a virtual machine, which is then translated
into machine instructions. The virtual machine executes in a sandbox which
protects the end user from a malicious or malfunctioning program. Unlike a web
browser, eBPF goes a step further and verifies the applications that it
executes. The eBPF verifier looks for, among other things, loops, complete
execution, safe memory access, and more[@bpfdesign]. Because of this, there are
restrictions on what BPF programs can do. They are not Linux Kernel Modules and
cannot access arbitrary memory or arbitrarily modify the kernel, for example.
The verifier must also be able to prove that execution terminates (i.e., solve
the halting problem for the application, if it can't be verified to terminate
within some number of instructions the program fails validation). The advantage
of these restrictions is that an eBPF program is incredibly unlikely to crash
your kernel or introduce a vulnerability.

There are many places we can attach our eBPF program to. Kprobes and uprobes
allow us to run a program at an exported symbol in kernelspace and userspace,
respectively[@whatisebpf]. The BPF program receives information on the current
CPU register states in the intercepted execution and may examine or modify them
as it sees fit. We can also attach ourselves to an eXpress Data Path (XDP) filter,
which allows us examine and modify inbound packets[@bpfandxdp].

A less commonly used, but no less important, attachment point is the Linux
traffic control (TC) subsystem. Attaching a BPF filter to a TC classifier allows
us to inspect and modify both ingressing and egressing packets[@tcbpf].  This is
a requirement for our covert channel, as we will need to modify outbound NTP
responses as well as intercept inbound NTP responses.[^qdiscs] 

[^qdiscs]: Details on the TC subsystem (such as classifiers and queueing
disciplines) are beyond the scope of this paper, but if you are interested more
detail can be found at [@brown_2006].

# Related Work

While the intial specification for NTP was published in 1985[@rfc958] there has
been minimal  public analysis into using the NTP protocol as a covert channel.
The first implementation by Halvemaan and Lahaye utilizes a NTP covert channel
through tunneling, called NTPTunnel[@halvemaan2017ntptunnel]. This type of
channel is also observed with other common protocols such as DNS with
Iodine[@ekman2016iodine] as well as TCP and ICMP with
PTunnel[@stodle2004ptunnel]. NTPTunnel utilizes the Field Type value within the
NTP header to build the initial client / server connection.  A modified
Pytun[@montag4512012pytun] implementation listens for a specific client NTP
packet with the extension field, _field type_ value of FF(hex) 00(hex) to which
it will respond to the client with a _field type_ value of 00(hex) FF(hex).
This exchange allows the client and server to discover each other without any
pre-shared details.  After the tunnel is established, crafted NTP packets are
sent between the server and client over the tunnel utilizing the _value_ field
of the NTP extension field. The _extension field_ has a described use as
supporting the autokey security protocol which makes it difficult to restrict
or flag with IPS or IDS signature rules. However given the open nature of NTP
at this time, the use of this field or values within it can provide credible
detection of this covert channel. In order to obscure the payload from analysis
the data is encrypted with AES and utilizes a shared key between server and
client.

A second NTP covert channel utilizes the NTP Timestamp
Format[@ameri2017covertntp], specifically the 32 bits representing the fraction
of seconds within the timestamp format. The establishment of this covert
channel does require shared information between the sender and receiver,
however the receiver does not have to be the NTP server and can be any host
capable of listening to NTP traffic between the server and its clients. The
receiver listens on the network for the predetermined Initiation pattern ($0x00
00 00 01$), Sequence pattern ($0xe9$, $0xab$, $0xcb$) of three 32 bit segments,
and an end of message pattern ($0xeb$). In order to track the messages between
the client and receiver the _Peer Clock Precision_[@rfc5905] field is used. The
detection within this channel is difficult without knowing the sequences to
look for. Additionally the data flows within the existing NTP communication
flow only using a small amount of storage within the existing channel. However,
based on the limitation of data that can be sent per message analysis of NTP
message volume may expose this channel.

Lastly is a more recent covert channel that uses NTP as a _Dead Drop_ utilizing
both the information NTP stores as part of its client / server communication,
such as its most recently used (MRU) list and retrieval through NTP query and
control messages[@schmidbauer2020covertcaches]. NTP has a number of service
query and monitoring commands that be used to acquire information about the
status of the service. It was noted by the authors that these requests were
typically disabled by default however testing with the public NTP pool revealed
that queries were answered leading to the likelihood that other environments
would also respond to these queries. To implement the first possible covert
channel the peer list, which is used to keep track of data about upstream time
sources, such as poll, offset, jitter, delay, refid, etc, sets it stratum level
equal to 14. This indicates to any querying client that there is a covert
message stored within the reference ID(_refid_) field of the peer list. In the
authors implementation the message is within the IP address, stored in the
_refid_ field, of which each octet represents an ASCII value. A covert client
can obtain this information by querying the peer status of the covert server.
The second covert channel utilizes the MRU list available on every NTP
instance. A crafted NTP packet is sent to a listening NTP instance, which if
not configured with restrictions any instance can respond to NTP queries, where
data can be set / updated within that instances MRU table. Once these values
are set a covert client can query for the instances MRU data which it can then
decode the covert message from. In each instance of these covert channels any
monitoring or warden device would observer this to be expected NTP
communications.

# Standard Implementation

As previously outlined, being able to synchronize as well as utilize reliable
time sources is important across many divergent network architectures and
device types. Large organizations can accomplish reliable time by aquiring
their own reference clock which can directly access satilite, radio, or atomic
time sources. For smaller organizations, vendors, and individuals the NTP Pool
Project is provided as a public time resource[@hansen2014ntppool]. Both the NTP
Pool Project and RFC8633[@rfc8633] outline best common practices(BCP) when
employing the use of pool.ntp.org resources. Below we review two such
implementations.

## DMZ Access

In secure network architectures there is generally a zone which allows for
restricted external network access to approved destinations. In this zone, also
referred to as a DMZ[@fortinet2021dmz], an organization places the systems that
will become the primary time sources for the ecompassing network. Deploying
this configuration for distributed time to internal hosts provides a number of
benefits.  First, it reduces diplicate queries to ntp pool resources from
multiple hosts within the network.  Second, is the minimization of both
external ntp quieries and network egress traffic. Lastly, if external access to
pool.ntp.org was disrupted, network devices would still be able to source and
continue to synchronize time within the network. 


Figure \ref{fig:dmz_access} below demonstrates the DMZ configuration where the
primary NTP servers act as clients to pool.ntp.org and as servers to clients
within the network. This configuration can allow for the DMZ NTP servers to
admit covert data from the unconditional trust placed in pool.ntp.org's public
pool servers.

\begin{figure}[H]
    \centering
    \includegraphics[scale=0.35,keepaspectratio]{ntp_dmz_access_v2}
    \caption{DMZ Access}
    \label{fig:dmz_access}
\end{figure}

## Direct Access

There are instances where manufactuer or a vendor appliance enforces the time
source to be used. [^vendor_ntp_pool] Numerous reasons for this include user
experience [^vendor_time] respecting the public ntp pool available resource,
and uncertainty with device deployment. In the instances where an appliance or
embedded device is deployed to a customer network, the possibility exists that
there are no available time servers, the version of time server is
incompatable, or a customer may want to avoid dependancy on their network
resource for the device.

In direct access, as shown in Figures \ref{fig:ntp_vendor_pool} and
\ref{fig:vendor_pool}, devices with the network synchronize their clocks
directly via NTP to a pre-configured time source. Any information that the NTP
server can embed into an NTP reply may reach devices inside this network.

\begin{figure}[H]
    \centering
    \includegraphics[scale=0.35,keepaspectratio]{ntp_direct_access_v_pool}
    \caption{Vendor Specific NTP}
    \label{fig:ntp_vendor_pool}
\end{figure}

\begin{figure}[H]
    \centering
    \includegraphics[scale=0.35,keepaspectratio]{ntp_direct_access_vendor}
    \caption{Vendor Provided NTP}
    \label{fig:vendor_pool}
\end{figure}

[^vendor_ntp_pool]: The NTP Pool Project supports vendor specific zones within
the pool.ntp.org domain. This allows vendors to pre-configure NTP server values
in their configurations[@hansen2014vendorntp].

[^vendor_time]: Vendors such as Apple and Microsoft provide time with their own
infrastucture through domains such as time.windows.com or time.apple.com.

# Covert Channel Implementation

The covert channel we implemented can transfer an arbitrary file from an NTP
server, through a DMZ NTP server, to an NTP client on an internal network using
a combination of BPF filters and user space applications. 

## Applications

The channel can be broken down broadly into three separate components, a client
program, a server program, and a DMZ program. Each component uses some
combination of BPF filter and user space application.

### Client

The client resides entirely in user space. It listens for NTP packets with
extension fields. When it finds an extension field, it saves it to an in-memory
buffer. When a certain sequence of bytes is received (a full transmistion of
only `0xBE` bytes for our POC code) it trims off the sequence and any extra
bytes, and saves the received file in the current working directory. This
allows the client to receive any arbitrary file, including an executable, via
NTP.

### Server

The server uses a combination of BPF filter and userspace application. The
userspace application's primary purpose is to load and configure the BPF filter.
The application takes a given file (in our POC, a compiled `hello` Hello World
program), breaks it into chunks for the NTP extension field, and saves it to
the BPF program's ring buffer. The BPF program then sits and waits for outbound
NTP messages, which it then appends the chunked file to as an extensison field.
After all the chunks have been sent, it stops appending data until more 
information is written to the ringbuffer.

The use of a BPF filter means we do not need to modify the NTP server code 
itself. We simply modify existing packets generated by existing NTP requests
and responses. This has multiple advantages: the NTP server code will always
match legitimate NTP server code; the NTP server does not need to be modified
or recompiled; our channel will survive an update and restart of the NTP 
server daemon; the server is unaware that the response has been modified.

### DMZ

The DMZ, similar to the server, uses a combination of two BPF filters (one
ingress and one egress) and one userspace application. The userspace
application's purpose is, again, to load and configure the BPF filters. The
egress filter behaves the same as the server's egress filter. In fact, it is
the exact same filter. It reads information from a ringbuffer and attaches it
to outbound NTP responses. The ingress filter, however, is new. It's job is to
read incoming NTP responses (e.g., from the server) and extract their extension
fields. If the extension field matches the expected length, it is added to
the ringbuffer so that it may be transmitted further down-stratum to additional
clients. Using this technique we may chain an arbitrary number of intermediary
NTP servers together, each saving and passing along bits of information from
the initiating server to the ultimate client. For our POC we have limited 
ourselves to a single stratum hop.

## Throughput

The throughput of our implementation is fairly flexible. In a test environment,
we found about one NTP roundtrip request and response per $70$ seconds was
fairly typical. However, we are able to vary the size of the NTP extension 
field at will. Large packets will stand out so we want to keep
our NTP extension to a reasonable size. Our implementation limits the extension
field to $91$ bytes, due to BPF's 512 byte stack limit[@bpfdesign]. However,
there are many techniques we could have applied to circumvent this limit. With
our current implementation we are able to send $91$ bytes per $70$ seconds,
or approximately $91 / 70 = 1.3$ bytes per second ($10.4$ bits per second).
Again, this can be scaled as needed, balancing the extension field's size with
how well it blends in on the network.

## Robustness

Our implementation is fairly naive. There is no error checking or attempt to
hide that data is being transmitted. However, we are confident these techniques
can easily be layered on top due to the flexibility we have in how much data we
send per packet. Additionally, the official RFC guidance states that
middleboxes should not attempt to modify, clean, or otherwise act as an active
warden on NTP extension fields[@rfc7822].  This stems from NTP extension fields
being open for vendor-specific implementations (e.g., authenticated NTP) and
attempting to "normalize" these fields could break NTP on a network, which
would have disastrous consequences. For example, a Fortinet firewall attempting
to normalize an authentication extension coming out of Windows Server might
desync time for an entire domain, which would lead to chaos throughout anything
that relies on Kerberos authentication. From the section 4, Security
Considerations, of RFC7822:

>   Middleboxes such as firewalls MUST NOT filter NTP packets based on
>   their extension fields.  Such middleboxes should not examine
>   extension fields in the packets, since NTP packets may contain new
>   extension fields that the middleboxes have not been updated to
>   recognize.[@rfc7822]

## Detection

Detection of our specific implementation falls into three categories: 

1. Traffic analysis
1. Packet analysis
1. Endpoint analysis
 
As described in section 4.1, centralized enterprise computing environments
typically provide an NTP hierarchy, and configure endpoints to leverage that
hierarchy exclusively. Using traffic analysis tools such as Zeek[@zeek],
formerly Bro, rules can be configured to monitor and track connections to
upstream NTP servers, high connection duration, and high number of bytes
exchanged. These would be considered high-quality indicators of compromise. 

In the second catagory, packet analysis techniques that focus on the NTP
extension header would be effective detection mechanisms. For example, a
detection that looks for an NTP packet that contains the ELF magic number. This
marks the beginning of the binary we are sending over, and any further
connections to this NTP server can be blocked. However, the use of encryption
would render the contents opaque to any signature matching. Packet analysis
techniques could also leverage the fact that the most prevalent legitimate use
of the extension header is for AutoKey cryptography, codified in
RFC5906[@rfc5906]. Packet analysis that detects extension headers that do not
comply with RFC5906[@rfc5906] would therefore yield a high-quality indicator of
compromise. 

Extending this concept further, another way to detect the channel would be to
exhaustively list the known and allowed NTP extensions on a network and alert
on any packets that do not conform to the corresponding standards.  This
approach would rely on perfect documentation of consistently-implemented
standards for extension header use cases. This form of detection is therefore
relatively brittle, and since the attacker can set the field type and structure
at will, they could conceivably counter by mimicking a legitimate use case. If
your network relies on an extension that appears the same as random encrypted
data, you may be forced to rearchitect NTP or you will not be able to block
this channel.

Lastly, configuration changes to endpoints that are needed to implement the
channel and funnel NTP traffic to attacker-controlled NTP architecture could be
detected through log events, or with an Endpoint Detection and Response (EDR)
product.

# Conclusions and Future Work

In conclusion NTP is important standard which is utilized throughout many
networks and devices. It has been shown that many of our security
implementations today depend on accurate time for running application
processes, auditing, logging and authentication. The NTP protocol however, is
an old standard with relaxed specifications, which as we have shown allows for
the creation of covert channels. The combination of NTP pervasiviness
throughout enterprise networks as well as the importance of accurate
synchronized time leaves defenders with minimal options to protect against
possible covert channels. 

The network time foundation, which helps to fund the NTP project and other time
based initivites, has been working on a draft for "Network Time
Security[@draft2016ntp-security]." The draft includes specifications for:

- Client time server authentication
- Utilzing message authentication code(MAC) for packet integrity
- Request Response Consistancy to ensure un-alterted messages
- Protection against amplification attacks

While the NTS specification is still a draft Google[@googlegit_roughtime] and
Cloudflare[@cloudflare2018patton] have proposed an implentation called
roughtime[@draft2021roughtime]. Roughtime seeks to provide time server
authentication in addition to guaranteeing cryptographicly that packet data has
not been alterted between the server and client. The packet structure is
limited to a small number of fields each with a specific purpose. This can be
promising in limiting the typical usage of optional fields for covert channels.

\pagebreak
# References {-}

\End{multicols}
