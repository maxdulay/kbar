use std::io::Cursor;

use neli::{
    FromBytesWithInput, ToBytes,
    attr::{Attribute},
    consts::{ nl::*, socket::*},
    genl::{Genlmsghdr, GenlmsghdrBuilder, NlattrBuilder, NoUserHeader},
    nl::{NlPayload},
    router::synchronous::NlRouter,
    types::GenlBuffer,
    utils::Groups,
};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::network::nl80211_stream::{Nl80211Attribute, Nl80211Command, Nl80211StaInfo};

#[derive(Debug, Clone)]
pub enum Connection {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub struct NetworkState {
    ticks: u8,
    ifindex: u32,
    pub state: Connection,
    pub ssid: String,
    pub signal: usize,
}

impl NetworkState {
    pub fn new() -> Self {
        let mut netstate = Self {
            ticks: 0,
            ifindex: NetworkState::get_default_ifindex(),
            ssid: "Disconnected".to_string(),
            state: Connection::Disconnected,
            signal: 0,
        };
        if netstate.ifindex != 0 {
            netstate.set_ssid(netstate.ifindex);
            netstate.set_wifi_quality(netstate.ifindex);
        }
        netstate
    }

    pub fn disconnected(&mut self) {
        self.state = Connection::Disconnected;
        self.signal = 0;
        self.ssid = "Disconnected".to_string();
    }
    pub fn connected(&mut self, ifindex: u32) {
        self.state = Connection::Connected;
        self.set_ssid(ifindex);
        self.set_wifi_quality(ifindex);
    }
    
    pub fn tick(&mut self) {
        self.ticks += 1;
        if self.ticks >= 100 {
            self.set_wifi_quality(self.ifindex);
            self.ticks = 0;
        }
    }

    fn set_ssid(&mut self, ifindex: u32) {
        self.ifindex = ifindex;
        let attrs = vec![
            NlattrBuilder::<Nl80211Attribute, _>::default()
                .nla_type((u16::from(Nl80211Attribute::Ifindex)).into())
                .nla_payload(ifindex)
                .build()
                .unwrap(),
        ]
        .into_iter()
        .collect::<GenlBuffer<Nl80211Attribute, neli::types::Buffer>>();

        let (s, _) = NlRouter::connect(NlFamily::Generic, None, Groups::empty()).unwrap();
        let family_id = s.resolve_genl_family("nl80211").unwrap();
        let recv = s
            .send::<_, _, u16, Genlmsghdr<Nl80211Command, Nl80211Attribute>>(
                family_id,
                NlmF::REQUEST,
                NlPayload::Payload(
                    GenlmsghdrBuilder::<Nl80211Command, Nl80211Attribute, NoUserHeader>::default()
                        .cmd(Nl80211Command::GetInterface)
                        .version(1)
                        .attrs(attrs)
                        .build()
                        .unwrap(),
                ),
            )
            .unwrap();
        let msg = recv.into_iter().next().unwrap().unwrap();
        let payload = match msg.nl_payload() {
            NlPayload::Payload(p) => p,
            _ => return,
        };
        let attr_handle = payload.attrs().get_attr_handle();
        if let Some(attr) = attr_handle.get_attribute(Nl80211Attribute::Ssid) {
            self.state = Connection::Connected;
            let payload = attr.payload();
            let mut buf: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            let _ = payload.to_bytes(&mut buf);
            buf.set_position(0);
            self.ssid = String::from_bytes_with_input(&mut buf, payload.len() + 1).unwrap();
        }
        // TODO: Connections without SSIDS
    }

    pub fn get_default_ifindex() -> u32 {
        let (s, _) = NlRouter::connect(NlFamily::Generic, None, Groups::empty()).unwrap();
        let family_id = s.resolve_genl_family("nl80211").unwrap();
        let recv = s
            .send::<_, _, u16, Genlmsghdr<Nl80211Command, Nl80211Attribute>>(
                family_id,
                NlmF::DUMP | NlmF::REQUEST,
                NlPayload::Payload(
                    GenlmsghdrBuilder::<Nl80211Command, Nl80211Attribute, NoUserHeader>::default()
                        .cmd(Nl80211Command::GetInterface)
                        .version(1)
                        .build()
                        .unwrap(),
                ),
            )
            .unwrap();
        for msg in recv {
            let msg = msg.unwrap();
            let payload = match msg.nl_payload() {
                NlPayload::Payload(p) => p,
                _ => return 0,
            };
            let attr_handle = payload.attrs().get_attr_handle();
            let mut ifindex = 0;
            for attr in attr_handle.iter() {
                match attr.nla_type().nla_type() {
                    Nl80211Attribute::Ifindex => {
                        ifindex = attr.get_payload_as::<u32>().unwrap();
                    }
                    Nl80211Attribute::Iftype => {
                        if attr.get_payload_as::<u16>().unwrap() == 2 {
                            return ifindex;
                        }
                    }
                    _ => (),
                }
            }
        }
        return 0;
    }
    pub fn set_wifi_quality(&mut self, ifindex: u32) {
        let attrs = vec![
            NlattrBuilder::<Nl80211Attribute, _>::default()
                .nla_type((u16::from(Nl80211Attribute::Ifindex)).into())
                .nla_payload(ifindex)
                .build()
                .unwrap(),
        ]
        .into_iter()
        .collect::<GenlBuffer<Nl80211Attribute, neli::types::Buffer>>();

        let (s, _) = NlRouter::connect(NlFamily::Generic, None, Groups::empty()).unwrap();
        let family_id = s.resolve_genl_family("nl80211").unwrap();
        let recv = s
            .send::<_, _, u16, Genlmsghdr<Nl80211Command, Nl80211Attribute>>(
                family_id,
                NlmF::DUMP | NlmF::REQUEST,
                NlPayload::Payload(
                    GenlmsghdrBuilder::<Nl80211Command, Nl80211Attribute, NoUserHeader>::default()
                        .cmd(Nl80211Command::GetStation)
                        .version(1)
                        .attrs(attrs)
                        .build()
                        .unwrap(),
                ),
            )
            .unwrap();
        let msg = recv.into_iter().next().unwrap().unwrap();
        let payload = match msg.nl_payload() {
            NlPayload::Payload(p) => p,
            _ => return,
        };
        let attr_handle = payload.attrs().get_attr_handle();
        let station_attributes = attr_handle.get_nested_attributes::<Nl80211StaInfo>(Nl80211Attribute::StaInfo);
        let signal = station_attributes.unwrap().get_attribute(Nl80211StaInfo::Signal).unwrap().get_payload_as::<i8>().unwrap();
        self.signal = 2 * ((signal + 100) as usize)
    }
}

pub struct NetworkWidget {
    alignment: Alignment,
}

impl<'a> NetworkWidget {
    pub fn new() -> Self {
        Self {
            alignment: Alignment::Left,
        }
    }

    pub fn right_aligned(&mut self) {
        self.alignment = Alignment::Right;
    }
}

impl StatefulWidget for NetworkWidget {
    type State = NetworkState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut NetworkState) {
        let icon = match state.state {
            Connection::Connected => ["󰤯", "󰤟", "󰤢", "󰤥", "󰤨"][((state.signal)/25).clamp(0, 4)],
            Connection::Disconnected => "󰤮",
        };
        Paragraph::new(format!("{} {}% {} ", icon, state.signal, state.ssid))
            .alignment(self.alignment)
            .render(area, buf);
    }
}
