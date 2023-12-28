mod example2;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use rosc::{encoder, OscMessage, OscPacket, OscType};

use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;

use actix_web::dev::Server;
use std::io;

use tokio::net::{TcpSocket, TcpStream};

use actix_service::{fn_service, ServiceFactoryExt as _};

use bytes::BytesMut;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[actix_web::main]
async fn main() -> io::Result<()> {
    let os2l_addr = ("127.0.0.1", 9996);
    let osc_client_adddr = "127.0.0.1:7700";

    // let sock = UdpSocket::bind("127.0.0.1")?;
    let dst_addr = SocketAddrV4::from_str(osc_client_adddr).unwrap();

    Server::build()
        .bind("ServiceName", os2l_addr, move || {
            fn_service(move |mut stream: TcpStream| {
                async move {
                    let mut size = 0;
                    let mut buf = BytesMut::with_capacity(200);
                    println!("Connection initialized...starting");
                    let mut buff2 = vec![0; 1024];
                    // let tcpsock = TcpSocket::new_v4()?;
                    // let mut outstream = tcpsock.connect(osc_client_adddr).await?;
                    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();

                    let mut beat_mode = BeatMode::EVERY;
                    let mut flash_mode = FlashMode::BLACKOUT;

                    //create socket here?
                    // let mut msg_buf;

                    loop {
                        match stream.read(&mut buff2).await {
                            // end of stream; bail from loop
                            Ok(0) => break,

                            // write bytes back to stream
                            Ok(bytes_read) => {
                                let read_bytes = &buff2[..bytes_read];
                                let s = std::str::from_utf8(read_bytes)
                                    .expect("invalid utf-8 sequence");

                                println!("rcvd:  {:?}", s);
                                // outstream.write_all(&buff2[..bytes_read]).await?;
                                // stream.write_all(&buf[size..]).await.unwrap();
                                // size += bytes_read;

                                //convert message to json
                                if let Ok(p) = serde_json::from_str::<Os2lBeat>(s) {
                                    println!("rcd: {:?}", &p);

                                    //check if we should send the beat

                                    let should_send = match beat_mode {
                                        BeatMode::EVERY => true,
                                        BeatMode::FOURS => p.pos % 4 == 0,
                                        BeatMode::EIGHTS => p.pos % 8 == 0,
                                        BeatMode::SIXTEENS => p.pos % 16 == 0,
                                    };

                                    if should_send {
                                        let msg_buf =
                                            encoder::encode(&OscPacket::Message(OscMessage {
                                                addr: "/1/dj/beat".to_string(),
                                                args: vec![OscType::Int(0)],
                                            }))
                                            .unwrap();
                                        sock.send_to(&msg_buf, dst_addr).unwrap();

                                        let msg_buf =
                                            encoder::encode(&OscPacket::Message(OscMessage {
                                                addr: "/1/dj/beat".to_string(),
                                                args: vec![OscType::Int(255)],
                                            }))
                                            .unwrap();
                                        sock.send_to(&msg_buf, dst_addr).unwrap();

                                        // let msg_buf =
                                        //     encoder::encode(&OscPacket::Message(OscMessage {
                                        //         addr: "/1/dj/singlebeat".to_string(),
                                        //         args: vec![OscType::Int(255)],
                                        //     }))
                                        //     .unwrap();
                                        // sock.send_to(&msg_buf, dst_addr).unwrap();
                                        //
                                        // let msg_buf =
                                        //     encoder::encode(&OscPacket::Message(OscMessage {
                                        //         addr: "/1/dj/singlebeat".to_string(),
                                        //         args: vec![OscType::Int(0)],
                                        //     }))
                                        //     .unwrap();
                                        // sock.send_to(&msg_buf, dst_addr).unwrap();

                                        println!("sending: beat {}", p.pos);
                                    }

                                    // sock.send_to(&msg_buf, dst_addr).unwrap();
                                    // msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
                                    //     addr: "test/1".to_string(),
                                    //     args: vec![OscType::Int(255)],
                                    // }))
                                    // .unwrap();
                                } else if let Ok(p) = serde_json::from_str::<Os2lButton>(s) {
                                    let feedback_off = match p.name.as_str() {
                                        "hold" => get_flashname(&flash_mode),
                                        "fade" => get_flashname(&flash_mode),
                                        "flash" => get_flashname(&flash_mode),
                                        "blackout" => get_flashname(&flash_mode),
                                        "every" => get_beatname(&beat_mode),
                                        "four" => get_beatname(&beat_mode),
                                        "eight" => get_beatname(&beat_mode),
                                        "sixteen" => get_beatname(&beat_mode),
                                        &_ => {
                                            panic!("unknown command")
                                        }
                                    };

                                    //set new:
                                    match p.name.as_str() {
                                        "hold" => flash_mode = FlashMode::HOLD,
                                        "fade" => flash_mode = FlashMode::FADE,
                                        "flash" => flash_mode = FlashMode::FLASH,
                                        "blackout" => flash_mode = FlashMode::BLACKOUT,
                                        "every" => beat_mode = BeatMode::EVERY,
                                        "four" => beat_mode = BeatMode::FOURS,
                                        "eight" => beat_mode = BeatMode::EIGHTS,
                                        "sixteen" => beat_mode = BeatMode::SIXTEENS,
                                        &_ => {
                                            panic!("unknown command")
                                        }
                                    };

                                    //turn off the previous
                                    let feedback = Os2lFeedback {
                                        evt: "feedback".to_string(),
                                        name: feedback_off.to_string(),
                                        state: "off".to_string(),
                                    };

                                    let tosend = serde_json::to_string(&feedback).unwrap();
                                    println!("sending: feedback {:?}", &tosend);
                                    stream.write_all(tosend.as_bytes()).await.unwrap();

                                    let msg_buf =
                                        encoder::encode(&OscPacket::Message(OscMessage {
                                            addr: format!("/1/dj/{}", feedback_off),
                                            args: vec![OscType::Int(0)],
                                        }))
                                        .unwrap();
                                    sock.send_to(&msg_buf, dst_addr).unwrap();
                                    let msg_buf =
                                        encoder::encode(&OscPacket::Message(OscMessage {
                                            addr: format!("/1/dj/{}", feedback_off),
                                            args: vec![OscType::Int(255)],
                                        }))
                                        .unwrap();
                                    sock.send_to(&msg_buf, dst_addr).unwrap();

                                    //turn on what you pressed
                                    let feedback = Os2lFeedback {
                                        evt: "feedback".to_string(),
                                        name: p.name.clone(),
                                        state: "on".to_string(),
                                    };

                                    let tosend = serde_json::to_string(&feedback).unwrap();
                                    println!("sending: feedback {:?}", &tosend);
                                    stream.write_all(tosend.as_bytes()).await.unwrap();

                                    let msg_buf =
                                        encoder::encode(&OscPacket::Message(OscMessage {
                                            addr: format!("/1/dj/{}", &p.name),
                                            args: vec![OscType::Int(0)],
                                        }))
                                        .unwrap();
                                    sock.send_to(&msg_buf, dst_addr).unwrap();
                                    let msg_buf =
                                        encoder::encode(&OscPacket::Message(OscMessage {
                                            addr: format!("/1/dj/{}", &p.name),
                                            args: vec![OscType::Int(255)],
                                        }))
                                        .unwrap();
                                    sock.send_to(&msg_buf, dst_addr).unwrap();

                                    println!("sending: button {:?}", &p);
                                } else if let Ok(p) = serde_json::from_str::<Os2lCmd>(s) {
                                    match p.id {
                                        1 => {
                                            //send blackout
                                            let msg_buf =
                                                encoder::encode(&OscPacket::Message(OscMessage {
                                                    addr: "/1/dj/blackout".to_string(),
                                                    args: vec![OscType::Int(255)],
                                                }))
                                                .unwrap();
                                            sock.send_to(&msg_buf, dst_addr).unwrap();

                                            let msg_buf =
                                                encoder::encode(&OscPacket::Message(OscMessage {
                                                    addr: "/1/dj/blackout".to_string(),
                                                    args: vec![OscType::Int(0)],
                                                }))
                                                .unwrap();
                                            sock.send_to(&msg_buf, dst_addr).unwrap();
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            Err(err) => {
                                eprintln!("Stream Error: {:?}", err);
                                return Err(());
                            }
                        }
                    }

                    Ok(())
                }
            })
            .map_err(|err| eprintln!("Service Error: {:?}", err))
        })?
        .run()
        .await
}
//
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| {
//         App::new()
//             .service(hello)
//             .service(echo)
//             .route("/hey", web::get().to(manual_hello))
//     })
//     .bind(("127.0.0.1", 9996))?
//     .run()
//     .await
// }

// fn main() {

//provide server to accept os2l commands.

//process them

//forward them as osc

//take in os2l, convert to OSC

//
//
// // let args: Vec<String> = env::args().collect();
// // let usage = format!("Usage {} IP:PORT", &args[0]);
// // if args.len() < 2 {
// //     println!("{}", usage);
// //     ::std::process::exit(1)
// // }
// //
// let ip = "127.0.0.1:5005";
// let outport = 5006;
//
//
// let addr = match SocketAddrV4::from_str(ip) {
//     Ok(addr) => addr,
//     Err(_) => panic!("could not open {}", ip),
// };
// let sock = UdpSocket::bind(addr).unwrap();
// println!("Listening to {}", addr);
//
// let mut buf = [0u8; rosc::decoder::MTU];
//
// loop {
//     match sock.recv_from(&mut buf) {
//         Ok((size, addr)) => {
//             println!("Received packet with size {} from: {}", size, addr);
//             let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
//             handle_packet(packet);
//         }
//         Err(e) => {
//             println!("Error receiving from socket: {}", e);
//             break;
//         }
//     }
// }
// }

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("recvd")
}

#[post("/")]
async fn echo(req_body: String) -> impl Responder {
    println!("rcvd: {}", req_body);
    HttpResponse::Ok().body("rcvd")
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            println!("OSC address: {}", msg.addr);
            println!("OSC arguments: {:?}", msg.args);
        }
        OscPacket::Bundle(bundle) => {
            println!("OSC Bundle: {:?}", bundle);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Os2lBeat {
    evt: String,
    change: bool,
    pos: i32,
    bpm: f32,
    strength: f32,
    // evt (string): "beat"
    // change (bool): true if either the phase or the bpm changed since the last beat message
    // pos (integer): beat number (pos modulo 4 should be 0 on 4:4 measure boundaries, pos modulo 16 should be 0 on 16 beat phrases boundaries, etc)
    // bpm (double): number of beats per minute
    // strength (double): relative audible strenght of this beat compared to the strongest beat in the song (in percentage). (optional)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Os2lCmd {
    evt: String,
    id: i32,
    param: f32,
    // evt (string): "cmd"
    // id (integer): command id, that should be mapped to a trigger on the DMX software (similar to a MIDI command)
    // param (double): command parameter, between 0 and 100%
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Os2lButton {
    evt: String,
    name: String,
    page: Option<String>,
    state: String,
    // evt (string): "btn"
    // name (string): name of the button (will automatically activate any button with this name on the DMX software)
    // page (string): (optional) name of a page that contains the button. If empty, uses the master page. If set to "*", uses all pages. If set to "-", uses the page currently displayed in the DMX software
    // state (string): either "on" if the button is being pushed, or "off" if the button is being released
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Os2lFeedback {
    evt: String,
    name: String,
    // page: Option<String>,
    state: String,
    // evt (string): "feedback"
    // name (string): name of the button
    // page (string): (optional) name of a page that contains the button
    // state (string): either "on" to light the button on, or "off" to light it off
}

#[derive(PartialEq)]
enum BeatMode {
    EVERY,
    FOURS,
    EIGHTS,
    SIXTEENS,
}

#[derive(PartialEq)]
enum FlashMode {
    HOLD,
    FLASH,
    FADE,
    BLACKOUT,
}

fn get_flashname(f: &FlashMode) -> &'static str {
    match f {
        FlashMode::HOLD => "hold",
        FlashMode::FLASH => "flash",
        FlashMode::FADE => "fade",
        FlashMode::BLACKOUT => "blackout",
    }
}

fn get_beatname(b: &BeatMode) -> &'static str {
    match b {
        BeatMode::EVERY => "every",
        BeatMode::FOURS => "four",
        BeatMode::EIGHTS => "eight",
        BeatMode::SIXTEENS => "sixteen",
    }
}
