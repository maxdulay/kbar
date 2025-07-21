use pipewire::metadata::{Metadata, MetadataListener};
use pipewire::node::{Node, NodeListener};
use pipewire::spa::param::ParamType;
use pipewire::types::ObjectType;
use pipewire::{context::Context, main_loop::MainLoop};

use std::cell::RefCell;
use std::rc::Rc;

use libspa::pod::{Object, Pod, Value, ValueArray, deserialize::PodDeserializer};

use tokio::sync::mpsc;

use crate::event::Event;

#[derive(Clone, Debug)]
pub enum PipeWireEvent {
    UpdateVolumes(u32, Vec<f32>),
    UpdateMuted(u32, bool),
    SetDefaultSinkName(String),
    UpdateNodeId(u32, String),
}

pub fn deserialize(param: Option<&Pod>) -> Option<Object> {
    param
        .and_then(|pod| PodDeserializer::deserialize_any_from(pod.as_bytes()).ok())
        .and_then(|(_, value)| match value {
            Value::Object(obj) => Some(obj),
            _ => None,
        })
}

pub fn pw_monitor(sender: mpsc::UnboundedSender<Event>) {
    tokio::spawn(async move {
        let mainloop = MainLoop::new(None).unwrap();
        let context = Context::new(&mainloop).unwrap();
        let core = context.connect(None).unwrap();

        let registry = Rc::new(core.get_registry().unwrap());
        let registry_weak = Rc::downgrade(&registry);

        let nodes = Rc::new(RefCell::new(Vec::<(Node, NodeListener)>::new()));
        let metadatas = Rc::new(RefCell::new(Vec::<(Metadata, MetadataListener)>::new()));

        let _listener = registry
            .add_listener_local()
            .global(move |global| {
                if let Some(registry) = registry_weak.upgrade() {
                    match global.type_ {
                        ObjectType::Node => {
                            let obj_id = global.id;
                            let node: Node = registry.bind(global).unwrap();
                            let _sender = sender.clone();
                            let __sender = sender.clone();
                            let listener = node
                                .add_listener_local()
                                .info(move |info| {
                                    if let Some(name) = info.props().unwrap().get("node.name") {
                                        __sender
                                            .send(Event::UpdatePipeWireState(
                                                PipeWireEvent::UpdateNodeId(
                                                    info.id(),
                                                    name.to_string(),
                                                ),
                                            ))
                                            .unwrap();
                                    }
                                })
                                .param(move |_seq, id, _index, _next, param| {
                                    if let Some(param) = deserialize(param) {
                                        if id == ParamType::Props {
                                            for property in param.clone().properties {
                                                match property.key {
                                                    65540 => {
                                                        if let Value::Bool(mute_bool) =
                                                            property.value
                                                        {
                                                            _sender
                                                                .send(Event::UpdatePipeWireState(
                                                                    PipeWireEvent::UpdateMuted(
                                                                        obj_id, mute_bool,
                                                                    ),
                                                                ))
                                                                .unwrap();
                                                        }
                                                    }

                                                    65544 => {
                                                        if let Value::ValueArray(
                                                            ValueArray::Float(floats),
                                                        ) = property.value
                                                        {
                                                            _sender
                                                                .send(Event::UpdatePipeWireState(
                                                                    PipeWireEvent::UpdateVolumes(
                                                                        obj_id, floats,
                                                                    ),
                                                                ))
                                                                .unwrap();
                                                        }
                                                    }
                                                    _ => (),
                                                }
                                            }
                                        }
                                        match id {
                                            ParamType::Props => {}
                                            _ => (),
                                        }
                                    }
                                })
                                .register();
                            node.subscribe_params(&[ParamType::Props]);
                            nodes.borrow_mut().push((node, listener));
                        }
                        ObjectType::Metadata => {
                            let metadata: Metadata = registry.bind(global).unwrap();
                            let _sender = sender.clone();
                            let listener = metadata
                                .add_listener_local()
                                .property(move |_subject, key, _type, value| {
                                    if let Some(key) = key {
                                        if key == "default.audio.sink" {
                                            let name = value.unwrap_or("{\"name\":\"\"}");
                                            _sender
                                                .send(Event::UpdatePipeWireState(
                                                    PipeWireEvent::SetDefaultSinkName(
                                                        name.get(9..(name.len() - 2))
                                                            .unwrap()
                                                            .to_string(),
                                                    ),
                                                ))
                                                .unwrap();
                                        }
                                    }
                                    0
                                })
                                .register();
                            metadatas.borrow_mut().push((metadata, listener));
                        }
                        _ => (),
                    }
                }
            })
            .register();
        mainloop.run();
    });
}
