#define KBUILD_MODNAME "timebase"
#define asm_volatile_goto(x...) asm volatile("invalid use of asm_volatile_goto")
#include <linux/kconfig.h>
#include <uapi/linux/bpf.h>
#include <uapi/linux/if_packet.h>
#include <uapi/linux/if_ether.h>
#include <uapi/linux/ip.h>
#include <uapi/linux/udp.h>
#include <uapi/linux/pkt_cls.h>

#define IPPROTO_UDP 17
#define VALUE_LEN 8

static unsigned long long (*bpf_get_smp_processor_id)(void) =
    (void *)8;
static int (*bpf_perf_event_output)(void *ctx, void *map, unsigned long long flags, void *data, int size) =
    (void *)25;
static int (*bpf_skb_store_bytes)(struct __sk_buff *skb, u32 offset, const void *from, u32 len, u64 flags) =
    (void *)9;
static int (*bpf_skb_change_tail)(struct __sk_buff *skb, u32 len, u64 flags) =
    (void *)38;

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
    int retcode;
};

struct ntp_extensionless {
    u8 lvm;
    u8 stratum;
    u8 poll;
    u8 precision;
    u32 root_delay;
    u32 root_dispersion;
    u32 reference_id;
    u64 reference_ts;
    u64 originate_ts;
    u64 receive_ts;
    u64 transmit_ts;
};

struct extension_field {
    u16 field_type;
    u16 field_len;
    u8 value[VALUE_LEN];
}  __attribute__ ((aligned (4)));

__attribute__((section("classifier/ntp_filter"), used)) int ntp_filter(struct __sk_buff *ctx) {
	void *data = (void *)(long)ctx->data;
	void *data_end = (void *)(long)ctx->data_end;
	struct ethhdr *eth = data;
	if ((void*)eth + sizeof(*eth) > data_end)
		return TC_ACT_OK;
	struct iphdr *ip = data + sizeof(*eth);
	if ((void*)ip + sizeof(*ip) > data_end)
		return TC_ACT_OK;

	if (ip->protocol != IPPROTO_UDP)
		return TC_ACT_OK;
	struct udphdr *udp = (void *)ip + sizeof(*ip);
	if ((void*) udp + sizeof(*udp) > data_end)
		return TC_ACT_OK;

	if (udp->source == ntohs(123)) {
		struct ntp_extensionless *ntp = (void *)udp + sizeof(*udp);
		if ((void *) ntp + sizeof(*ntp) > data_end)
			return TC_ACT_OK;

		u32 offset = sizeof(*eth) + sizeof(*ip) + sizeof(*udp) + sizeof(*ntp);
		//u32 idx = ctx->ifindex;
		//u32 dst = udp->source;
		u32 ret = 0;
		//udp->dest = 0xdead;

		struct extension_field ef = {
		    /* *******************************************************
		     * R: 0 for query, 1 for response                        *
		     * |             E: 0 for OK, 1 for err                  *
		     * |             |                                       *
		     * |             |  Code (implementation specific)       *
		     * |             ||-|   Type (implementation specific)   *
		     * -------------|||     |                                *
		     *              VVV     V                                */
		    .field_type = 0b1000000000000000,
		    .field_len = sizeof(struct extension_field),
		    .value = {0, 0, 'H', 'E', 'L', 'L', 'O', '!'},
		};

		ret = bpf_skb_change_tail(ctx, offset + sizeof(struct extension_field), 0);

		struct ntp_filter_event tail_ev = {
			.retcode = ret,
		};
		bpf_perf_event_output(ctx,
		                      &ntp_filter_events,
		                      bpf_get_smp_processor_id(),
		                      &tail_ev,
		                      sizeof(tail_ev));

		ret = bpf_skb_store_bytes(ctx, offset, &ef,
					  sizeof(struct extension_field),
					  BPF_F_RECOMPUTE_CSUM);

		struct ntp_filter_event ev = {
			.retcode = ret,
		};
		bpf_perf_event_output(ctx,
				      &ntp_filter_events,
				      bpf_get_smp_processor_id(),
				      &ev,
				      sizeof(ev));
	}

	return TC_ACT_OK;
}

char _license[] __attribute__((section("license"), used)) = "Dual MIT/GPL";
uint32_t _version __attribute__((section("version"), used)) = 0xFFFFFFFE;
