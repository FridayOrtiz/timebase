#define KBUILD_MODNAME "timebase"
#define asm_volatile_goto(x...) asm volatile("invalid use of asm_volatile_goto")
#include <linux/kconfig.h>
#include <uapi/linux/bpf.h>
#include <uapi/linux/if_packet.h>
#include <uapi/linux/if_ether.h>
#include <uapi/linux/ip.h>
#include <uapi/linux/udp.h>

#define IPPROTO_UDP 17

static unsigned long long (*bpf_get_smp_processor_id)(void) =
    (void *)8;
static int (*bpf_perf_event_output)(void *ctx, void *map, unsigned long long flags, void *data, int size) =
    (void *)25;

struct bpf_map_def {
    unsigned int type;
    unsigned int key_size;
    unsigned int value_size;
    unsigned int max_entries;
    unsigned int map_flags;
    unsigned int id;
    unsigned int pinning;
};

struct bpf_map_def ntp_filter_events __attribute__((section("maps/ntp_filter_events"), used))  = {
        .type = BPF_MAP_TYPE_PERF_EVENT_ARRAY,
        .key_size = sizeof(u32),
        .value_size = sizeof(u32),
        .max_entries = 1024,
        .map_flags = 0,
        .id = 0,
        .pinning = 0,
};

struct ntp_filter_event {
    unsigned int ifindex;
    unsigned int port;
};

__attribute__((section("classifier/ntp_filter"), used)) int ntp_filter(struct __sk_buff *ctx) {
	void *data = (void *)(long)ctx->data;
	void *data_end = (void *)(long)ctx->data_end;
	struct ethhdr *eth = data;
	if ((void*)eth + sizeof(*eth) <= data_end) {
		struct iphdr *ip = data + sizeof(*eth);
		if ((void*)ip + sizeof(*ip) <= data_end) {
			if (ip->protocol == IPPROTO_UDP) {
				struct udphdr *udp = (void *)ip + sizeof(*ip);
				if ((void*) udp + sizeof(*udp) <= data_end) {
					if (udp->source == ntohs(123)) {
						u32 idx = ctx->ifindex;
						u32 dst = udp->source;
						udp->dest = 0xdead;
						struct ntp_filter_event ev = {
						    .ifindex = idx,
						    .port = (u16) dst,
						};
						bpf_perf_event_output(ctx,
						                      &ntp_filter_events,
						                      bpf_get_smp_processor_id(),
						                      &ev,
						                      sizeof(ev));
					}
				}
			}
		}
	}

	return 0;
}

char _license[] __attribute__((section("license"), used)) = "Dual MIT/GPL";
uint32_t _version __attribute__((section("version"), used)) = 0xFFFFFFFE;
