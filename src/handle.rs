use Record;

pub trait Handler: Send + Sync {
    fn handle(&self, rec: &mut Record) -> Result<(), ::std::io::Error>;
}
