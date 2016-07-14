use std::io::{stdout, Write};

use libc;

use meta::format::{FormatSpec, Formatter};
use handle::Handle;
use record::Record;

pub struct Dev;

impl Handle for Dev {
    fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error> {
        // {timestamp} {severity:.1s} {process}/{process:d} - {message}\r\n{name}: {value}\r\n
        // ^gray       ^vary          ^gray                   ^bright
        let mut buf = Vec::with_capacity(512);

        write!(buf, "\x1B[2;m")?;
        write!(buf, "{}", rec.datetime().format("%+"))?;
        write!(buf, "\x1B[0m")?;

        buf.write_all(b" ")?;
        let mut spec = FormatSpec::default();
        spec.precision = Some(1);
        let sev = rec.severity();
        write!(buf, "\x1B[")?;
        let color = match sev {
            1 => 9,
            2 => 3,
            3 => 2,
            4 => 10,
            _ => 11,
        };
        write!(buf, "38;5;{}m", color)?;
        rec.severity_format()(sev, &mut Formatter::new(&mut buf, spec))?;
        write!(buf, "\x1B[0m")?;

        write!(buf, "\x1B[2;m")?;
        write!(buf, " [{:#x}/{}]", rec.thread(), unsafe { libc::getpid() })?;

        buf.write_all(b" - ")?;
        write!(buf, "\x1B[0m")?;

        write!(buf, "\x1B[")?;
        write!(buf, "37m")?;
        buf.write_all(rec.message().as_bytes())?;
        write!(buf, "\x1B[0m")?;
        buf.write_all(b"\r\n")?;

        for meta in rec.iter() {
            buf.write_all(b"\t")?;
            write!(buf, "\x1B[")?;
            write!(buf, "37m")?;
            buf.write_all(meta.name.as_bytes())?;
            write!(buf, "\x1B[0m")?;
            buf.write_all(b": ")?;
            write!(buf, "\x1B[2;m")?;
            meta.value.format(&mut Formatter::new(&mut buf, Default::default()))?;
            write!(buf, "\x1B[0m")?;
            buf.write_all(b"\r\n")?;
        }

        let out = stdout();
        let mut wr = out.lock();
        wr.write_all(&buf)
    }
}
