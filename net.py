from mininet.net import Mininet
from mininet.util import dumpNodeConnections
from mininet.log import setLogLevel
from mininet.link import TCLink
from mininet.cli import CLI

TRACK_NAMESPACE = "bbb"

API_IP = "10.0.0.1"
DIR_IP = "10.0.0.2"
PUB_IP = "10.0.0.3"
SUB_IP = "10.0.0.4"
RELAY_1_IP = "10.0.0.5"
RELAY_2_IP = "10.0.0.6"


def build_api(net):
    api = net.addHost("api", ip=API_IP)
    api.cmd(f"RUST_LOG=debug RUST_BACKTRACE=0 ./target/moq-api")
    return api


def build_dir(net):
    dir = net.addHost("dir", ip=DIR_IP)
    dir.cmd(
        f"RUST_LOG=debug RUST_BACKTRACE=0 ./target/moq-dir"
        f" --tls-cert ./dev/localhost.crt"
        f" --tls-key ./dev/localhost.key"
    )
    return dir


def build_pub(net, id):
    pub = net.addHost(f"pub{id}", ip=PUB_IP)
    pub.cmd(
        f"ffmpeg -hide_banner -v quiet"
        f" -stream_loop -1 -re"
        f" -i ./dev/bbb.fmp4"
        f" -c copy"
        f" -f mp4 -movflags cmaf+separate_moof+delay_moov+skip_trailer+frag_every_frame"
        f" - | "
        f"RUST_LOG=debug RUST_BACKTRACE=0 ./target/moq-pub"
        f" --name {TRACK_NAMESPACE}"
        f" --url https://{RELAY_1_IP}"
    )
    return pub


def build_relay(net, id, ip):
    relay = net.addHost(f"relay{id}", ip=ip)
    relay.cmd(
        f"RUST_LOG=debug RUST_BACKTRACE=0 ./target/moq-relay-ietf"
        f" --tls-cert ./dev/localhost.crt"
        f" --tls-key ./dev/localhost.key"
        f" --tls-disable-verify"
        f" --api http://{API_IP}"
        f" --node https://{RELAY_1_IP}"
        f" --dev --announce https://{DIR_IP}"
    )
    return relay


def build_sub(net, id):
    sub = net.addHost(f"sub{id}", ip=SUB_IP)
    sub.cmd(
        f"RUST_LOG=debug RUST_BACKTRACE=0 ./target/moq-sub"
        f" --name {TRACK_NAMESPACE}"
        f" --url_primary https://{RELAY_1_IP}/{TRACK_NAMESPACE}"
        f" --url_secondary https://{RELAY_2_IP}/{TRACK_NAMESPACE}"
        f" | ffplay - -f mp4 -t 30 ha-bbb.mp4"
    )
    return sub


def build():
    net = Mininet(link=TCLink, controller=None)

    control_sw = net.addSwitch("api_sw")

    api = build_api(net)
    dir = build_dir(net)
    pub1 = build_pub(net, 1)
    relay1 = build_relay(net, 1, RELAY_1_IP)
    relay2 = build_relay(net, 2, RELAY_2_IP)
    sub1 = build_sub(net, 1)

    # Special links for API-* and DIR-* communications
    net.addLink(control_sw, api)
    net.addLink(control_sw, dir)
    net.addLink(control_sw, relay1)
    net.addLink(control_sw, relay2)

    linkopts = dict(bw=10, delay="5ms", loss=5)

    net.addLink(relay1, sub1, **linkopts)
    net.addLink(relay2, sub1, **linkopts)

    return net


if __name__ == "__main__":
    setLogLevel("info")
    net = build()
    dumpNodeConnections(net.hosts)

    net.start()
    CLI(net)
    net.stop()
