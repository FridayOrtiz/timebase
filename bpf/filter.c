#define KBUILD_MODNAME "timebase"
#include <uapi/linux/bpf.h>


__attribute__((section("xdp/tcp_filter"), used)) int tcp_filter(struct xdp_md *ctx) {
	return XDP_PASS;
}

// kernel recognizes MIT license as Proprietary, more or less
char _license[] __attribute__((section("license"), used)) = "Proprietary";
uint32_t _version __attribute__((section("version"), used)) = 0xFFFFFFFE;
