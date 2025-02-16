mod poller;
mod promises;
mod timers;
use http::Response;
use httparse::{Request, Status};
use poller::Poller;
use promises::{Promise, PromiseState};
use std::{
    io::{ErrorKind, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

pub trait TcpHandler {
    fn on_acquire_connection(&mut self, connection: &(TcpStream, SocketAddr));
    fn on_receive_data(
        &mut self,
        buf: &[u8],
        len: usize,
        addr: SocketAddr,
    ) -> Result<Vec<u8>, httparse::Error>;
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
    pub fn read_clients(&mut self) {
        if let Ok(Some(indexes)) = self.read() {
            for idx in indexes {
                self.clients.swap_remove(idx);
            }
        }
    }
    fn read(&mut self) -> std::io::Result<Option<Vec<usize>>> {
        let mut idxs = Vec::new();
        let buf = &mut [0; 1024];
        for (idx, client) in self.clients.iter_mut().enumerate() {
            match client.0.read(buf) {
                Ok(len) => {
                    if len > 0 {
                        if let Ok(data) =
                            self.handler.on_receive_data(buf.as_slice(), len, client.1)
                        {
                            match client.0.write_all(&data) {
                                Ok(_) => client.0.flush().unwrap(),
                                Err(e) => println!("Qual foi mano {e:?}"),
                            };
                        } else {
                            client.0.write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                            client.0.flush().unwrap();
                        };
                    } else {
                        idxs.push(idx);
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => continue,
                    ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => {
                        idxs.push(idx);
                    }
                    err => {
                        panic!("{err:?}");
                    }
                },
            }
        }
        Ok(Some(idxs))
    }
}
impl<H: TcpHandler> Promise for TcpConnector<H> {
    fn poll(&mut self) -> PromiseState<Option<Box<dyn std::any::Any>>, Box<dyn std::error::Error>> {
        if let Err(e) = self.check_connection() {
            PromiseState::Rejected(Box::new(e))
        } else {
            self.read_clients();
            PromiseState::Pending
        }
    }
    fn then(&mut self, _: Box<promises::PromiseCb>) {
        unimplemented!();
    }
}
struct TcpH;

impl TcpH {
    fn on_get_req(&self, req: &Request) -> Response<String> {
        Response::builder()
            .status(200)
            .header("Content-Length", "4")
            .header("Connection", "close")
            .body("Damn".to_string())
            .unwrap()
    }
}

impl TcpHandler for TcpH {
    fn on_acquire_connection(&mut self, connection: &(TcpStream, SocketAddr)) {
        println!("{connection:?}");
    }

    fn on_receive_data(
        &mut self,
        buf: &[u8],
        len: usize,
        addr: SocketAddr,
    ) -> Result<Vec<u8>, httparse::Error> {
        println!("{}", String::from_utf8_lossy(&buf[..len]));
        let headers = &mut [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(headers);
        match req.parse(buf) {
            Ok(v) => {
                if let Status::Complete(_) = v {
                    let res = self.on_get_req(&req);
                    let (parts, body) = res.into_parts();
                    let status_line = format!("{:?} {} \r\n", parts.version, parts.status);
                    let mut header_string = String::new();
                    for (key, value) in parts.headers.iter() {
                        header_string.push_str(&format!(
                            "{}: {}\r\n",
                            key,
                            value.to_str().unwrap()
                        ));
                    }
                    let response_string = format!("{}{}\r\n{}", status_line, header_string, body);
                    println!("{}", response_string);
                    let bytes = response_string.into_bytes();
                    Ok(bytes)
                } else {
                    Ok(vec![])
                }
            }
            Err(e) => {
                println!("Deu p fazer o request não paehzao {e}");
                Err(e)
            }
        }
    }
}
fn main() -> std::io::Result<()> {
    let connector = TcpConnector::connect("127.0.0.1:6969", TcpH).unwrap();
    let mut poller = Poller::new();
    poller.schedule(connector);
    poller.run();
    Ok(())
}
