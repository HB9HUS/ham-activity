use std::io::{self, BufRead, Write};
use std::io::{BufReader, Cursor};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// Anything that can give us lines of text, and that we can also write to
/// (the Telnet protocol expects us to send a callsign first).
pub trait LineSource: Write + BufRead {
    /// Convenience wrapper – reads a line into `buf` and returns the number of
    /// bytes read (0 == EOF). Mirrors `BufRead::read_line`.
    fn read_next_line(&mut self, buf: &mut String) -> io::Result<usize> {
        buf.clear();
        self.read_line(buf)
    }
    fn send_callsign(&mut self, callsign: &str) -> io::Result<()>;
}

/// Wrapper that owns the underlying `TcpStream` and a `BufReader`.
pub struct RealTelnet {
    stream: TcpStream,
    reader: io::BufReader<TcpStream>,
}

impl RealTelnet {
    pub fn connect(host: &str, port: u16) -> io::Result<Self> {
        let addr = (host, port);
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let reader = io::BufReader::new(stream.try_clone()?);
        Ok(Self { stream, reader })
    }
}

impl Write for RealTelnet {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}
impl BufRead for RealTelnet {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.reader.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt);
    }
}
impl io::Read for RealTelnet {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}
impl LineSource for RealTelnet {
    fn send_callsign(&mut self, callsign: &str) -> io::Result<()> {
        let cs = format!("{}\r\n", callsign);
        self.stream.write_all(cs.as_bytes())?;
        self.stream.flush()
    }
}

/// `MockTelnet` pretends to be a Telnet connection but simply reads from an
/// in‑memory buffer. The buffer can contain any number of lines you want to
/// test against.
pub struct MockTelnet {
    /// The writer part – we keep it so the code that does `write_all` still works.
    writer: Cursor<Vec<u8>>,
    /// The reader part – a `BufReader` over the same underlying cursor.
    reader: BufReader<Cursor<Vec<u8>>>,
    delay_per_read: Duration,
}

impl MockTelnet {
    /// Build a mock from a static string (or any `&[u8]` you like).
    pub fn from_bytes(data: &[u8]) -> Self {
        let writer = Cursor::new(Vec::new());
        // Clone the data for the reader side.
        let reader_cursor = Cursor::new(data.to_vec());
        let reader = BufReader::new(reader_cursor);
        Self {
            writer,
            reader,
            delay_per_read: Duration::ZERO,
        }
    }
    /// Build a mock from a static string (or any `&[u8]` you like).
    /// adds the specified delay to each read to simulate the speed
    /// data is generated
    pub fn from_bytes_with_delay(data: &[u8], delay_per_read: Duration) -> Self {
        let writer = Cursor::new(Vec::new());
        // Clone the data for the reader side.
        let reader_cursor = Cursor::new(data.to_vec());
        let reader = BufReader::new(reader_cursor);
        Self {
            writer,
            reader,
            delay_per_read,
        }
    }
}

impl Write for MockTelnet {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Append to the internal buffer – useful if you want to verify what the
        // caller wrote (e.g., that it sent the correct callsign).
        self.writer.get_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl BufRead for MockTelnet {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.delay_per_read != Duration::ZERO {
            let delay = self.delay_per_read;
            thread::sleep(delay);
        }
        self.reader.fill_buf()
    }
    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt);
    }
}
impl io::Read for MockTelnet {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}
impl LineSource for MockTelnet {
    fn send_callsign(&mut self, _callsign: &str) -> io::Result<()> {
        Ok(())
    }
}
