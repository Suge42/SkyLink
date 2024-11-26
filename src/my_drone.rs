use std::collections::HashMap;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use crossbeam_channel::{select_biased, Receiver, Sender};
use wg_2024::controller::{DroneCommand, NodeEvent};
use wg_2024::drone::{Drone, DroneOptions};
use wg_2024::packet::{Packet, PacketType, Nack};
use wg_2024::packet::Nack::{DestinationIsDrone, ErrorInRouting};
use rand::Rng;

pub struct MyDrone {
    id: NodeId,
    controller_send: Sender<NodeEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: u32,
    flood_ids: Vec<u64>,
}

impl Drone for MyDrone {
    fn new(options: DroneOptions) -> Self {
        MyDrone {
            id: options.id,
            controller_send: options.controller_send,
            controller_recv: options.controller_recv,
            packet_recv: options.packet_recv,
            //packet_send: options.packet_send,
            packet_send: HashMap::new(),
            pdr: (options.pdr*100.0) as u32,
            flood_ids: Vec::new(),
        }
    }

    fn run(&mut self) {
        loop {
            select_biased! {
                recv(self.controller_recv) -> cmd => {
                    if let Ok(command) = cmd {
                        if let DroneCommand::Crash = command {
                            println!("Drone {} has crashed", self.id);
                            break;
                        }
                        self.handle_command(command);
                    }
                }
                recv(self.packet_recv) -> pkt => {
                    if let Ok(packet) = pkt {
                        self.handle_packet(packet);
                    }
                }
            }
        }
    }
}

impl MyDrone {
    fn handle_command(&mut self, command: DroneCommand) {
        match command {
            DroneCommand::AddSender(node_id, sender) => {
                self.packet_send.insert(node_id, sender);
            },
            DroneCommand::SetPacketDropRate(pdr) => {
                self.pdr = (pdr*100.0) as u32;
            },
            DroneCommand::Crash => unreachable!(),
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        let position = packet.routing_header.hop_index;

        match packet.pack_type.clone() {
            PacketType::MsgFragment(_fragment_id) => {
                if position < packet.routing_header.hops.len() {
                    let next = packet.routing_header.hops[position];
                    if self.packet_send.contains_key(&next) {
                        let mut rng = rand::thread_rng();
                        let random_number: u32 = rng.gen_range(0..101);

                        if random_number > self.pdr {
                             if self.forward_packet(packet.clone(), &next) {
                                 return;
                             }
                        }
                    }
                    self.send_nack(ErrorInRouting(next),
                                         packet.routing_header.hops.clone(),
                                         packet.session_id
                    );

                } else {
                    self.send_nack(DestinationIsDrone,
                                         packet.routing_header.hops.clone(),
                                         packet.session_id
                    );
                }
            },
            PacketType::FloodRequest(_flood_request_id) => {

            },
            _ => {
                if position < packet.routing_header.hops.len() {
                    let next = packet.routing_header.hops[position];
                    if self.packet_send.contains_key(&next) {
                        if self.forward_packet(packet.clone(), &next){
                            return;
                        }
                    }
                    self.send_nack(ErrorInRouting(next),
                                         packet.routing_header.hops.clone(),
                                         packet.session_id
                    );
                } else{
                    self.send_nack(DestinationIsDrone,
                              packet.routing_header.hops.clone(),
                              packet.session_id
                    );
                }
            }
        }
    }

    fn forward_packet(&self, mut packet: Packet, next: &NodeId) -> bool{
        packet.routing_header.hop_index += 1;
        if let Ok(_) = self.packet_send.get(next).unwrap().send(packet.clone()) {
            self.controller_send.send(NodeEvent::PacketSent(packet)).unwrap();
            //document panic?
            true
        } else {
            false
        }
    }

    fn send_nack(&mut self, nack_type: Nack, routing_vector: Vec<NodeId>, session_id: u64) {
        let err = Packet {
            pack_type: PacketType::Nack(nack_type),
            routing_header: SourceRoutingHeader{
                hop_index: 1,
                hops: routing_vector
                    .into_iter()
                    .rev()
                    .collect::<Vec<NodeId>>()
            },
            session_id
        };
        self.handle_packet(err.clone());
        self.controller_send.send(NodeEvent::PacketSent(err)).unwrap();
    }
}