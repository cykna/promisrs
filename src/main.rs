mod poller;
mod promises;
mod timers;
use poller::Poller;
use promises::{Promise, PromiseState};
use std::{
    cell::RefCell,
    io::{ErrorKind, Read},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
    rc::Rc,
};
use timers::{set_interval, set_timeout};

pub trait TcpHandler {
    fn on_acquire_connection(&mut self, connection: &(TcpStream, SocketAddr));
    fn on_receive_data(&mut self, buf: &mut [u8], len: usize, addr: SocketAddr);
}

struct TcpConnector<H: TcpHandler> {
    listener: TcpListener,
    clients: Vec<(TcpStream, SocketAddr)>,
    handler: H,
}
impl<H: TcpHandler> TcpConnector<H> {
    pub fn connect(addr: impl ToSocketAddrs, handler: H) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            clients: Vec::new(),
            handler,
        })
    }
    //Checks wheater it found some connection or not, returns false if the error is WouldBlock,
    //if the error is another one, returns it, if no error, means found a connection so returns
    //true
    pub fn check_connection(&mut self) -> std::io::Result<bool> {
        match self.listener.accept() {
            Ok(val) => {
                val.0.set_nonblocking(true)?;
                self.handler.on_acquire_connection(&val);
                self.clients.push(val);
                Ok(true)
            }
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
    ///Reads the contents of the clients into their respective buffers if there is some data
    ///pending
    pub fn read_clients(&mut self) -> std::io::Result<()> {
        let buf = &mut [0; 128];
        for client in self.clients.iter_mut() {
            match client.0.read(buf) {
                Ok(len) => self.handler.on_receive_data(buf, len, client.1),
                Err(e) => {
                    if e.kind() != ErrorKind::WouldBlock {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
}
impl<H: TcpHandler> Promise for TcpConnector<H> {
    fn poll(&mut self) -> PromiseState<Option<Box<dyn std::any::Any>>, Box<dyn std::error::Error>> {
        if let Err(e) = self.check_connection() {
            PromiseState::Rejected(Box::new(e))
        } else {
            self.read_clients().unwrap();
            PromiseState::Pending
        }
    }
    fn then(&mut self, _: Box<promises::PromiseCb>) {
        unimplemented!();
    }
}
struct TcpH;
impl TcpHandler for TcpH {
    fn on_acquire_connection(&mut self, connection: &(TcpStream, SocketAddr)) {
        println!("{connection:?}");
    }
    fn on_receive_data(&mut self, buf: &mut [u8], len: usize, addr: SocketAddr) {
        println!(
            "Look, i've got some data: {:?} from address {addr} with len {len}",
            &buf[..len]
        )
    }
}
fn main() -> std::io::Result<()> {
    let mut poller = Poller::new();
    let listener = TcpConnector::connect("127.0.0.1:6969", TcpH).unwrap();
    poller.schedule(listener);
    poller.schedule(set_interval(
        || {
            println!("after 5 secs");
        },
        5.0,
    ));
    poller.run();
    Ok(())
}
