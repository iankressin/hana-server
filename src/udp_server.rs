use crate::utils::response_packet;
use bytes::Bytes;
use dns_message_parser::question::Question;
use dns_message_parser::Dns;
use dns_message_parser::DomainName;
use net2::UdpBuilder;
use std::net::{SocketAddr, UdpSocket};

pub struct UdpServer;

impl UdpServer {
    pub fn new() -> UdpServer {
        UdpServer
    }

    pub fn listen(&self, keep_alive: &bool) -> Result<(), std::io::Error> {
        println!("UDP Listening ... ");

        let mut buf = [0 as u8; 512];

        // Used net2 instead of std::net in order to reuse the address
        let socket = UdpBuilder::new_v4()?
            .reuse_address(true)?
            .bind("224.0.0.251:5353")?;

        let (received, src_addr) = socket.recv_from(&mut buf).expect("Didnt received any data");

        println!("New connection: {}, {}", received, src_addr);

        match self.get_domain_name(buf, received) {
            Ok(domain_name) => {
                self.send_response(&socket, &domain_name, &src_addr)?;
                if *keep_alive {
                    self.listen(&keep_alive)?;
                }

                Ok(())
            }
            Err(err) => {
                self.listen(&keep_alive)?;
                Err(err)
            }
        }
    }

    fn send_response(
        &self,
        socket: &UdpSocket,
        domain_name: &DomainName,
        src_addr: &SocketAddr,
    ) -> Result<(), std::io::Error> {
        let same = domain_name.eq(&"_drive.local.");

        if same {
            socket.connect(src_addr).expect("Could not connect");
            let dns_packet = response_packet();

            socket
                .send(&dns_packet[..])
                .expect("Could not send message");
        }

        Ok(())
    }

    fn get_domain_name(
        &self,
        buf: [u8; 512],
        received: usize,
    ) -> Result<DomainName, std::io::Error> {
        let packet = Bytes::copy_from_slice(&buf[..received]);

        match Dns::decode(packet) {
            Ok(Dns { questions, .. }) => {
                let Question { domain_name, .. } = questions
                    .first()
                    .expect("No question was asked in this packet");

                println!("{:#?}", domain_name);

                Ok(domain_name.clone())
            }
            Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!")),
        }
    }
}
