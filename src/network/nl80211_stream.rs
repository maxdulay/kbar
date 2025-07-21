use neli::{
    attr::Attribute,
    consts::{genl::*, nl::*},
    err::RouterError,
    genl::Genlmsghdr,
    nl::{NlPayload, Nlmsghdr},
    types::Buffer,
};

#[neli::neli_enum(serialized_type = "u8")]
pub enum Nl80211Command {
    Unspecified = 0,
    GetWiPhy = 1,
    GetInterface = 5,
    GetStation = 17,
    Connect = 46,
    Disconnect = 48,
    /* Many many more elided */
}
impl neli::consts::genl::Cmd for Nl80211Command {}

#[neli::neli_enum(serialized_type = "u16")]
pub enum Nl80211Attribute {
    Unspecified = 0,
    Wiphy = 1,
    WiphyName = 2,
    Ifindex = 3,
    Iftype = 5,
    Ssid = 52,
    StaInfo = 21,
    /* Literally hundreds elided */
}
impl neli::consts::genl::NlAttrType for Nl80211Attribute {}

#[neli::neli_enum(serialized_type = "u16")]
pub enum Nl80211StaInfo {
    Invalid = 0,
    InactiveTime = 1,
    RxBytes = 2,
    TxBytes = 3,
    Llid = 4,
    Plid = 5,
    PlinkState = 6,
    Signal = 7,
    TxBitrate = 8,
    RxPackets = 9,
    TxPackets = 10,
    TxRetries = 11,
    TxFailed = 12,
    SignalAvg = 13,
    RxBitrate = 14,
    BssParam = 15,
    ConnectedTime = 16,
    StaFlags = 17,
    BeaconLoss = 18,
    TOffset = 19,
    LocalPm = 20,
    PeerPm = 21,
    NonpeerPm = 22,
    RxBytes64 = 23,
    TxBytes64 = 24,
    ChainSignal = 25,
    ChainSignalAvg = 26,
    ExpectedThroughput = 27,
    RxDropMisc = 28,
    BeaconRx = 29,
    BeaconSignalAvg = 30,
    TidStats = 31,
    RxDuration = 32,
    Pad = 33,
}
impl NlAttrType for Nl80211StaInfo {}

#[derive(Clone, Debug)]
pub enum Nl80211Event {
    Connect(String),
    Disconnect,
}

#[derive(Clone, Debug)]
pub enum Event {
    Unspecified,
    GetWiPhy,
    GetInterface,
    Connect(Option<u32>, Option<u32>),
    Disconnect,
    UnrecognizedConst(u8),
}

pub fn parse_event(
    nlmsghdr: Nlmsghdr<GenlId, Genlmsghdr<Nl80211Command, Nl80211Attribute>>,
) -> Result<Event, RouterError<u16, Buffer>> {
    let payload = match nlmsghdr.nl_payload() {
        NlPayload::Payload(p) => p,
        _ => return Err(RouterError::new("Bad Payload")),
    };
    let event = match payload.cmd() {
        Nl80211Command::Unspecified => Event::Unspecified,
        Nl80211Command::GetWiPhy => Event::GetWiPhy,
        Nl80211Command::GetInterface => Event::GetInterface,
        Nl80211Command::Connect => {
            let mut wiphy = None;
            let mut ifindex = None;
            for attr in payload.attrs().get_attr_handle().iter() {
                match attr.nla_type().nla_type() {
                    Nl80211Attribute::Wiphy => {
                        wiphy = Some(attr.get_payload_as::<u32>().unwrap());
                    }
                    Nl80211Attribute::Ifindex => {
                        ifindex = Some(attr.get_payload_as::<u32>().unwrap());
                    }
                    _ => ()
                }

            }
            Event::Connect(wiphy, ifindex)
        },
        Nl80211Command::Disconnect => Event::Disconnect,
        Nl80211Command::UnrecognizedConst(i) => Event::UnrecognizedConst(*i),
        _ => Event::Unspecified
    };
    Ok(event)
}

