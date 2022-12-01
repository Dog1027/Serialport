use tokio_serial::{SerialPortBuilderExt, };

use tokio::{self, sync};

use std::io::{stdin, stdout, Write};

use crossterm::{
    event::{read, Event, KeyCode, KeyModifiers,
    },
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The communications port
    #[arg(short, long, default_value_t = String::from("COM1"))]
    portname: String,

    /// The baud rate
    #[arg(short, long, default_value_t = 115200)]
    baudrate: u32,

    /// Setting mode
    #[arg(short, long)]
    setting: bool,
}

fn print_events(tx: sync::mpsc::UnboundedSender<u8>) -> Result<()> {
    loop {
        // Wait up to 1s for another event
        // if poll(Duration::from_millis(1_000))? {
            // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
            let event = read()?;

            // println!("Event::{:?}\r", event);

            if let Event::Key(key_event) = event {
                match key_event.modifiers {
                    KeyModifiers::CONTROL => {
                        match key_event.code {
                            KeyCode::Char('c') => {
                                tx.send(3u8).unwrap();
                            }
                            KeyCode::Char('k') => {
                                tx.send(11u8).unwrap();
                            }
                            KeyCode::Char('u') => {
                                tx.send(21u8).unwrap();
                            }
                            KeyCode::Char('a') => {
                                drop(tx);
                                break;
                            },
                            _ => {
                                
                            }
                        }
                    },
                    KeyModifiers::SHIFT =>  {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                tx.send(c as u8).unwrap();
                            }
                            _ => {
                                
                            }
                        }
                    },
                    KeyModifiers::NONE =>  {
                        match key_event.code {
                            KeyCode::Home => {
                                tx.send(1u8).unwrap(); 
                            },
                            KeyCode::End => {
                                tx.send(5u8).unwrap(); 
                            },
                            KeyCode::Backspace => {
                                tx.send(8u8).unwrap(); 
                            },
                            KeyCode::Tab => {
                                tx.send(9u8).unwrap(); 
                            },
                            KeyCode::Enter => {
                                tx.send(13u8).unwrap(); 
                            },
                            KeyCode::Esc => {
                                tx.send(27u8).unwrap(); 
                            },
                            KeyCode::Delete => {
                                tx.send(27u8).unwrap();
                                tx.send('[' as u8).unwrap();
                                tx.send('3' as u8).unwrap();
                                tx.send('~' as u8).unwrap();
                            },
                            KeyCode::Up => {
                                tx.send(27u8).unwrap();
                                tx.send('[' as u8).unwrap();
                                tx.send('A' as u8).unwrap();
                            },
                            KeyCode::Down => {
                                tx.send(27u8).unwrap();
                                tx.send('[' as u8).unwrap();
                                tx.send('B' as u8).unwrap();
                            },
                            KeyCode::Right => {
                                tx.send(27u8).unwrap();
                                tx.send('[' as u8).unwrap();
                                tx.send('C' as u8).unwrap();
                            },
                            KeyCode::Left => {
                                tx.send(27u8).unwrap();
                                tx.send('[' as u8).unwrap();
                                tx.send('D' as u8).unwrap();
                            },
                            KeyCode::Char(c) => {
                                tx.send(c as u8).unwrap(); 
                            },
                            _ => {
                                
                            }
                        }
                    },
                    _ => {

                    }
                }
            }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    
    let args = Args::parse();
    let (tx, mut rx) = sync::mpsc::unbounded_channel();

    let port_name = if args.setting {
        print!("Port name: ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        String::from(buf.trim())
    }
    else {
        args.portname// String::from("COM4"); 
    };

    let baud_rate = if args.setting {
        print!("Baud rate: ");
        stdout().flush().unwrap();
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        buf.trim().parse::<u32>().unwrap()
    }
    else {
        args.baudrate // 115200;
    };


    let mut port = tokio_serial::new(&port_name, baud_rate).open_native_async().unwrap();
    let mut buf = [0u8; 32];

    println!("{{{}, {}}} Port Opened!", port_name, baud_rate);

    tokio::task::spawn_blocking(move || {

        enable_raw_mode().unwrap();

        // let mut stdout = stdout();
        // execute!(stdout, PushKeyboardEnhancementFlags(
        //     KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        // ))?;
    
        if let Err(e) = print_events(tx) {
            println!("Error: {:?}\r", e);
        }
    
        // execute!(stdout, DisableMouseCapture)?;
    
        disable_raw_mode()

    });

    loop {
        tokio::select! {
            Ok(_) = port.readable() => {
                if let Ok(n) = port.try_read(&mut buf) {
                    for i in 0..n {
                        print!("{}", buf[i] as char);
                        stdout().flush().unwrap();
                    }
                }
            },
            option = rx.recv() => {
                match option {
                    Some(msg) => {
                        loop {
                            if let Ok(_) = port.writable().await {
                                if let Ok(_) = port.try_write(&mut [msg;1]) {
                                    break;
                                }
                            }
                        }
                    }
                    None => {
                        break;
                    }
                }
            },
        };
    }

}
