use std::io::{self, BufRead, Write};
use std::io::{BufReader, Cursor};
use std::net::TcpStream;
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

    /// Send the initial callsign (or any other command) to the server.
    pub fn send_callsign(&mut self, callsign: &str) -> io::Result<()> {
        let cs = format!("{}\r\n", callsign);
        self.stream.write_all(cs.as_bytes())?;
        self.stream.flush()
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
impl LineSource for RealTelnet {}

/// `MockTelnet` pretends to be a Telnet connection but simply reads from an
/// in‑memory buffer. The buffer can contain any number of lines you want to
/// test against.
pub struct MockTelnet {
    /// The writer part – we keep it so the code that does `write_all` still works.
    writer: Cursor<Vec<u8>>,
    /// The reader part – a `BufReader` over the same underlying cursor.
    reader: BufReader<Cursor<Vec<u8>>>,
}

impl MockTelnet {
    /// Build a mock from a static string (or any `&[u8]` you like).
    pub fn from_bytes(data: &[u8]) -> Self {
        let writer = Cursor::new(Vec::new());
        // Clone the data for the reader side.
        let reader_cursor = Cursor::new(data.to_vec());
        let reader = BufReader::new(reader_cursor);
        Self { writer, reader }
    }

    /// Helper that mimics the “send callsign” step – it just discards the data.
    pub fn send_callsign(&mut self, _callsign: &str) -> io::Result<()> {
        // In a mock we don’t need to do anything; the test data is already
        // present in the read buffer.
        Ok(())
    }
}

/* Trait impls ----------------------------------------------------------- */
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
impl LineSource for MockTelnet {}

pub const TEST_DATA: &str = "\
Please enter your call: HB9HUS
Hello, HB9HUS! Connected.
Local users: 208
Spot rate: 4/s (14,172/h)
HB9HUS de RELAY 15-Dec-2025 14:48Z >

DX de OK1FCJ-#:   7004.0  SP/DH2UAI      CW    20 dB  16 WPM  CQ      1448Z
DX de NU4F-#:    28205.0  N9LXP/B        CW     5 dB  15 WPM  BEACON  1448Z
DX de DF2CK-#:    1854.0  OK0EV          CW     3 dB  16 WPM  BEACON  1448Z
DX de DF2CK-#:   28254.6  K4JEE/B        CW     4 dB  15 WPM  BEACON  1448Z
DX de DF2CK-#:    7037.6  IZ3DVW/B       CW     5 dB  16 WPM  BEACON  1448Z
DX de DF2CK-#:   14055.0  DL0IN          CW    14 dB  10 WPM  CQ      1448Z
DX de DF2CK-#:    3600.0  OK0EN          CW    10 dB  10 WPM  BEACON  1448Z
DX de MM0GPZ-#:  18097.8  IK6BAK/B       CW     5 dB  12 WPM  BEACON  1448Z
DX de MM0GPZ-#:  14055.0  DL0IN          CW    35 dB  10 WPM  CQ      1448Z
DX de SQ5J-#:     7004.1  SP/DH2UAI      CW     6 dB  16 WPM  CQ      1448Z
DX de K5TR-#:    28234.4  WS2K/B         CW    28 dB  17 WPM  BEACON  1448Z
DX de K5TR-#:    28221.8  W1DLO/B        CW     6 dB  18 WPM  BEACON  1448Z
DX de NU4F-2-#:  28205.0  N9LXP/B        CW    12 dB  15 WPM  BEACON  1448Z
DX de W6YX-#:     7002.5  JA1GZV         CW     9 dB  21 WPM  CQ      1448Z
DX de SM7IUN-#:   7004.0  SP/DH2UAI      CW    20 dB  16 WPM  CQ      1448Z
DX de WZ7I-#:    28269.9  IZ8FFZ/B       CW    16 dB  15 WPM  BEACON  1448Z
DX de DK0TE-#:    7004.0  SP/DH2UAI      CW    21 dB  16 WPM  CQ      1448Z
DX de DR4W-#:     7004.0  SP/DH2UAI      CW    13 dB  16 WPM  CQ      1448Z
DX de K3LR-#:    28024.0  EA6ACA         CW    59 dB  17 WPM  CQ      1448Z
DX de K1RA-#:    28024.0  EA6ACA         CW    12 dB  16 WPM  CQ      1448Z
DX de S54L-#:     7004.0  SP/DH2UAI      CW    16 dB  16 WPM  CQ      1448Z
DX de W3OA-#:    21024.0  F/DK9HE        CW     9 dB  24 WPM  CQ      1448Z
DX de UA4M-#:    18077.8  DL1CW          CW     6 dB  29 WPM  CQ      1448Z
DX de UA4M-#:     3527.4  YO4AR          CW    21 dB  27 WPM  CQ      1448Z
DX de UA4M-#:     3530.9  YO4CAH         CW    12 dB  27 WPM  CQ      1448Z
DX de UA4M-#:     3529.9  YO4BEX         CW    13 dB  30 WPM  CQ      1448Z
DX de UA4M-#:     3522.5  YO4CAI         CW    10 dB  24 WPM  CQ      1448Z
DX de OK1HRA-#:   7004.0  SP/DH2UAI      CW    18 dB  16 WPM  CQ      1448Z
";
